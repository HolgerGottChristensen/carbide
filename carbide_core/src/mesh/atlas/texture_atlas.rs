use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::{Rc, Weak};

use fxhash::{FxBuildHasher, FxHashMap};
use image::GenericImageView;
use rusttype::{GlyphId, Point, Rect};

use crate::draw::Position;
use crate::draw::Scalar;
use crate::mesh::atlas::lossy_glyph_info::LossyGlyphInfo;
use crate::text::{Font, FontId, FontSize, Glyph};

type ImageId = crate::image_map::ImageId;
type ImageData = image::DynamicImage;
pub type TextureAtlasIndex = usize;

pub type AtlasEntry = Rc<RefCell<Book>>;

const SHELVE_WIDTH: u32 = 512;

#[derive(Copy, Clone, Hash, Eq, PartialEq, Debug)]
pub enum AtlasId {
    Image(ImageId),
    RasterGlyph(FontId, GlyphId, u32),
    LossyGlyph(LossyGlyphInfo),
}

impl AtlasId {
    fn new_lossy(font_id: FontId, glyph_id: GlyphId, font_size: FontSize, offset_over_tolerance: (u16, u16)) -> AtlasId {
        AtlasId::LossyGlyph(LossyGlyphInfo::new(font_id, glyph_id, font_size, offset_over_tolerance))
    }
}

/// Inspired by the gpu_cache from rusttype
/// Another interesting source: https://nical.github.io/posts/etagere.html
#[derive(Debug)]
pub struct TextureAtlas {
    /// The width and height of the atlas in pixels. The atlas should be a multiple of SHELVE_WIDTH
    width: u32,
    height: u32,

    not_yet_added_queue: Vec<(AtlasId, ImageData, AtlasEntry)>,

    /// An atlas is split up into a number of shelves. Each shelf can hold a number of images that
    /// are less than or equal to that space.
    shelves: Vec<Shelf>,

    all_books_cabinet: FxHashMap<AtlasId, (AtlasEntry, ImageData)>,
    position_tolerance: Scalar,
}

impl TextureAtlas {
    /// Create a new texture atlas of the size of the parameters in physical pixels.
    pub fn new(width: u32, height: u32) -> TextureAtlas {
        TextureAtlas {
            width,
            height,
            not_yet_added_queue: vec![],
            shelves: vec![],
            all_books_cabinet: HashMap::with_hasher(FxBuildHasher::default()),
            position_tolerance: 0.5,
        }
    }

    /// Returns the width of the atlas in physical pixels
    pub fn width(&self) -> u32 {
        self.width
    }

    /// Returns the height of the atlas in physical pixels
    pub fn height(&self) -> u32 {
        self.height
    }

    /// Queue the given glyph to from the given font to the atlas at the given scale factor.
    /// Returns an AtlasEntry with the corresponding information about where the glyph is located
    /// within the atlas.
    pub fn queue_glyph(&mut self, glyph: &Glyph, font: &Font, scale_factor: Scalar) -> Option<AtlasEntry> {
        if font.is_bitmap() {
            self.queue_raster_glyph_id(glyph.id(), glyph.font_size(), font, scale_factor)
        } else {
            self.queue_glyph_id(
                glyph.id(),
                glyph.font_size(),
                glyph.position(),
                font,
                scale_factor,
            )
        }
    }

    pub fn queue_glyph_id(
        &mut self,
        glyph_id: GlyphId,
        font_size: FontSize,
        position: Position,
        font: &Font,
        scale_factor: Scalar,
    ) -> Option<AtlasEntry> {
        let offset = (position.fraction_0_1() / (self.position_tolerance * scale_factor)).round_to_u16();

        let atlas_id = AtlasId::new_lossy(font.id(), glyph_id, font_size, offset);

        // Check if a suitable item has already been added.
        if let Some(item) = self.all_books_cabinet.get(&atlas_id) {
            return Some(item.0.clone())
        }

        // Get the image data for the given glyph from the font
        let image_data = font.get_glyph_image_from_id(
            glyph_id,
            font_size,
            scale_factor,
            position.fraction_0_1(),
        );


        if let Some(image_data) = image_data {
            if let Some(already_in_queue) = self.not_yet_added_queue.iter().find(|&a| &a.0 == &atlas_id) {
                Some(already_in_queue.2.clone())
            } else {
                // Generate a new empty book. This will be added to a shelf when cache_queued is called.
                let book = Book {
                    x: 0,
                    y: 0,
                    width: image_data.width(),
                    height: image_data.height(),
                    is_active: false,
                    tex_coords: Default::default()
                };

                self.not_yet_added_queue.push((atlas_id, image_data, Rc::new(RefCell::new(book))));
                Some(self.not_yet_added_queue[self.not_yet_added_queue.len() - 1].2.clone())
            }
        } else {
            None
        }
    }

