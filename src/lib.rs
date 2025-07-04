use std::{collections::HashMap, path::Path};

mod procedure;

#[derive(Debug, Clone)]
struct Item {
    pub path: Box<Path>,
    pub bytes: Vec<u8>,
    pub properties: HashMap<String, Meta>,
}

#[derive(Debug, Clone, PartialEq)]
enum Meta {
    List(Vec<Meta>),
    Text(String),
}
