use std::io::Cursor;

use anyhow::Result;
use async_trait::async_trait;
use image::{ImageFormat, ImageReader};

use crate::data::{Item, State};

use super::ParserProcedure;

#[derive(Clone)]
pub struct ImageConverter {
    format: ImageFormat,
}

impl ImageConverter {
    fn new(format: ImageFormat) -> Self {
        Self { format }
    }
}

#[async_trait(?Send)]
impl ParserProcedure for ImageConverter {
    fn default() -> Self {
        Self::new(ImageFormat::WebP)
    }

    async fn process(&self, state: &mut State, item: &Item) -> Result<Item> {
        let img = ImageReader::new(Cursor::new(item.bytes.clone())).with_guessed_format()?.decode()?;
        let mut bytes = Vec::new();
        img.write_to(&mut Cursor::new(&mut bytes), self.format.clone())?;

        Ok(Item {
            bytes,
            ..item.clone()
        })
    }
}
