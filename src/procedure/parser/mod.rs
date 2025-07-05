use std::collections::HashMap;

use anyhow::Result;

use crate::{Item, Meta};

use super::SingleProcedure;

pub mod markdown;

pub trait Parser: Clone {
    fn process(&self, bytes: &Vec<u8>, properties: &HashMap<String, Meta>) -> Result<(Vec<u8>, HashMap<String, Meta>)>;
}
