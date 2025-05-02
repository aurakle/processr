use std::path::Path;

use anyhow::bail;

use crate::Item;

use super::Procedure;

fn change_directory<'a>(new_dir: &'a Path) -> Procedure<'a> {
    Box::new(|item| {
        let file_name = match item.path.file_name() {
            Some(v) => v,
            None => bail!("Item has an invalid path"),
        };

        let mut new_path = new_dir.to_path_buf();
        new_path.push(file_name);

        Ok(Item {
            path: new_path.as_path().into(),
            bytes: item.bytes.clone(),
        })
    })
}

// fn change_extension<'a>(new_extension: &'a str) -> Procedure<'a> {
//     Box::new(|item| {
//         //TODO: why does new_extension get outlived?
//         let new_path = item.path.with_extension(new_extension);
//
//         Ok(Item {
//             path: new_path.as_path().into(),
//             bytes: item.bytes.clone(),
//         })
//     })
// }
