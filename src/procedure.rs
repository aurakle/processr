use std::fs;
use std::{env, path::PathBuf};

use anyhow::{bail, Result};
use crate::parser::{template::TemplateParser, Parser};

use crate::{selector::Selector, Item, Meta};

pub trait Procedure: Sized {
    fn write(&self, out: &str) -> Result<()>;
}

pub trait SingleProcedure: Procedure + Sized + Clone {
    fn eval(&self) -> Result<Item>;

    fn property<S: Into<String>>(self, key: S, value: Meta) -> SetProperty<Self> {
        SetProperty {
            prior: self,
            key: key.into(),
            value,
        }
    }

    fn directory<S: Into<PathBuf>>(self, dir: S) -> SetDirectory<Self> {
        SetDirectory {
            prior: self,
            dir: dir.into(),
        }
    }

    fn extension<S: Into<String>>(self, extension: S) -> SetExtension<Self> {
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

    fn apply<S: Into<PathBuf>>(self, template: S) -> ApplyTemplate<Self> {
        ApplyTemplate {
            prior: self,
            template: template.into(),
        }
    }
}

pub trait MultiProcedure<P: SingleProcedure>: Procedure + Sized {
    fn chain<O, F>(&self, func: F) -> impl MultiProcedure<O>
    where
        O: SingleProcedure,
        F: Fn(P) -> O,
    ;

    fn into_meta(&self) -> Result<Meta>;
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
        F: Fn(P) -> O,
    {
        let mut result = Vec::new();

        for p in self {
            result.push(func(p.clone()));
        }

        result
    }

    fn into_meta(&self) -> Result<Meta> {
        let mut result = Vec::new();

        for p in self {
           result.push(p.eval()?.into_meta()?);
        }

        Ok(Meta::from(result))
    }
}

#[derive(Clone)]
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

#[derive(Clone)]
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

#[derive(Clone)]
pub struct SetExtension<P: SingleProcedure> {
    prior: P,
    extension: String,
}

impl<P: SingleProcedure> SingleProcedure for SetExtension<P> {
    fn eval(&self) -> Result<Item> {
        let item = self.prior.eval()?;
        let path = item.path.with_extension(self.extension.clone()).clone();

        Ok(Item {
            path,
            bytes: item.bytes.clone(),
            properties: item.properties.clone(),
        })
    }
}

#[derive(Clone)]
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

#[derive(Clone)]
pub struct ApplyTemplate<P: SingleProcedure> {
    prior: P,
    template: PathBuf,
}

impl<P: SingleProcedure> SingleProcedure for ApplyTemplate<P> {
    fn eval(&self) -> Result<Item> {
        let item = self.prior.eval()?;
        let properties = item.properties_with_url_and_body()?;
        let template = fs::read(self.template.clone())?;
        let (bytes, properties) = TemplateParser::default().process(&template, &properties)?;

        Ok(Item {
            path: item.path.clone(),
            bytes,
            properties,
        })
    }
}