    pub fn queue_image(&mut self, image_id: ImageId, image_data: ImageData) -> Option<AtlasEntry> {
        let atlas_id = AtlasId::Image(image_id);

        // Check if a suitable item has already been added.
        if let Some(item) = self.all_books_cabinet.get(&atlas_id) {
            return Some(item.0.clone())
        }

        if let Some(already_in_queue) = self.not_yet_added_queue.iter().find(|&a| &a.0 == &atlas_id) {
            Some(already_in_queue.2.clone())
        } else {
            // Generate a new empty book. This will be added to a shelf when cache_queued is called.
            let book = Book {
                x: 0,
                y: 0,
                width: image_data.width(),
                height: image_data.height(),
                is_active: false,
                tex_coords: Default::default()
            };

            self.not_yet_added_queue.push((atlas_id, image_data, Rc::new(RefCell::new(book))));
            Some(self.not_yet_added_queue[self.not_yet_added_queue.len() - 1].2.clone())
        }
    }
/*
    pub fn queue_raster_glyph(
        &mut self,
        c: char,
        font_size: FontSize,
        font: &Font,
        scale_factor: Scalar,
    ) -> TextureAtlasIndex {
        let id = font.get_glyph_id(c);
        self.queue_raster_glyph_id(id.unwrap(), font_size, font, scale_factor)
    }
*/
    pub fn queue_raster_glyph_id(
        &mut self,
        id: GlyphId,
        font_size: FontSize,
        font: &Font,
        scale_factor: Scalar,
    ) -> Option<AtlasEntry> {
        let atlas_id = AtlasId::RasterGlyph(font.id(), id, font_size);

        // Check if a suitable item has already been added.
        if let Some(item) = self.all_books_cabinet.get(&atlas_id) {
            return Some(item.0.clone())
        }

        // Get the image data for the given raster image glyph
        let image_data = font
            .get_glyph_raster_image_from_id(id, font_size, scale_factor);

        if let Some(image_data) = image_data {
            if let Some(already_in_queue) = self.not_yet_added_queue.iter().find(|&a| &a.0 == &atlas_id) {
                Some(already_in_queue.2.clone())
            } else {
                // Generate a new empty book. This will be added to a shelf when cache_queued is called.
                let book = Book {
                    x: 0,
                    y: 0,
                    width: image_data.width(),
                    height: image_data.height(),
                    is_active: false,
                    tex_coords: Default::default()
                };

                self.not_yet_added_queue.push((atlas_id, image_data, Rc::new(RefCell::new(book))));
                Some(self.not_yet_added_queue[self.not_yet_added_queue.len() - 1].2.clone())
            }
        } else {
            println!("The raster glyph could not be found in the font.");
            None
        }

    }
/*
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
    }*/

    /// The uploader should be x, y, image_data
    pub fn cache_queued<F: FnMut(u32, u32, &ImageData)>(&mut self, mut uploader: F) {
        if self.not_yet_added_queue.len() == 0 {return;}

        let mut queue = self.not_yet_added_queue.drain(..).collect::<Vec<_>>();

        queue.sort_unstable_by(|(_, data1, _), (_, data2, _)| {
            data2.height().cmp(&data1.height())
        });

        let size = (self.width, self.height);

        let all_queued = queue.drain(..)
            .fold(true, |state, (id, image_data, book)| {
                let shelf = self.get_or_new_fitting_shelve(image_data.width(), image_data.height());

                let res = if let Some(shelf) = shelf {
                    shelf.append(size, &image_data, &mut uploader, book.clone());
                    state
                } else {
                    false
                };

                self.all_books_cabinet.insert(id, (book, image_data));

                res
            });

        // If there was not space for all the queued images, try to clean up.
        if !all_queued {
            println!("Not space for all glyphs. Trying to cleanup atlas.");
            self.shelves = vec![];
            let mut queue = self.all_books_cabinet.drain().map(|(key, (entry, img))| {
                (key, img, entry)
            }).filter(|(_, _, e)| {
                Rc::strong_count(e) > 1
            }).collect::<Vec<_>>();

            queue.sort_unstable_by(|(_, data1, _), (_, data2, _)| {
                data2.height().cmp(&data1.height())
            });

            let all_queued = queue.drain(..)
                .fold(true, |state, (id, image_data, book)| {
                    let shelf = self.get_or_new_fitting_shelve(image_data.width(), image_data.height());

                    let res = if let Some(shelf) = shelf {
                        shelf.append(size, &image_data, &mut uploader, book.clone());
                        state
                    } else {
                        false
                    };

                    self.all_books_cabinet.insert(id, (book, image_data));

                    res
                });

            if !all_queued {
                println!("Tried to add more images to the atlas but there was not enough space even after cleanup.");
            }
        }
    }

