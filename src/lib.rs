use std::path::Path;

mod procedure;

struct Item {
    pub path: Box<Path>,
    pub bytes: Vec<u8>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
    }
}
