use std::collections::HashMap;
use std::sync::Arc;

use image;
use vello::peniko::{Blob, Format, Image};

#[derive(Default)]
pub struct ImageCache {
    images: HashMap<String, Image>,
}

impl ImageCache {
    pub fn from_file(&mut self, path: &String) -> anyhow::Result<Image> {
        if let Some(image) = self.images.get(path) {
            Ok(image.clone())
        } else {
            let data = std::fs::read(path)?;
            let image = decode_image(&data)?;
            self.images.insert(path.clone(), image.clone());
            Ok(image)
        }
    }
}

fn decode_image(data: &[u8]) -> anyhow::Result<Image> {
    let image = image::io::Reader::new(std::io::Cursor::new(data))
        .with_guessed_format()?
        .decode()?;
    let width = image.width();
    let height = image.height();
    let data = Arc::new(image.into_rgba8().into_vec());
    let blob = Blob::new(data);
    Ok(Image::new(blob, Format::Rgba8, width, height))
}

