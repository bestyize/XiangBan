use serde::{Deserialize, Serialize};
use crate::ai_reasoning::ReplyPreference;

/// A conversation owns its selection, while credentials and endpoints stay in the provider registry.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SessionModel {
    pub provider_id: String,
    pub model: String,
    #[serde(default)]
    pub reply_preference: ReplyPreference,
    #[serde(default)]
    pub reasoning_efforts: Option<Vec<String>>,
}
