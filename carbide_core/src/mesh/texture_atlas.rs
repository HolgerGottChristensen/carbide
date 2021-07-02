use std::collections::HashMap;

use fxhash::{FxBuildHasher, FxHasher, FxHashMap};
use image::{GenericImage, GenericImageView};
use rusttype::{GlyphId, Point, Rect};

use crate::draw::Dimension;
use crate::text::{FontFamily, FontId, FontSize, FontStyle, FontWeight};
use crate::widget::{Environment, GlobalState};

type ImageId = crate::image_map::Id;
type ImageData = image::DynamicImage;

const SHELVE_WIDTH: u32 = 512;

#[derive(Clone)]
pub enum QueueType {
    Image(ImageId, ImageData),
    /// FontId, GlyphId, width, height
    RasterGlyph(FontId, GlyphId, ImageData),
}

impl QueueType {
    fn id(&self) -> AtlasId {
        match self {
            QueueType::Image(id, _) => {
                AtlasId::Image(id.clone())
            }
            QueueType::RasterGlyph(font_id, glyph_id, image_data) => {
                AtlasId::RasterGlyph(font_id.clone(), glyph_id.clone(), image_data.height())
            }
        }
    }

    fn width(&self) -> u32 {
        match self {
            QueueType::Image(_, image) => {
                image.width()
            }
            QueueType::RasterGlyph(_, _, image) => {
                image.width()
            }
        }
    }

    fn height(&self) -> u32 {
        match self {
            QueueType::Image(_, image) => {
                image.height()
            }
            QueueType::RasterGlyph(_, _, image) => {
                image.height()
            }
        }
    }
}

#[derive(Hash, Eq, PartialEq)]
pub enum AtlasId {
    Image(ImageId),
    RasterGlyph(FontId, GlyphId, u32),
}

/// Inspired by the gpu_cache from rusttype
/// Another interesting source: https://nical.github.io/posts/etagere.html
pub struct TextureAtlas {
    /// The width and height of the atlas in pixels. The atlas should be a multiple of SHELVE_WIDTH
    width: u32,
    height: u32,

    /// The queue for new not yet added textures
    queue: Vec<QueueType>,

    shelves: Vec<Shelf>,

    all_books: FxHashMap<AtlasId, Book>,
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
pub struct Shelf {
    shelf_height: u32,
    shelf_y: u32,
    shelf_current_x: u32,
    books: Vec<Book>,
}

impl Shelf {
    fn append<F: FnMut(u32, u32, ImageData)>(&mut self, item: QueueType, uploader: &mut F) -> (AtlasId, Book) {
        let book = Book {
            x: self.shelf_current_x,
            y: self.shelf_y,
            width: item.width(),
            height: item.height(),
            is_active: true,
        };

        let book_id = item.id();

        match item {
            QueueType::Image(_, image_data) => {
                (*uploader)(self.shelf_current_x, self.shelf_y, image_data);
                self.shelf_current_x += book.width;
                self.books.push(book);
            }
            QueueType::RasterGlyph(font, glyph, image_data) => {
                (*uploader)(self.shelf_current_x, self.shelf_y, image_data);
                self.shelf_current_x += book.width;
                self.books.push(book);
            }
        }


        (book_id, book)
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
            queue: vec![],
            shelves: vec![],
            all_books: HashMap::with_hasher(FxBuildHasher::default()),
        }
    }

    pub fn queue_image(&mut self, image_id: ImageId, image_data: ImageData) {
        // Todo: only queue items that is not already in the atlas.
        self.queue.push(QueueType::Image(image_id, image_data))
    }

    pub fn queue_raster_glyph<GS: GlobalState>(&mut self, font_id: FontId, c: char, font_size: FontSize, env: &Environment<GS>) {
        let id = env.get_font(font_id).get_glyph_id(c);
        if let Some(id) = id {
            let image_data = env.get_font(font_id).get_glyph_raster_image(c, font_size).unwrap();
            self.queue.push(QueueType::RasterGlyph(font_id, id, image_data))
        }
    }

