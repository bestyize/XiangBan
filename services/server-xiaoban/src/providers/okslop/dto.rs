#![allow(dead_code)]

//! Shapes of okslop's API, taken from the fields of its official client library (PyPI `okslop`, 0.0.1).
//! No live response has been captured yet, so every field the site might leave out is optional and
//! the mapper copes with gaps instead of dropping the photo.

use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct OkslopPage {
    #[serde(default)]
    pub results: Vec<OkslopPhoto>,
    #[serde(default)]
    pub total: Option<u64>,
    #[serde(default)]
    pub total_pages: Option<u64>,
}

#[derive(Debug, Deserialize)]
pub struct OkslopPhoto {
    pub id: String,
    pub width: u32,
    pub height: u32,
    #[serde(default)]
    pub color: Option<String>,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub alt_description: Option<String>,
    pub urls: OkslopUrls,
    #[serde(default)]
    pub links: Option<OkslopLinks>,
    #[serde(default)]
    pub likes: Option<u64>,
    #[serde(default)]
    pub user: Option<OkslopUser>,
    /// Layout graphics (posters, logos) set this; plain images leave it null.
    #[serde(default)]
    pub is_design: Option<bool>,
    #[serde(default)]
    pub tags: Vec<OkslopTag>,
}

/// A tag as a bare word or as an object; the live API has only returned empty lists so far.
#[derive(Debug, Deserialize)]
#[serde(untagged)]
pub enum OkslopTag {
    Word(String),
    Named {
        #[serde(default)]
        title: Option<String>,
        #[serde(default)]
        name: Option<String>,
    },
}

impl OkslopTag {
    pub fn text(&self) -> Option<String> {
        let raw = match self {
            OkslopTag::Word(w) => Some(w.as_str()),
            OkslopTag::Named { title, name } => title.as_deref().or(name.as_deref()),
        };
        raw.map(|t| t.trim().to_lowercase()).filter(|t| !t.is_empty())
    }
}

#[derive(Debug, Default, Deserialize)]
pub struct OkslopUrls {
    #[serde(default)]
    pub raw: Option<String>,
    #[serde(default)]
    pub full: Option<String>,
    #[serde(default)]
    pub regular: Option<String>,
    #[serde(default)]
    pub small: Option<String>,
    #[serde(default)]
    pub thumb: Option<String>,
}

#[derive(Debug, Default, Deserialize)]
pub struct OkslopLinks {
    #[serde(default)]
    pub html: Option<String>,
}

#[derive(Debug, Default, Deserialize)]
pub struct OkslopUser {
    #[serde(default)]
    pub username: String,
    #[serde(default)]
    pub name: String,
    #[serde(default)]
    pub portfolio_url: Option<String>,
    #[serde(default)]
    pub profile_image: Option<OkslopAvatar>,
}

#[derive(Debug, Default, Deserialize)]
pub struct OkslopAvatar {
    #[serde(default)]
    pub small: Option<String>,
    #[serde(default)]
    pub medium: Option<String>,
    #[serde(default)]
    pub large: Option<String>,
}
