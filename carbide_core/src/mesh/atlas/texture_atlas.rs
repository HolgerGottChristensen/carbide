use std::collections::HashMap;

use fxhash::{FxBuildHasher, FxHasher, FxHashMap};
use image::{GenericImage, GenericImageView};
use rusttype::{GlyphId, Point, Rect};

use crate::draw::{Dimension, Position};
use crate::mesh::atlas::lossy_glyph_info::LossyGlyphInfo;
use crate::Scalar;
use crate::text::{Font, FontFamily, FontId, FontSize, FontStyle, FontWeight, Glyph};
use crate::widget::{Environment, GlobalState};

type ImageId = crate::image_map::Id;
type ImageData = image::DynamicImage;
pub type TextureAtlasIndex = usize;

const SHELVE_WIDTH: u32 = 512;

#[derive(Copy, Clone, Hash, Eq, PartialEq, Debug)]
pub enum AtlasId {
    Image(ImageId),
    RasterGlyph(FontId, GlyphId, u32),
    LossyGlyph(LossyGlyphInfo),
}

/// Inspired by the gpu_cache from rusttype
/// Another interesting source: https://nical.github.io/posts/etagere.html
#[derive(Debug)]
pub struct TextureAtlas {
    /// The width and height of the atlas in pixels. The atlas should be a multiple of SHELVE_WIDTH
    width: u32,
    height: u32,

    /// The queue for new not yet added textures
    id_queue: Vec<AtlasId>,
    /// This contains the image_data and a list of glyph_index to insert the queued book index into.
    data_queue: Vec<(ImageData, Vec<usize>)>,

    shelves: Vec<Shelf>,

    all_books_cabinet: FxHashMap<AtlasId, TextureAtlasIndex>,
    glyph_index: Vec<usize>,
    book_index: Vec<Book>,
    position_tolerance: Scalar,
}

#[derive(Copy, Clone, Debug)]
pub struct Book {
    pub x: u32,
    pub y: u32,
    pub width: u32,
    pub height: u32,
    pub is_active: bool,
}

/// Each section of rows in the atlas is a shelf
#[derive(Debug)]
pub struct Shelf {
    shelf_height: u32,
    shelf_y: u32,
    shelf_current_x: u32,
    books: Vec<Book>,
}

impl Shelf {
    fn append<F: FnMut(u32, u32, &ImageData)>(&mut self, atlas_id: AtlasId, image_data: &ImageData, uploader: &mut F) -> (AtlasId, Book) {
        let book = Book {
            x: self.shelf_current_x,
            y: self.shelf_y,
            width: image_data.width(),
            height: image_data.height(),
            is_active: true,
        };

        (*uploader)(self.shelf_current_x, self.shelf_y, image_data);
        self.shelf_current_x += book.width + 1;
        self.books.push(book);

        (atlas_id, book)
    }

    fn get_width_left(&self) -> u32 {
        SHELVE_WIDTH - self.shelf_current_x
    }
}


impl TextureAtlas {
    pub fn new(width: u32, height: u32) -> TextureAtlas {
        TextureAtlas {
            width,
            height,
            id_queue: vec![],
            data_queue: vec![],
            shelves: vec![],
            all_books_cabinet: HashMap::with_hasher(FxBuildHasher::default()),
            glyph_index: vec![],
            book_index: vec![],
            position_tolerance: 0.2,
        }
    }

    pub fn width(&self) -> u32 {
        self.width
    }

    pub fn height(&self) -> u32 {
        self.height
    }

    /*pub fn get_glyph_index(&mut self, font_id: FontId, glyph_id: GlyphId, font_size: FontSize, position: Position) -> Option<TextureAtlasIndex> {
        let offset = (position.fraction_0_1() / self.position_tolerance).round_to_u16();
        let atlas_id = AtlasId::LossyGlyph(LossyGlyphInfo::new(font_id, glyph_id, font_size, offset));

        self.all_books_cabinet.get(&atlas_id).cloned()
    }

    pub fn get_raster_glyph_index(&mut self, font_id: FontId, id: GlyphId, font_size: FontSize) -> Option<TextureAtlasIndex> {
        let atlas_id = AtlasId::RasterGlyph(font_id, id, font_size);

        self.all_books_cabinet.get(&atlas_id).cloned()
    }

    pub fn get_image_index(&mut self, image_id: ImageId) -> Option<TextureAtlasIndex> {
        let atlas_id = AtlasId::Image(image_id);

        self.all_books_cabinet.get(&atlas_id).cloned()
    }*/

