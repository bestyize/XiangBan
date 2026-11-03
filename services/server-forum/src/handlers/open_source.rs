//! 首批随版本维护的精选目录。网页与原生端共用一份源文件，不在浏览请求中访问 GitHub。
use axum::{
    extract::{Path, Query, State},
    Json,
};
use serde::Deserialize;
use serde_json::{json, Value};
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

const PAGE_SIZE: usize = 20;

#[derive(Deserialize)]
pub struct PageQuery {
    category: Option<String>,
    tag: Option<String>,
    q: Option<String>,
    page: Option<usize>,
}

fn entries<'a>(data: &'a Value, field: &str) -> &'a [Value] {
    data[field]
        .as_array()
        .map(Vec::as_slice)
        .unwrap_or_default()
}

fn project_card(project: &Value) -> Value {
    const FIELDS: &[&str] = &[
        "full_name",
        "name",
        "title",
        "summary",
        "category",
        "related_categories",
        "tags",
        "cover_url",
        "icon_url",
        "cover_style",
        "cover_layout",
        "cover_accent",
        "cover_lines",
        "cover_template",
        "cover_variant",
        "cover_focus_x",
        "cover_focus_y",
        "icon_fit",
    ];
    Value::Object(
        FIELDS
            .iter()
            .filter_map(|key| {
                project
                    .get(key)
                    .map(|value| (key.to_string(), value.clone()))
            })
            .collect(),
    )
}

fn belongs(project: &Value, category: &str) -> bool {
    match category {
        "all" => true,
        "recommend" => project["featured"].as_bool().unwrap_or(false),
        _ => {
            project["category"] == category
                || entries(project, "related_categories")
                    .iter()
                    .any(|value| value == category)
        }
    }
}

fn metadata(data: &Value) -> Value {
    json!({
        "categories": data["categories"],
        "reviewed_at": data["reviewed_at"],
        "notice": data["notice"],
        "revision": data["revision"].as_i64().unwrap_or_default(),
    })
}

fn page(data: &Value, query: PageQuery) -> Value {
    let category = query.category.as_deref().unwrap_or("recommend");
    let tag = query.tag.as_deref().unwrap_or_default();
    let search = query.q.as_deref().unwrap_or_default().trim().to_lowercase();
    let number = query.page.unwrap_or(1).max(1);
    let mut projects: Vec<_> = entries(data, "projects")
        .iter()
        .filter(|project| belongs(project, category))
        .collect();
    projects.sort_by_key(|project| project["order"].as_i64().unwrap_or_default());
    let mut tags = Vec::new();
    for value in projects.iter().flat_map(|project| entries(project, "tags")) {
        if !tags.contains(&value) && tags.len() < 8 {
            tags.push(value);
        }
    }
    projects.retain(|project| {
        (tag.is_empty() || entries(project, "tags").iter().any(|value| value == tag))
            && (search.is_empty()
                || ["name", "full_name", "title", "summary"]
                    .iter()
                    .filter_map(|key| project[*key].as_str())
                    .chain(entries(project, "tags").iter().filter_map(Value::as_str))
                    .any(|text| text.to_lowercase().contains(&search)))
    });
    let offset = (number - 1).saturating_mul(PAGE_SIZE);
    let mut result = metadata(data);
    result["projects"] = json!(projects
        .iter()
        .skip(offset)
        .take(PAGE_SIZE)
        .map(|project| project_card(project))
        .collect::<Vec<_>>());
    result["total"] = json!(projects.len());
    result["next_page"] = if offset.saturating_add(PAGE_SIZE) < projects.len() {
        json!(number + 1)
    } else {
        Value::Null
    };
    result["tags"] = json!(tags);
    result["collections"] = if category == "recommend" && tag.is_empty() && number == 1 {
        json!(entries(data, "collections").iter().take(2).map(|collection| json!({
            "key": collection["key"], "title": collection["title"], "summary": collection["summary"],
            "projects": collection["projects"],
            "project_names": entries(collection, "projects").iter().filter_map(|name| {
                entries(data, "projects").iter().find(|project| project["full_name"] == *name)
                    .and_then(|project| project["name"].as_str())
            }).collect::<Vec<_>>(),
        })).collect::<Vec<_>>())
    } else {
        json!([])
    };
    result
}

pub async fn projects(
    State(state): State<crate::state::AppState>,
    Query(query): Query<PageQuery>,
) -> crate::error::AppResult<Json<Value>> {
    let data = super::open_source_workflow::public_catalog(&state.pool).await?;
    Ok(Json(page(&data, query)))
}

pub async fn project(
    State(state): State<crate::state::AppState>,
    Path((owner, repo)): Path<(String, String)>,
) -> crate::error::AppResult<Json<Value>> {
    let data = super::open_source_workflow::public_catalog(&state.pool).await?;
    let name = format!("{owner}/{repo}");
    let projects = entries(&data, "projects");
    let project = projects
        .iter()
        .find(|item| {
            item["full_name"]
                .as_str()
                .is_some_and(|value| value.eq_ignore_ascii_case(&name))
        })
        .or_else(|| crate::software_sync::directory_project(&name))
        .ok_or_else(|| crate::error::AppError::not_found("精选项目"))?;
    let mut result = metadata(&data);
    result["projects"] = json!([project]);
    result["related"] = json!(projects
        .iter()
        .filter(|item| *item != project
            && (project["category"]
                .as_str()
                .is_some_and(|category| belongs(item, category))
                || entries(project, "related_categories").contains(&item["category"])))
        .take(2)
        .map(project_card)
        .collect::<Vec<_>>());
    Ok(Json(result))
}

pub async fn collection(
    State(state): State<crate::state::AppState>,
    Path(key): Path<String>,
) -> crate::error::AppResult<Json<Value>> {
    let data = super::open_source_workflow::public_catalog(&state.pool).await?;
    let collection = entries(&data, "collections")
        .iter()
        .find(|item| item["key"] == key)
        .ok_or_else(|| crate::error::AppError::not_found("精选专题"))?;
    let mut result = metadata(&data);
    result["collections"] = json!([collection]);
    result["projects"] = json!(entries(collection, "projects")
        .iter()
        .filter_map(|name| {
            entries(&data, "projects")
                .iter()
                .find(|item| item["full_name"] == *name)
                .map(project_card)
        })
        .collect::<Vec<_>>());
    Ok(Json(result))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;

    #[test]
    fn pages_filter_before_slicing_and_exclude_detail_fields() {
        let projects: Vec<_> = (0..45)
            .rev()
            .map(|i| {
                json!({
                    "full_name": format!("owner/project-{i}"),
                    "category": "office",
                    "related_categories": ["design"],
                    "tags": [if i < 22 { "匹配" } else { "其他" }],
                    "order": i,
                    "highlights": ["完整导读"],
                })
            })
            .collect();
        let data = json!({ "projects": projects, "revision": 7 });
        let query = |number| PageQuery {
            category: Some("design".into()),
            tag: Some("匹配".into()),
            q: None,
            page: Some(number),
        };
        let first = page(&data, query(1));
        let second = page(&data, query(2));
        assert_eq!(first["total"], 22);
        assert_eq!(first["next_page"], 2);
        assert_eq!(entries(&first, "projects").len(), 20);
        assert_eq!(entries(&second, "projects").len(), 2);
        assert_eq!(second["projects"][0]["full_name"], "owner/project-20");
        assert_eq!(first["tags"], json!(["匹配", "其他"]));
        assert_eq!(first["revision"], 7);
        assert!(first["projects"][0].get("highlights").is_none());
        assert!(second["next_page"].is_null());
    }

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
