use std::fs;
use std::path::Path;
use std::{env, path::PathBuf};

use anyhow::{bail, Result};
use time::macros::format_description;
use time::{format_description, Date, Month};
use crate::parser::{template::TemplateParser, ParserProcedure};

use crate::{selector, FsError};
use crate::{selector::Selector, Item, Meta};

pub trait Procedure: Sized + Clone {
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

    fn parse<P: ParserProcedure>(self, parser: P) -> Parse<Self, P> {
        Parse {
            prior: self,
            parser: parser,
        }
    }

    fn apply<T: SingleProcedure>(self, template: T) -> ApplyTemplate<Self, T> {
        ApplyTemplate {
            prior: self,
            template,
        }
    }

    fn load_and_apply<S: Into<String>>(self, path: S) -> LoadAndApplyTemplate<Self> {
        LoadAndApplyTemplate {
            prior: self,
            path: path.into(),
        }
    }

    fn load_date(self) -> LoadDate<Self> {
        LoadDate {
            prior: self,
        }
    }

    fn map<F>(self, func: F) -> Map<Self, F>
    where
        F: Fn(Item) -> Result<Item> + Clone,
    {
        Map {
            prior: self,
            func,
        }
    }
}

pub trait MultiProcedure<P: SingleProcedure>: Procedure + Sized + Clone {
    fn chain<O, F>(self, func: F) -> impl MultiProcedure<O>
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
    fn chain<O, F>(self, func: F) -> impl MultiProcedure<O>
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
pub struct Parse<P: SingleProcedure, PARSER: ParserProcedure> {
    prior: P,
    parser: PARSER,
}

impl<P: SingleProcedure, PARSER: ParserProcedure> SingleProcedure for Parse<P, PARSER> {
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
pub struct ApplyTemplate<P: SingleProcedure, T: SingleProcedure> {
    prior: P,
    template: T,
}

impl<P: SingleProcedure, T: SingleProcedure> SingleProcedure for ApplyTemplate<P, T> {
    fn eval(&self) -> Result<Item> {
        let item = self.prior.eval()?;
        let template = self.template.eval()?;
        let bytes = template.bytes;
        let mut properties = template.properties.clone();
        properties.extend(item.properties_with_url_and_body()?);

        let (bytes, properties) = TemplateParser::default().process(&bytes, &properties)?;

        Ok(Item {
            path: item.path.clone(),
            bytes,
            properties,
        })
    }
}

#[derive(Clone)]
pub struct LoadAndApplyTemplate<P: SingleProcedure> {
    prior: P,
    path: String,
}

impl<P: SingleProcedure> SingleProcedure for LoadAndApplyTemplate<P> {
    fn eval(&self) -> Result<Item> {
        self.prior.clone().apply(selector::exact(&self.path)?).eval()
    }
}

#[derive(Clone)]
pub struct LoadDate<P: SingleProcedure> {
    prior: P,
}

impl<P: SingleProcedure> SingleProcedure for LoadDate<P> {
    fn eval(&self) -> Result<Item> {
        let item = self.prior.eval()?;
        let file_name = item.path
            .file_name()
            .map(|os_str| Path::new(os_str))
            .ok_or(FsError::InvalidFileName)?
            .to_str()
            .ok_or(FsError::OsStringNotUtf8)?
            .to_owned();

        let format = format_description!("[year]-[month]-[day]");
        let mut v = file_name.splitn(4, '-').take(3).collect::<Vec<_>>();
        let date = Date::parse(v.join("-").as_str(), format)?;

        Ok(item.set_property("date", format!("{}", date)))
    }
}

#[derive(Clone)]
pub struct Map<P, F>
where
    P: SingleProcedure,
    F: Fn(Item) -> Result<Item> + Clone,
{
    prior: P,
    func: F,
}

impl<P, F> SingleProcedure for Map<P, F>
where
    P: SingleProcedure,
    F: Fn(Item) -> Result<Item> + Clone,
{
    fn eval(&self) -> Result<Item> {
        (self.func)(self.prior.eval()?)
    }
}
