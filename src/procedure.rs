use std::path::Path;
use std::{env, path::PathBuf};

use anyhow::{bail, Context, Result};
use async_trait::async_trait;
use time::macros::format_description;
use time::{format_description, Date};
use crate::data::{State, Value};
use crate::error::FsError;
use crate::parser::{template::TemplateParser, ParserProcedure};

use crate::selector;
use crate::Item;

#[async_trait(?Send)]
pub trait Procedure: Sized + Clone {
    async fn write(&self, state: &mut State) -> Result<()>;
}

#[async_trait(?Send)]
pub trait SingleProcedure: Procedure + Sized + Clone {
    async fn eval(&self, state: &mut State) -> Result<Item>;

    fn property<S: Into<String>>(self, key: S, value: Value) -> SetProperty<Self> {
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

    /// https://time-rs.github.io/book/api/format-description.html
    fn load_date(self, format: &'static [time::format_description::BorrowedFormatItem<'static>]) -> LoadDate<Self> {
        LoadDate {
            prior: self,
            format
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

#[async_trait(?Send)]
pub trait MultiProcedure<P: SingleProcedure>: Procedure + Sized + Clone {
    fn chain<O, F>(self, func: F) -> impl MultiProcedure<O>
    where
        O: SingleProcedure,
        F: Fn(P) -> O,
    ;

    async fn into_meta(&self, state: &mut State) -> Result<Value>;
}

#[async_trait(?Send)]
impl<P: SingleProcedure> Procedure for P {
    async fn write(&self, state: &mut State) -> Result<()> {
        self.eval(state).await?.write(&state.root)
    }
}

#[async_trait(?Send)]
impl<P: SingleProcedure> Procedure for Vec<P> {
    async fn write(&self, state: &mut State) -> Result<()> {
        for p in self {
            p.write(state).await?;
        }

        Ok(())
    }
}

#[async_trait(?Send)]
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

    async fn into_meta(&self, state: &mut State) -> Result<Value> {
        let mut result = Vec::new();

        for p in self {
           result.push(p.eval(state).await?.into_meta()?);
        }

        Ok(Value::from(result))
    }
}

#[derive(Clone)]
pub struct SetProperty<P: SingleProcedure> {
    prior: P,
    key: String,
    value: Value,
}

#[async_trait(?Send)]
impl<P: SingleProcedure> SingleProcedure for SetProperty<P> {
    async fn eval(&self, state: &mut State) -> Result<Item> {
        Ok(self.prior.eval(state).await?.set_property(self.key.clone(), self.value.clone()))
    }
}

#[derive(Clone)]
pub struct SetDirectory<P: SingleProcedure> {
    prior: P,
    dir: PathBuf,
}

#[async_trait(?Send)]
impl<P: SingleProcedure> SingleProcedure for SetDirectory<P> {
    async fn eval(&self, state: &mut State) -> Result<Item> {
        let item = self.prior.eval(state).await?;

        let file_name = match item.path.file_name() {
            Some(v) => v,
            None => bail!("Item has an invalid path"),
        };

        let mut new_path = self.dir.clone();
        new_path = PathBuf::from(new_path.strip_prefix(env::current_dir()?).unwrap_or(&new_path));
        new_path.push(file_name);

        Ok(Item {
            path: new_path.as_path().into(),
            ..item.clone()
        })
    }
}

#[derive(Clone)]
pub struct SetExtension<P: SingleProcedure> {
    prior: P,
    extension: String,
}

#[async_trait(?Send)]
impl<P: SingleProcedure> SingleProcedure for SetExtension<P> {
    async fn eval(&self, state: &mut State) -> Result<Item> {
        let item = self.prior.eval(state).await?;
        let path = item.path.with_extension(self.extension.clone()).clone();

        Ok(Item {
            path,
            ..item.clone()
        })
    }
}

#[derive(Clone)]
pub struct Parse<P: SingleProcedure, PARSER: ParserProcedure> {
    prior: P,
    parser: PARSER,
}

#[async_trait(?Send)]
impl<P: SingleProcedure, PARSER: ParserProcedure> SingleProcedure for Parse<P, PARSER> {
    async fn eval(&self, state: &mut State) -> Result<Item> {
        let item = self.prior.eval(state).await?;

        self.parser
            .process(state, &item)
            .await
            .context(format!("While parsing {}", item.path.display()))
    }
}

#[derive(Clone)]
pub struct ApplyTemplate<P: SingleProcedure, T: SingleProcedure> {
    prior: P,
    template: T,
}

#[async_trait(?Send)]
impl<P: SingleProcedure, T: SingleProcedure> SingleProcedure for ApplyTemplate<P, T> {
    async fn eval(&self, state: &mut State) -> Result<Item> {
        let item = self.prior.eval(state).await?;
        let template = self.template.eval(state).await?;
        let mut properties = template.properties.clone();
        properties.extend(item.properties_with_url_and_body()?);
        let mut cache = template.cache.clone();
        cache.extend(item.cache.clone());

        TemplateParser::default()
            .process(state, &Item {
                path: item.path.clone(),
                bytes: template.bytes.clone(),
                properties,
                cache
            })
            .await
            .context(format!("While applying template {}", template.path.display()))
    }
}

#[derive(Clone)]
pub struct LoadAndApplyTemplate<P: SingleProcedure> {
    prior: P,
    path: String,
}

#[async_trait(?Send)]
impl<P: SingleProcedure> SingleProcedure for LoadAndApplyTemplate<P> {
    async fn eval(&self, state: &mut State) -> Result<Item> {
        self.prior.clone().apply(selector::exact(&self.path).context(format!("While loading template {}", self.path))?).eval(state).await
    }
}

#[derive(Clone)]
pub struct LoadDate<P: SingleProcedure> {
    prior: P,
    format: &'static [time::format_description::BorrowedFormatItem<'static>],
}

#[async_trait(?Send)]
impl<P: SingleProcedure> SingleProcedure for LoadDate<P> {
    async fn eval(&self, state: &mut State) -> Result<Item> {
        let item = self.prior.eval(state).await?;
        let file_name = item.get_filename()?;

        let parse_format = format_description!("[year]-[month]-[day]");
        let v = file_name.splitn(4, '-').take(3).collect::<Vec<_>>();
        let date_raw = v.join("-");
        let date = Date::parse(date_raw.as_str(), parse_format)?;

        let mut properties = item.properties.clone();
        properties.insert("dateRaw".to_owned(), Value::from(date_raw));
        properties.insert("date".to_owned(), Value::from(date.format(self.format)?));
        properties.insert("dateYear".to_owned(), Value::from(format!("{}", v[0])));
        properties.insert("dateMonth".to_owned(), Value::from(format!("{}", v[1])));
        properties.insert("dateDay".to_owned(), Value::from(format!("{}", v[2])));

        Ok(Item {
            properties,
            ..item.clone()
        })
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

#[async_trait(?Send)]
impl<P, F> SingleProcedure for Map<P, F>
where
    P: SingleProcedure,
    F: Fn(Item) -> Result<Item> + Clone,
{
    async fn eval(&self, state: &mut State) -> Result<Item> {
        (self.func)(self.prior.eval(state).await?)
    }
}