    pub fn queue_glyph(&mut self, glyph: &mut Glyph, font: &Font, scale_factor: Scalar) {
        let texture_index = if font.is_bitmap() {
            self.queue_raster_glyph_id(glyph.id(), glyph.font_size(), font, scale_factor)
        } else {
            self.queue_glyph_id(glyph.id(), glyph.font_size(), glyph.position(), font, scale_factor)
        };

        //println!("Queue glyph at: {}", glyph.position());

        glyph.set_texture_index(texture_index);
    }

    pub fn queue_glyph_id(&mut self, glyph_id: GlyphId, font_size: FontSize, position: Position, font: &Font, scale_factor: Scalar) -> TextureAtlasIndex {
        let offset = (position.fraction_0_1() / (self.position_tolerance * scale_factor)).round_to_u16();
        let atlas_id = AtlasId::LossyGlyph(LossyGlyphInfo::new(font.id(), glyph_id, font_size, offset));
        let next_glyph_index = self.glyph_index.len();

        if !self.all_books_cabinet.contains_key(&atlas_id) {
            let image_data = font.get_glyph_image_from_id(glyph_id, font_size, scale_factor, position.fraction_0_1());
            if let Some(image_data) = image_data {
                if let Some(id_index) = self.id_queue.iter().position(|i| i == &atlas_id) {
                    self.data_queue[id_index].1.push(next_glyph_index);
                } else {
                    self.id_queue.push(atlas_id);
                    self.data_queue.push((image_data, vec![next_glyph_index]));
                }
            }
            self.glyph_index.push(0);
        } else {
            let book_index = self.all_books_cabinet.get(&atlas_id).unwrap();
            self.glyph_index.push(*book_index);
        }

        next_glyph_index
    }

    pub fn queue_image(&mut self, image_id: ImageId, image_data: ImageData) -> TextureAtlasIndex {
        let atlas_id = AtlasId::Image(image_id);
        let next_glyph_index = self.glyph_index.len();

        if !self.all_books_cabinet.contains_key(&atlas_id) {
            if let Some(id_index) = self.id_queue.iter().position(|i| i == &atlas_id) {
                self.data_queue[id_index].1.push(next_glyph_index);
            } else {
                self.id_queue.push(atlas_id);
                self.data_queue.push((image_data, vec![next_glyph_index]));
            }
            self.glyph_index.push(0);
        } else {
            let book_index = self.all_books_cabinet.get(&atlas_id).unwrap();
            self.glyph_index.push(*book_index);
        }

        next_glyph_index
    }

    pub fn queue_raster_glyph(&mut self, c: char, font_size: FontSize, font: &Font, scale_factor: Scalar) -> TextureAtlasIndex {
        let id = font.get_glyph_id(c);
        self.queue_raster_glyph_id(id.unwrap(), font_size, font, scale_factor)
    }

    pub fn queue_raster_glyph_id(&mut self, id: GlyphId, font_size: FontSize, font: &Font, scale_factor: Scalar) -> TextureAtlasIndex {
        let atlas_id = AtlasId::RasterGlyph(font.id(), id, font_size);
        let next_glyph_index = self.glyph_index.len();

        if !self.all_books_cabinet.contains_key(&atlas_id) {
            if let Some(id_index) = self.id_queue.iter().position(|i| i == &atlas_id) {
                self.data_queue[id_index].1.push(next_glyph_index);
            } else {
                let image_data = font.get_glyph_raster_image_from_id(id, font_size, scale_factor).unwrap();
                self.id_queue.push(atlas_id);
                self.data_queue.push((image_data, vec![next_glyph_index]));
            }
            self.glyph_index.push(0);
        } else {
            let book_index = self.all_books_cabinet.get(&atlas_id).unwrap();
            self.glyph_index.push(*book_index);
        }

        next_glyph_index
    }

    pub fn get_tex_coords_by_index(&self, id: TextureAtlasIndex) -> Rect<f32> {
        let book = self.book_index[self.glyph_index[id]];
        Rect {
            min: Point {
                x: book.x as f32 / self.width as f32,
                y: book.y as f32 / self.height as f32,
            },
            max: Point {
                x: (book.x as f32 + book.width as f32) / self.width as f32,
                y: (book.y as f32 + book.height as f32) / self.height as f32,
            },
        }
    }

