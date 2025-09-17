//   light_king: pieces.data(Piece::LightKing).map(|data| {
//                     Image::new(ImageSource::Bytes {
//                         uri: "image://LightKing".into(),
//                         bytes: data.iter().map(|c| *c).collect::<Vec<_>>().into(),
//                     })
//                 }),

use std::{collections::HashMap, io::Cursor};

use egui::{Image, ImageSource, Pos2, Rect, Vec2, pos2};
use image::{DynamicImage, ImageReader};

fn image_data(image: &DynamicImage, pos: Pos2, size: Vec2) -> Vec<u8> {
    let mut bytes: Vec<u8> = Vec::new();
    let rect = Rect::from_min_size(pos, size);
    image
        .crop_imm(
            rect.min.x as u32,
            rect.min.y as u32,
            rect.width() as u32,
            rect.height() as u32,
        )
        .write_to(&mut Cursor::new(&mut bytes), image::ImageFormat::Png)
        .expect("failed to write image");
    bytes
}

fn make_source<'a>(name: &'a str, data: Vec<u8>) -> Image<'a> {
    Image::new(ImageSource::Bytes {
        uri: format!("image://{name}").into(),
        bytes: data.into(),
    })
}

fn piece_parameters(pw: f32, ph: f32) -> Vec<(&'static str, Pos2)> {
    vec![
        ("K", (pos2(0.0, ph))),
        ("Q", (pos2(1.0 * pw, ph))),
        ("R", (pos2(2.0 * pw, ph))),
        ("B", (pos2(3.0 * pw, ph))),
        ("N", (pos2(4.0 * pw, ph))),
        ("P", (pos2(5.0 * pw, ph))),
        ("k", (pos2(0.0 * pw, 0.0))),
        ("q", (pos2(1.0 * pw, 0.0))),
        ("r", (pos2(2.0 * pw, 0.0))),
        ("b", (pos2(3.0 * pw, 0.0))),
        ("n", (pos2(4.0 * pw, 0.0))),
        ("p", (pos2(5.0 * pw, 0.0))),
    ]
}

pub struct Sources<'a> {
    collection: HashMap<&'a str, Image<'a>>,
}

impl<'a> Sources<'a> {
    pub fn new() -> Self {
        let mut collection = HashMap::new();

        let chessmen_set = include_bytes!("../assets/pieces-leipzig.png");
        if let Ok(image) =
            ImageReader::with_format(Cursor::new(chessmen_set), image::ImageFormat::Png).decode()
        {
            let width = image.width();
            let height = image.height();

            let pw = width as f32 / 6.0;
            let ph = height as f32 / 2.0;
            let size = Vec2::new(pw, ph);

            for (name, pos) in piece_parameters(pw, ph) {
                let image_data = image_data(&image, pos, size);
                collection.insert(name, make_source(name, image_data));
            }
        }

        let bytes = include_bytes!("../assets/dark-square.png");
        collection.insert(
            "dark-square",
            Image::new(ImageSource::Bytes {
                uri: "image://dark-square".into(),
                bytes: bytes.into(),
            }),
        );

        Self { collection }
    }

    pub fn get<S>(&self, name: S) -> Option<&Image<'a>>
    where
        S: AsRef<str>,
    {
        self.collection.get(name.as_ref())
    }
}
