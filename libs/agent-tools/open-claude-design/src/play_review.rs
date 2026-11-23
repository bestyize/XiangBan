//! Human Play checks are stored separately from model-editable canvas metadata.
use super::{file_store, nonce, Project, Store, STORAGE_LOCK};
use serde::{Deserialize, Serialize};
use std::{collections::BTreeSet, fs, io::ErrorKind};

pub fn play_steps(project: &Project) -> Vec<&str> {
    let mut steps = Vec::new();
    if let Some(items) = project.canvas["xiaoban"]["playChecks"].as_array() {
        for step in items.iter().filter_map(|item| item.as_str()).map(str::trim) {
            if !step.is_empty() && !steps.contains(&step) {
                steps.push(step);
            }
        }
    }
    steps
}

#[derive(Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PlayReview {
    pub revision: u64,
    pub passed: BTreeSet<usize>,
}

impl Store {
    pub fn play_review(&self, project: &Project) -> Result<PlayReview, String> {
        let path = self.dir(&project.id)?.join("play-review.json");
        file_store::regular(&path)?;
        match fs::read(path) {
            Ok(bytes) => {
                let review: PlayReview =
                    serde_json::from_slice(&bytes).map_err(|e| e.to_string())?;
                if review.revision == project.revision {
                    return Ok(review);
                }
            }
            Err(e) if e.kind() == ErrorKind::NotFound => {}
            Err(e) => return Err(e.to_string()),
        }
        Ok(PlayReview {
            revision: project.revision,
            ..Default::default()
        })
    }

    pub fn mark_play_step(
        &self,
        id: &str,
        revision: u64,
        step: usize,
        passed: bool,
    ) -> Result<PlayReview, String> {
        let _lock = STORAGE_LOCK.lock().map_err(|_| "设计存储锁不可用")?;
        let project = self.load(id)?;
        if project.revision != revision {
            return Err("设计稿已更新，请在当前版本重新验收".into());
        }
        if step >= play_steps(&project).len() {
            return Err("验收项已变化，请重新打开画布".into());
        }
        let mut review = self.play_review(&project)?;
        if passed {
            review.passed.insert(step);
        } else {
            review.passed.remove(&step);
        }
        let dir = self.dir(id)?;
        let temp = dir.join(format!("review-{}.tmp", nonce()?));
        fs::write(
            &temp,
            serde_json::to_vec(&review).map_err(|e| e.to_string())?,
        )
        .map_err(|e| e.to_string())?;
        fs::rename(temp, dir.join("play-review.json")).map_err(|e| e.to_string())?;
        Ok(review)
    }
}
