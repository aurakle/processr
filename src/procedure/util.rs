use std::path::Path;

use anyhow::bail;

use crate::{Item, Meta};

use super::Procedure;

// fn change_extension<'a>(new_extension: &'a str) -> Procedure<'a> {
//     Box::new(|item| {
//         //TODO: why does new_extension get outlived?
//         let new_path = item.path.with_extension(new_extension);
//
//         Ok(Item {
//             path: new_path.as_path().into(),
//             bytes: item.bytes.clone(),
//             properties: item.properties.clone(),
//         })
//     })
// }
