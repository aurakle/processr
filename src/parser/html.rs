use std::{collections::HashMap, path::{Path, PathBuf}};

use async_trait::async_trait;
use dom_query::{Document, Selection};
use mime_guess::get_extensions;
use pathdiff::diff_paths;
use anyhow::Result;
use reqwest::Client;

use crate::{data::{Item, State}, error::FsError};

use super::ParserProcedure;

#[derive(Clone)]
pub struct HtmlParser {
    relativize_urls: bool,
    cache_linked_resources: bool,
}

impl HtmlParser {
    pub fn relativize_urls(self) -> Self {
        Self {
            relativize_urls: true,
            ..self
        }
    }

    pub fn cache_linked_resources(self) -> Self {
        Self {
            cache_linked_resources: true,
            ..self
        }
    }

    async fn apply<'a>(&self, state: &mut State, http_client: &Client, item: &mut Item, attr: &str, target: Selection<'a>) -> Result<()> {
        if let Some(link) = target.attr(attr) {
            let link = link.to_string();
            let new_link = {
                if link.starts_with("http://") || link.starts_with("https://") {
                    if self.cache_linked_resources && target.filter("*:not(a):not(link)").exists() {
                        let file = match state.cached_sources.get(&link) {
                            Some(p) => p.clone(),
                            None => {
                                println!("Caching resource at {}", link.clone());

                                let response = http_client.get(link.clone()).send().await?;
                                let status = response.status();

                                println!("Received HTTP status {}{}", status.as_u16(), status.canonical_reason().map(|s| format!(": {}", s)).unwrap_or(String::new()));

                                if status.is_success() {
                                    let extension = response
                                        .headers()
                                        .get("Content-Type")
                                        .and_then(|h| h.to_str().ok())
                                        .map(String::from)
                                        .and_then(|m| {
                                            let (left, right) = m.split_once("/")?;
                                            get_extensions(left, right)
                                        })
                                        .and_then(|exts| exts.to_vec().first().map(|ext| ext.to_owned().to_owned()))
                                        .or_else(|| link
                                            .rsplit_once("/")
                                            .and_then(|(left, right)| right.to_owned()
                                                .rsplit_once(".")
                                                .map(|(left, right)| right.to_owned())));
                                    let bytes = response.bytes().await?;

                                    item.insert_into_cache(state, link, bytes.to_vec(), extension)?
                                } else {
                                    println!("Caching failed");
                                    println!("Falling back to external link");
                                    state.cached_sources.insert(link.clone(), link.clone());

                                    link
                                }
                            },
                        };

                        if self.relativize_urls {
                            Self::relativize(item, PathBuf::from(file.clone()))?.unwrap_or(file)
                        } else {
                            file
                        }
                    } else {
                        link
                    }
                } else {
                    let path = PathBuf::from(link.clone());

                    if self.relativize_urls && path.is_absolute() {
                        Self::relativize(item, path)?.unwrap_or(link)
                    } else {
                        link
                    }
                }
            };

            target.set_attr(attr, &new_link);
        }

        Ok(())
    }

    fn relativize(item: &Item, path: PathBuf) -> Result<Option<String>> {
        if let Some(relative_path) = item.path.parent()
            .map(|p| PathBuf::from("/")
                .join(p))
            .and_then(|current_dir| {
                println!("Making {} relative to {}", path.display(), current_dir.display());
                diff_paths(path, current_dir)
            })
        {
            Ok(Some(format!(
                "./{}",
                relative_path
                    .as_os_str()
                    .to_str()
                    .ok_or(FsError::OsStringNotUtf8)?,
            )))
        } else {
            Ok(None)
        }
    }
}

#[async_trait(?Send)]
impl ParserProcedure for HtmlParser {
    fn default() -> Self {
        Self {
            relativize_urls: false,
            cache_linked_resources: false,
        }
    }

    async fn process(&self, state: &mut State, item: &Item) -> Result<Item> {
        let mut item = item.clone();
        let client = reqwest::Client::builder()
            .user_agent(concat!(env!("CARGO_PKG_NAME"), "/", env!("CARGO_PKG_VERSION")))
            .build()?;
        let mut document = Document::from(String::from_utf8(item.bytes.clone())?);
        document.normalize();

        let href_targets = document.select("*[href]:not([href=\"\"])").iter();

        for target in href_targets {
            self.apply(state, &client, &mut item, "href", target).await?;
        }

        let src_targets = document.select("*[src]:not([src=\"\"])").iter();

        for target in src_targets {
            self.apply(state, &client, &mut item, "src", target).await?;
        }

        document.normalize();
        item.bytes = document.html().to_string().as_bytes().to_vec();
        Ok(item)
    }
}

#[cfg(test)]
mod tests {
    use std::{collections::HashMap, path::PathBuf};

    use crate::{data::{Item, State}, parser::ParserProcedure};

    use super::HtmlParser;

    #[actix_web::test]
    async fn relativize() {
        let p = HtmlParser::default().relativize_urls();
        let res = p.process(&mut State::new("dist").unwrap(), &Item {
            path: PathBuf::from("/posts/thing1.html"),
            bytes: b"<html lang=\"en\"><head><link rel=\"stylesheet\" href=\"/css/default.css\"></head><body><a href=\"/another/file.html\">Some link</a><img src=\"/images/profile.png\"></body></html>".to_vec(),
            properties: HashMap::new(),
            cache: HashMap::new(),
        }).await.unwrap().bytes;
        let res = String::from_utf8(res).unwrap();
        let expected = format!("<html lang=\"en\"><head><link rel=\"stylesheet\" href=\"./../css/default.css\"></head><body><a href=\"./../another/file.html\">Some link</a><img src=\"./../images/profile.png\"></body></html>");

        assert_eq!(expected, res);
    }

    #[actix_web::test]
    async fn relativize_with_unapplied_caching() {
        let p = HtmlParser::default().relativize_urls().cache_linked_resources();
        let res = p.process(&mut State::new("dist").unwrap(), &Item {
            path: PathBuf::from("/posts/thing1.html"),
            bytes: b"<html lang=\"en\"><head><link rel=\"stylesheet\" href=\"/css/default.css\"></head><body><a href=\"/another/file.html\">Some link</a><img src=\"/images/profile.png\"></body></html>".to_vec(),
            properties: HashMap::new(),
            cache: HashMap::new(),
        }).await.unwrap().bytes;
        let res = String::from_utf8(res).unwrap();
        let expected = format!("<html lang=\"en\"><head><link rel=\"stylesheet\" href=\"./../css/default.css\"></head><body><a href=\"./../another/file.html\">Some link</a><img src=\"./../images/profile.png\"></body></html>");

        assert_eq!(expected, res);
    }
}