    pub fn queue_raster_glyph_id<GS: GlobalState>(&mut self, font_id: FontId, id: GlyphId, font_size: FontSize, env: &Environment<GS>) {
        let image_data = env.get_font(font_id).get_glyph_raster_image_from_id(id, font_size).unwrap();
        self.queue.push(QueueType::RasterGlyph(font_id, id, image_data))
    }

    pub fn get_tex_coords_for(&self, id: AtlasId) -> Rect<f32> {
        let book = self.all_books.get(&id);
        if let Some(book) = book {
            Rect {
                min: Point {
                    x: book.x as f32,
                    y: book.y as f32,
                },
                max: Point {
                    x: book.x as f32 + book.width as f32,
                    y: book.y as f32 + book.height as f32,
                },
            }
        } else {
            panic!("The atlas id was not cached.");
        }
    }

    /// The uploader should be x, y, image_data
    pub fn cache_queued<F: FnMut(u32, u32, ImageData)>(
        &mut self,
        mut uploader: F,
    ) {
        // Sort to get the smallest images first. Todo: what should the actual sorting be?
        self.queue.sort_unstable_by(|a, b| {
            let a_height = a.height();
            let b_height = b.height();
            a_height.cmp(&b_height)
        });

        while !self.queue.is_empty() {
            let item = self.queue.remove(0);
            let (book_id, book) = if let Some(shelf) = self.get_fitting_shelve(item.height(), item.width()) {
                self.shelves[shelf].append(item.clone(), &mut uploader)
            } else {
                let shelf_number = self.add_shelf(item.height());
                self.shelves[shelf_number as usize].append(item.clone(), &mut uploader)
            };

            self.all_books.insert(book_id, book);
        }
    }

    fn add_shelf(&mut self, height: u32) -> usize {
        let shelf_y = self.shelves.last().map(|s| s.shelf_y + s.shelf_height).unwrap_or(0);
        let new_shelf = Shelf {
            shelf_height: height,
            shelf_y,
            shelf_current_x: 0,
            books: vec![],
        };

        self.shelves.push(new_shelf);
        self.shelves.len() - 1
    }

    fn get_fitting_shelve(&self, height: u32, width: u32) -> Option<usize> {
        for (shelf_number, shelf) in self.shelves.iter().enumerate() {
            // We want to see it fit, but not if there is too much space in the shelf
            if height <= shelf.shelf_height && height > shelf.shelf_height / 2 {
                // If there is horizontal space enough to fit.
                if shelf.get_width_left() > width {
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
    let image1 = "/Users/holgergottchristensen/Documents/carbide/target/smile.png";
    let image2 = "/Users/holgergottchristensen/Documents/carbide/target/smile_new.png";

    let mut env = Environment::<String>::new(vec![], [0.0, 0.0], 1.0);
    let mut family = FontFamily::new("Apple Color Emoji");
    family.add_font("/System/Library/Fonts/Apple Color Emoji.ttc", FontWeight::Normal, FontStyle::Normal);
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
    atlas.queue_raster_glyph(0, '🥰', 32, &env);
    atlas.queue_raster_glyph(0, '🥶', 32, &env);
    atlas.queue_raster_glyph(0, '🐱', 32, &env);
    atlas.queue_raster_glyph(0, '🏆', 32, &env);
    atlas.queue_raster_glyph(0, '🎁', 32, &env);
    atlas.queue_raster_glyph(0, '🟥', 32, &env);
    atlas.queue_raster_glyph(0, '👬', 32, &env);
    atlas.queue_raster_glyph_id(0, id, 32, &env);


    let mut texture = image::DynamicImage::new_rgba8(512, 512);

    atlas.cache_queued(|x, y, image_data| {
        println!("Insert the image at: {}, {} with size {}, {}", x, y, image_data.width(), image_data.height());
        for (ix, iy, pixel) in image_data.pixels() {
            texture.put_pixel(x + ix, y + iy, pixel);
        }
    });

    texture.save("/Users/holgergottchristensen/Documents/carbide/target/smile_atlas.png").unwrap();
}