use std::{env, path::PathBuf};

use anyhow::{bail, Result};
use parser::Parser;

use crate::{selector::Selector, Item, Meta};

pub mod parser;
pub mod extractor;

pub trait Procedure: Sized {
    fn write(&self, out: &str) -> Result<()>;
}

pub trait SingleProcedure: Procedure + Sized {
    fn eval(&self) -> Result<Item>;

    fn property(self, key: String, value: Meta) -> SetProperty<Self> {
        SetProperty {
            prior: self,
            key,
            value,
        }
    }

    fn directory<S: Into<PathBuf>>(self, dir: S) -> SetDirectory<Self> {
        SetDirectory {
            prior: self,
            dir: dir.into(),
        }
    }

    fn extensions<S: Into<String>>(self, extension: S) -> SetExtension<Self> {
        SetExtension {
            prior: self,
            extension: extension.into(),
        }
    }

    fn parse<P: Parser>(self, parser: P) -> Parse<Self, P> {
        Parse {
            prior: self,
            parser: parser,
        }
    }
}

pub trait MultiProcedure<P: SingleProcedure>: Procedure + Sized {
    fn chain<O, F>(&self, func: F) -> impl MultiProcedure<O>
    where
        O: SingleProcedure,
        F: Fn(&P) -> O,
    ;
}

impl<P: SingleProcedure> Procedure for P {
    fn write(&self, out: &str) -> Result<()> {
        self.eval()?.write(out)
    }
}

impl<P: SingleProcedure> Procedure for Vec<P> {
    fn write(&self, out: &str) -> Result<()> {
        for p in self {
            p.write(out)?;
        }

        Ok(())
    }
}

impl<P: SingleProcedure> MultiProcedure<P> for Vec<P> {
    fn chain<O, F>(&self, func: F) -> impl MultiProcedure<O>
    where
        O: SingleProcedure,
        F: Fn(&P) -> O,
    {
        self.iter().map(func).collect::<Vec<_>>()
    }
}

pub struct SetProperty<P: SingleProcedure> {
    prior: P,
    key: String,
    value: Meta,
}

impl<P: SingleProcedure> SingleProcedure for SetProperty<P> {
    fn eval(&self) -> Result<Item> {
        Ok(self.prior.eval()?.set_property(self.key.clone(), self.value.clone()))
    }
}

pub struct SetDirectory<P: SingleProcedure> {
    prior: P,
    dir: PathBuf,
}

impl<P: SingleProcedure> SingleProcedure for SetDirectory<P> {
    fn eval(&self) -> Result<Item> {
        let item = self.prior.eval()?;

        let file_name = match item.path.file_name() {
            Some(v) => v,
            None => bail!("Item has an invalid path"),
        };

        let mut new_path = self.dir.clone();
        new_path = PathBuf::from(new_path.strip_prefix(env::current_dir()?).unwrap_or(&new_path));
        new_path.push(file_name);

        Ok(Item {
            path: new_path.as_path().into(),
            bytes: item.bytes.clone(),
            properties: item.properties.clone(),
        })
    }
}

pub struct SetExtension<P: SingleProcedure> {
    prior: P,
    extension: String,
}

impl<P: SingleProcedure> SingleProcedure for SetExtension<P> {
    fn eval(&self) -> Result<Item> {
        todo!()
    }
}

pub struct Parse<P: SingleProcedure, PARSER: Parser> {
    prior: P,
    parser: PARSER,
}

impl<P: SingleProcedure, PARSER: Parser> SingleProcedure for Parse<P, PARSER> {
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
