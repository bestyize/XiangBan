//! 首批随版本维护的精选目录。网页与原生端共用一份源文件，不在浏览请求中访问 GitHub。
use axum::{extract::State, Json};
use serde_json::Value;
use std::sync::OnceLock;

pub fn catalog() -> &'static Value {
    static CATALOG: OnceLock<Value> = OnceLock::new();
    CATALOG.get_or_init(|| {
        serde_json::from_str(include_str!("../../editorial/open-source.json"))
            .expect("valid open-source editorial catalog")
    })
}

#[cfg(test)]
pub fn is_curated(name: &str) -> bool {
    catalog()["projects"].as_array().is_some_and(|items| {
        items.iter().any(|item| {
            item["full_name"]
                .as_str()
                .is_some_and(|full_name| full_name.eq_ignore_ascii_case(name))
        })
    })
}

pub async fn list(
    State(state): State<crate::state::AppState>,
) -> crate::error::AppResult<Json<Value>> {
    Ok(Json(
        super::open_source_workflow::public_catalog(&state.pool).await?,
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;

    #[test]
    fn editorial_catalog_references_are_complete() {
        let data = catalog();
        let categories: HashSet<_> = data["categories"]
            .as_array()
            .unwrap()
            .iter()
            .map(|item| item["key"].as_str().unwrap())
            .collect();
        let projects = data["projects"].as_array().unwrap();
        let names: HashSet<_> = projects
            .iter()
            .map(|item| item["full_name"].as_str().unwrap())
            .collect();
        assert_eq!(names.len(), projects.len(), "duplicate project");
        for project in projects {
            assert!(categories.contains(project["category"].as_str().unwrap()));
            assert!(crate::collectors::repo_sources::valid_name(
                project["full_name"].as_str().unwrap()
            ));
            for category in project["related_categories"].as_array().unwrap() {
                assert!(categories.contains(category.as_str().unwrap()));
            }
            assert!(project["source_url"]
                .as_str()
                .unwrap()
                .starts_with("https://github.com/"));
        }
        for collection in data["collections"].as_array().unwrap() {
            let members = collection["projects"].as_array().unwrap();
            assert_eq!(members.len(), collection["notes"].as_array().unwrap().len());
            for name in members {
                assert!(names.contains(name.as_str().unwrap()));
            }
        }
    }

    #[test]
    fn curated_access_is_case_insensitive_and_does_not_admit_unknown_repos() {
        assert!(is_curated("Excalidraw/Excalidraw"));
        assert!(!is_curated("unknown/private-repo"));
        assert!(!is_curated("excalidraw/excalidraw/other"));
    }
}
