use std::io::Cursor;

use anyhow::Result;
use async_trait::async_trait;
use image::{ImageFormat, ImageReader};

use crate::data::{Item, State};

use super::ParserProcedure;

#[derive(Clone)]
pub struct ToWebpConverter();

#[async_trait(?Send)]
impl ParserProcedure for ToWebpConverter {
    fn default() -> Self {
        Self()
    }

    async fn process(&self, state: &mut State, item: &Item) -> Result<Item> {
        let img = ImageReader::new(Cursor::new(item.bytes.clone())).with_guessed_format()?.decode()?;
        let mut bytes = Vec::new();
        img.write_to(&mut Cursor::new(&mut bytes), ImageFormat::WebP)?;

        Ok(Item {
            bytes,
            ..item.clone()
        })
    }
}
