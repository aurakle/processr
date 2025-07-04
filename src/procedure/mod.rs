use std::path::PathBuf;

use anyhow::{bail, Result};
use parser::Parser;

use crate::{Item, Meta};

pub mod parser;
pub mod extractor;

pub trait Procedure {
    fn eval(&self) -> Result<Item>;

    fn property(self, key: String, value: Meta) -> SetProperty<Self>
    where
        Self: Sized
    {
        SetProperty {
            prior: self,
            key,
            value,
        }
    }

    fn directory<S: Into<PathBuf>>(self, dir: S) -> SetDirectory<Self>
    where
        Self: Sized
    {
        SetDirectory {
            prior: self,
            dir: dir.into(),
        }
    }

    fn parse<P: Parser>(self, parser: P) -> Parse<Self, P>
    where
        Self: Sized
    {
        Parse {
            prior: self,
            parser: parser,
        }
    }
}

impl Procedure for Item {
    fn eval(&self) -> Result<Item> {
        Ok(self.clone())
    }
}

pub struct SetProperty<P: Procedure> {
    prior: P,
    key: String,
    value: Meta,
}

impl<P: Procedure> Procedure for SetProperty<P> {
    fn eval(&self) -> Result<Item> {
        Ok(self.prior.eval()?.set_property(self.key.clone(), self.value.clone()))
    }
}

pub struct SetDirectory<P: Procedure> {
    prior: P,
    dir: PathBuf,
}

impl<P: Procedure> Procedure for SetDirectory<P> {
    fn eval(&self) -> Result<Item> {
        let item = self.prior.eval()?;

        let file_name = match item.path.file_name() {
            Some(v) => v,
            None => bail!("Item has an invalid path"),
        };

        let mut new_path = self.dir.clone();
        new_path.push(file_name);

        Ok(Item {
            path: new_path.as_path().into(),
            bytes: item.bytes.clone(),
            properties: item.properties.clone(),
        })
    }
}

pub struct Parse<P: Procedure, PARSER: Parser> {
    prior: P,
    parser: PARSER,
}

impl<P: Procedure, PARSER: Parser> Procedure for Parse<P, PARSER> {
    fn eval(&self) -> Result<Item> {
        let item = self.prior.eval()?;
        let (bytes, properties) = self.parser.process(&item.bytes, &item.properties)?;

        Ok(Item {
            path: item.path.clone(),
            bytes,
            properties,
        })
    }
}
