use std::collections::HashMap;

use anyhow::Result;

use crate::Item;

use super::Procedure;

mod markdown;

pub trait Parser {
    fn process(&self, bytes: &Vec<u8>) -> Result<(Vec<u8>, HashMap<String, String>)>;
}

pub fn parse<'a>(parser: &'a (dyn Parser + 'a)) -> Procedure<'a> {
    Box::new(|item| {
        let (bytes, properties) = parser.process(&item.bytes)?;

        Ok(Item {
            path: item.path.clone(),
            bytes,
            properties,
        })
    })
}