    /// The uploader should be x, y, image_data
    pub fn cache_queued<F: FnMut(u32, u32, &ImageData)>(
        &mut self,
        mut uploader: F,
    ) {
        // Sort to get the smallest images first. Todo: make sorting biggest first
        let mut zipped_queue = self.id_queue.iter().zip(self.data_queue.iter()).collect::<Vec<_>>();

        zipped_queue.sort_unstable_by(|(_, (a, _)), (_, (b, _))| {
            let a_height = a.height();
            let b_height = b.height();
            b_height.cmp(&a_height)
        });

        while !zipped_queue.is_empty() {
            let (atlas_id, (image_data, glyph_index_to_change)) = zipped_queue.remove(0);
            let (book_id, book) = if let Some(shelf) = self.get_fitting_shelve(image_data.height(), image_data.width()) {
                self.shelves[shelf].append(*atlas_id, image_data, &mut uploader)
            } else {
                let shelf = self.new_shelf(image_data.height());
                self.shelves.push(shelf);
                let new_shelf_number = (self.shelves.len() - 1);
                self.shelves[new_shelf_number].append(*atlas_id, image_data, &mut uploader)
            };

            let next_book_index = self.book_index.len();
            self.book_index.push(book);

            self.all_books_cabinet.insert(book_id, next_book_index);

            for index in glyph_index_to_change {
                self.glyph_index[*index] = next_book_index;
            }
        }

        self.id_queue.clear();
        self.data_queue.clear();
    }

    fn new_shelf(&self, height: u32) -> Shelf {
        let shelf_y = self.shelves.last().map(|s| s.shelf_y + s.shelf_height + 1).unwrap_or(0);
        Shelf {
            shelf_height: height,
            shelf_y,
            shelf_current_x: 0,
            books: vec![],
        }
    }

    fn get_fitting_shelve(&self, height: u32, width: u32) -> Option<usize> {
        for (shelf_number, shelf) in self.shelves.iter().enumerate() {
            // We want to see it fit, but not if there is too much space in the shelf
            if height <= shelf.shelf_height && height > shelf.shelf_height / 2 {
                // If there is horizontal space enough to fit.
                if shelf.get_width_left() >= width + 1 {
                    return Some(shelf_number);
                }
            }
        }

        None
    }
}

#[test]
fn create_packed_image() {
    let mut atlas = TextureAtlas::new(512, 512);
    let image1 = "/Users/holgergottchristensen/carbide/target/smile.png";
    let image2 = "/Users/holgergottchristensen/carbide/target/smile_new.png";

    let mut env = Environment::<String>::new(vec![], [0.0, 0.0], 1.0);
    let mut family = FontFamily::new("Apple Color Emoji");
    family.add_font("/System/Library/Fonts/Apple Color Emoji.ttc", FontWeight::Normal, FontStyle::Normal);
    env.add_font_family(family);

    let mut family = FontFamily::new("Noto Sans");
    family.add_font("fonts/NotoSans/NotoSans-Regular.ttf", FontWeight::Normal, FontStyle::Normal);
    env.add_font_family(family);

    let id = env.get_font(0).get_glyph_id('👴').unwrap();

    atlas.queue_image(0.into(), image::open(image1).unwrap());
    atlas.queue_image(0.into(), image::open(image1).unwrap());
    atlas.queue_image(1.into(), image::open(image2).unwrap());
    atlas.queue_image(1.into(), image::open(image2).unwrap());
    atlas.queue_image(1.into(), image::open(image2).unwrap());
    atlas.queue_image(1.into(), image::open(image2).unwrap());
    atlas.queue_image(1.into(), image::open(image2).unwrap());
    atlas.queue_image(1.into(), image::open(image2).unwrap());
    atlas.queue_image(1.into(), image::open(image2).unwrap());
    atlas.queue_image(1.into(), image::open(image2).unwrap());
    atlas.queue_image(1.into(), image::open(image2).unwrap());
    atlas.queue_image(1.into(), image::open(image2).unwrap());
    /*atlas.queue_raster_glyph(0, '🥰', 32, &env);
    atlas.queue_raster_glyph(0, '🥶', 32, &env);
    atlas.queue_raster_glyph(0, '🐱', 32, &env);
    atlas.queue_raster_glyph(0, '🏆', 32, &env);
    atlas.queue_raster_glyph(0, '🎁', 32, &env);
    atlas.queue_raster_glyph(0, '🟥', 32, &env);
    atlas.queue_raster_glyph(0, '👬', 32, &env);
    atlas.queue_raster_glyph_id(0, id, 32, &env);
    atlas.queue_glyph(1, 'A', 32, Position::new(0.0, 0.0), &env);
    atlas.queue_glyph(1, 'A', 32, Position::new(20.0, 0.0), &env);
    atlas.queue_glyph(1, 'A', 32, Position::new(0.5, 0.5), &env);*/

    let mut texture = image::DynamicImage::new_rgba8(512, 512);

    atlas.cache_queued(|x, y, image_data| {
        println!("Insert the image at: {}, {} with size {}, {}", x, y, image_data.width(), image_data.height());
        for (ix, iy, pixel) in image_data.pixels() {
            texture.put_pixel(x + ix, y + iy, pixel);
        }
    });

    texture.save("/Users/holgergottchristensen/carbide/target/smile_atlas.png").unwrap();
}