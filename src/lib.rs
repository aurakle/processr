use std::{collections::HashMap, path::Path};

mod procedure;

struct Item {
    pub path: Box<Path>,
    pub bytes: Vec<u8>,
    pub properties: HashMap<String, String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
    }
}
