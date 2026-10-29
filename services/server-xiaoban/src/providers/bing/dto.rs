use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct BingResponse {
    #[serde(default)]
    pub images: Vec<BingImage>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct BingImage {
    pub startdate: Option<String>,
    pub fullstartdate: Option<String>,
    pub enddate: Option<String>,
    pub url: String,
    pub urlbase: Option<String>,
    pub copyright: Option<String>,
    pub copyrightlink: Option<String>,
    pub title: Option<String>,
    pub hsh: Option<String>,
    #[serde(default)]
    pub wp: Option<bool>,
}