    fn get_or_new_fitting_shelve(&mut self, width: u32, height: u32) -> Option<&mut Shelf> {
        for (shelf_number, shelf) in self.shelves.iter().enumerate() {
            // We want to see it fit, but not if there is too much space in the shelf
            if height <= shelf.shelf_height && height > shelf.shelf_height / 2 {
                // If there is horizontal space enough to fit.
                if shelf.get_width_left() > width {
                    return Some(&mut self.shelves[shelf_number]);
                }
            }
        }

        // Create a new shelf
        let shelf_y = self
            .shelves
            .last()
            .map(|s| s.shelf_y + s.shelf_height + 1)
            .unwrap_or(0);

        if shelf_y + height >= self.height {
            return None;
        }

        let shelf = Shelf {
            shelf_height: height,
            shelf_y,
            shelf_current_x: 0,
            books: vec![],
        };

        self.shelves.push(shelf);
        let new_shelf_number = self.shelves.len() - 1;
        Some(&mut self.shelves[new_shelf_number])
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

}

/// The book is an area of the shelf where a single image is stored.
#[derive(Copy, Clone, Debug)]
pub struct Book {
    pub x: u32,
    pub y: u32,
    pub width: u32,
    pub height: u32,
    pub is_active: bool,
    pub tex_coords: Rect<f32>
}

/// Each section of rows in the atlas is a shelf
#[derive(Debug)]
pub struct Shelf {
    /// The height of the shelf in pixels. This is used to determine if there is space for a glyph
    /// based on the height of the glyph.
    shelf_height: u32,
    /// The y position of the upper corner of this shelf.
    shelf_y: u32,
    /// The x position at the end of the shelf. This is where the the next glyph is added if there
    /// is space on the x direction.
    shelf_current_x: u32,
    /// The list of books in order from left to right in the atlas.
    books: Vec<Weak<RefCell<Book>>>,
}

impl Shelf {
    fn append<F: FnMut(u32, u32, &ImageData)>(
        &mut self,
        atlas_size: (u32, u32),
        image_data: &ImageData,
        uploader: &mut F,
        entry: AtlasEntry,
    ) {
        let mut book = entry.borrow_mut();
        book.x = self.shelf_current_x;
        book.y = self.shelf_y;

        book.tex_coords = Rect {
            min: Point {
                x: book.x as f32 / atlas_size.0 as f32,
                y: book.y as f32 / atlas_size.1 as f32,
            },
            max: Point {
                x: (book.x as f32 + book.width as f32) / atlas_size.0 as f32,
                y: (book.y as f32 + book.height as f32) / atlas_size.1 as f32,
            },
        };

        (*uploader)(self.shelf_current_x, self.shelf_y, image_data);
        book.is_active = true;
        self.shelf_current_x += book.width + 1;
        self.books.push(Rc::downgrade(&entry));
    }

    fn get_width_left(&self) -> u32 {
        SHELVE_WIDTH - self.shelf_current_x
    }
}

/*#[test]
fn create_packed_image() {
    use crate::draw::Dimension;
    use crate::environment::Environment;
    use crate::text::{FontFamily, FontStyle, FontWeight};
    use image::GenericImage;

    let mut atlas = TextureAtlas::new(512, 512);
    let image1 = "/Users/holgergottchristensen/carbide/target/smile.png";
    let image2 = "/Users/holgergottchristensen/carbide/target/smile_new.png";

    let mut env = Environment::new(vec![], Dimension::new(0.0, 0.0), 1.0);
    let mut family = FontFamily::new("Apple Color Emoji");
    family.add_font_with_hints(
        "/System/Library/Fonts/Apple Color Emoji.ttc",
        FontWeight::Normal,
        FontStyle::Normal,
    );
    env.add_font_family(family);

    let mut family = FontFamily::new("Noto Sans");
    family.add_font_with_hints(
        "fonts/NotoSans/NotoSans-Regular.ttf",
        FontWeight::Normal,
        FontStyle::Normal,
    );
    env.add_font_family(family);

    let _id = env.get_font(0).get_glyph_id('👴').unwrap();

    /*atlas.queue_image(0.into(), image::open(image1).unwrap());
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
    atlas.queue_image(1.into(), image::open(image2).unwrap());*/
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
        println!(
            "Insert the image at: {}, {} with size {}, {}",
            x,
            y,
            image_data.width(),
            image_data.height()
        );
        for (ix, iy, pixel) in image_data.pixels() {
            texture.put_pixel(x + ix, y + iy, pixel);
        }
    });

    texture
        .save("/Users/holgergottchristensen/carbide/target/smile_atlas.png")
        .unwrap();
}*/
