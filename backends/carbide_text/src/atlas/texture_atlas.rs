use std::cell::RefCell;
use std::rc::Rc;
use cosmic_text::{CacheKey};

use fxhash::{FxHashMap};

use carbide_core::draw::image::ImageId;
use carbide_core::draw::{Dimension, Position, Rect, Scalar};
use carbide_core::image;

use carbide_core::image::{DynamicImage, GenericImage, GenericImageView};

pub type TextureAtlasIndex = usize;

pub type AtlasEntry = Rc<RefCell<Book>>;

const SHELVE_WIDTH: u32 = 512;

#[derive(Clone, Hash, Eq, PartialEq, Debug)]
pub enum AtlasId {
    Image(ImageId),
    Glyph(CacheKey),
}

/// Inspired by the gpu_cache from rusttype
/// Another interesting source: https://nical.github.io/posts/etagere.html
#[derive(Debug)]
pub struct TextureAtlas {
    /// The atlas image cache, that stores all the glyphs
    atlas: DynamicImage,

    /// A list of things to be queued
    queue: FxHashMap<AtlasId, (DynamicImage, i32, i32)>,

    /// An atlas is split up into a number of shelves. Each shelf can hold a number of images that
    /// are less than or equal to that space.
    shelves: Vec<Shelf>,

    /// A map of all the books currently stored in the atlas.
    books: FxHashMap<AtlasId, Book>,

    requires_render: bool,
}

impl TextureAtlas {
    /// Create a new texture atlas of the size of the parameters in physical pixels.
    pub fn new(width: u32, height: u32) -> TextureAtlas {
        TextureAtlas {
            atlas: DynamicImage::new_rgba8(width, height),
            queue: FxHashMap::default(),
            shelves: vec![],
            books: FxHashMap::default(),
            requires_render: false,
        }
    }

    /// Returns the width of the atlas in physical pixels
    pub fn width(&self) -> u32 {
        self.atlas.width()
    }

    /// Returns the height of the atlas in physical pixels
    pub fn height(&self) -> u32 {
        self.atlas.height()
    }

    pub fn book(&self, key: &AtlasId) -> Option<&Book> {
        self.books.get(key)
    }

    pub fn enqueue<F: FnOnce()->Option<(DynamicImage, i32, i32)>>(&mut self, id: AtlasId, f: F) {
        if !self.books.contains_key(&id) && !self.queue.contains_key(&id) {
            let image = f();

            if let Some(image) = image {
                self.queue.insert(id, image);
            }
        }
    }

    pub fn process_queued(&mut self) {
        // If no elements are in the queue, we can return early
        if self.queue.len() == 0 {
            return;
        }

        self.requires_render = true;

        // Retrieve all elements in a queue
        let mut queue = self.queue.drain().collect::<Vec<_>>();

        // Sort by height, to allow for better caching density
        queue.sort_unstable_by(|(_, (data1, _, _)), (_, (data2, _, _))| data2.height().cmp(&data1.height()));

        'queue: for (key, (image, top, left)) in queue {

            for shelf in &mut self.shelves {
                if image.height() <= shelf.shelf_height && image.height() > shelf.shelf_height / 2 {
                    // If there is horizontal space enough to fit.
                    if shelf.available_width() > image.width() {
                        let book = shelf.append(image, top, left, &mut self.atlas);
                        self.books.insert(key, book);
                        continue 'queue;
                    }
                }
            }

            // No fitting shelf available. Try to create a new one:
            let mut shelf = Shelf {
                shelf_height: image.height(),
                shelf_y: self.shelves.last().map(|s| s.shelf_y + s.shelf_height + 1).unwrap_or_default(),
                shelf_current_x: 0,
            };


            let book = shelf.append(image, top, left, &mut self.atlas);
            self.books.insert(key, book);
            self.shelves.push(shelf);
        }
    }

    /// The uploader should be x, y, image_data
    pub fn update_cache(&mut self, f: &mut dyn FnMut(&DynamicImage)) {
        if self.requires_render {

            //self.atlas.save("target/atlas.png").unwrap();

            f(&self.atlas)
        }

        self.requires_render = false;
    }
}

/// The book is an area of the shelf where a single image is stored.
#[derive(Copy, Clone, Debug)]
pub struct Book {
    pub width: u32,
    pub height: u32,
    pub top: i32,
    pub left: i32,
    pub tex_coords: Rect,
    pub has_color: bool,
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
}

impl Shelf {
    fn append(&mut self, image: DynamicImage, top: i32, left: i32, atlas: &mut DynamicImage) -> Book {
        let offset_x = self.shelf_current_x;
        let offset_y = self.shelf_y;

        let book = Book {
            width: image.width(),
            height: image.height(),
            top,
            left,
            tex_coords: Rect::new(
                Position::new(
                    offset_x as Scalar / atlas.width() as Scalar,
                    offset_y as Scalar / atlas.height() as Scalar,
                ),
                Dimension::new(
                    image.width() as Scalar / atlas.width() as Scalar,
                    image.height() as Scalar / atlas.height() as Scalar,
                )
            ),
            has_color: !matches!(image, DynamicImage::ImageLuma8(_)),
        };

        // Add image width to shelf and add 1 to make sure we have a pixel spacing.
        self.shelf_current_x += image.width() + 1;

        match image {
            DynamicImage::ImageLuma8(image) => {
                for (ix, iy, pixel) in image.enumerate_pixels() {
                    atlas.put_pixel(offset_x + ix, offset_y + iy, image::Rgba([0, 0, 0, pixel.0[0]]));
                }
            }
            image => {
                for (ix, iy, pixel) in image.pixels() {
                    atlas.put_pixel(offset_x + ix, offset_y + iy, pixel);
                }
            }
        }

        book
    }

    fn available_width(&self) -> u32 {
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
