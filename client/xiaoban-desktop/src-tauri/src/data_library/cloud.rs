//! Read-only cloud browser. Credentials and transport stay in the existing MCP manager.
use super::*;
use agent_kernel::{Tool, ToolCtx};
use std::sync::Arc;

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Reference {
    pub provider: String,
    pub id: String,
    pub path: String,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Entry {
    pub id: String,
    pub name: String,
    pub url: String,
    pub path: String,
    pub directory: bool,
    #[serde(default)]
    pub location: String,
}

fn provider(value: &str) -> Result<(&'static str, &'static str), String> {
    match value {
        "tencent" => Ok(("tencent-docs", "腾讯文档")),
        "baidu" => Ok(("baidu-netdisk", "百度网盘")),
        _ => Err("此平台暂不支持账号浏览".into()),
    }
}
fn allowed(provider: &str, search: bool) -> &'static [&'static str] {
    match (provider, search) {
        ("tencent", false) => &["query_space_node"],
        ("tencent", true) => &["manage.search_file", "search_space_file"],
        ("baidu", false) => &["file_list"],
        ("baidu", true) => &["file_keyword_search"],
        _ => &[],
    }
}
fn matches_tool(name: &str, allowed: &[&str]) -> bool {
    allowed
        .iter()
        .any(|suffix| name == *suffix || name.ends_with(&format!("_{suffix}")))
}
fn source<'a>(items: &'a Value, platform: &str) -> Option<&'a Value> {
    let (plugin, _) = provider(platform).ok()?;
    items
        .as_array()?
        .iter()
        .find(|item| item["plugin"] == plugin)
}
fn sources(items: &Value) -> Value {
    json!(["tencent", "baidu"].map(|platform| {
        let item = source(items, platform); let connected = item.is_some_and(|item| item["status"] == "connected");
        let tools = item.and_then(|item| item["tools"].as_array());
        let supports = |search| tools.is_some_and(|tools| tools.iter().any(|tool| tool["name"].as_str().is_some_and(|name| matches_tool(name, allowed(platform, search)))));
        json!({"provider":platform,"name":provider(platform).unwrap().1,"connectorId":item.and_then(|item| item["id"].as_str()),
            "status":item.map(|item| item["status"].clone()).unwrap_or(json!("uninstalled")),"connected":connected,
            "browse":connected && supports(false),"search":connected && supports(true)})
    }))
}

/// Fill only known read parameters; never accept a tool name or raw arguments from a webview.
fn arguments(schema: &Value, search: &str, folder: &str, page: u32) -> Result<Value, String> {
    let Some(properties) = schema["properties"].as_object() else {
        return Ok(json!({}));
    };
    let mut args = serde_json::Map::new();
    let context: Value = serde_json::from_str(folder).unwrap_or(Value::Null);
    for (key, field) in properties {
        let normalized = key.replace('_', "").to_ascii_lowercase();
        let value = match normalized.as_str() {
            "dir" | "path" => Some(json!(if folder.is_empty() { "/" } else { folder })),
            "spaceid" if context.is_object() => context.get("spaceId").cloned(),
            "parentid" if context.is_object() => context.get("parentId").cloned(),
            "parentid" | "nodeid" | "folderid" | "spaceid" => {
                if folder.is_empty() {
                    field.get("default").cloned()
                } else {
                    Some(json!(folder))
                }
            }
            "key" | "keyword" | "query" | "searchkey" | "searchkeyword" => {
                if search.is_empty() {
                    None
                } else {
                    Some(json!(search))
                }
            }
            "page" | "pagenum" => Some(json!(page)),
            "offset" | "start" | "from" => Some(json!((page - 1) * 20)),
            "num" | "limit" | "pagesize" | "size" => Some(json!(20)),
            _ => field.get("default").cloned(),
        };
        if let Some(mut value) = value {
            if field["type"] == "string" && value.is_number() {
                value = json!(value.to_string());
            }
            args.insert(key.clone(), value);
        }
    }
    for key in schema["required"]
        .as_array()
        .into_iter()
        .flatten()
        .filter_map(Value::as_str)
    {
        if !args.contains_key(key) {
            return Err("服务参数格式已更新，请更新连接器后重试".into());
        }
    }
    Ok(Value::Object(args))
}
fn provider_arguments(
    platform: &str,
    schema: &Value,
    search: &str,
    folder: &str,
    page: u32,
) -> Result<Value, String> {
    let mut args = arguments(schema, search, folder, page)?;
    if platform == "tencent" && schema["properties"].get("num").is_some() {
        args["num"] = json!(page - 1);
    }
    if platform == "baidu" {
        if let Some(path) = args.get_mut("dir") {
            if let Some(raw) = path.as_str() {
                *path =
                    json!(url::form_urlencoded::byte_serialize(raw.as_bytes()).collect::<String>());
            }
        }
    }
    Ok(args)
}
fn text<'a>(value: &'a Value, keys: &[&str]) -> Option<&'a str> {
    keys.iter()
        .find_map(|key| value[*key].as_str().filter(|s| !s.is_empty()))
}
fn identifier(value: &Value, keys: &[&str]) -> String {
    keys.iter()
        .find_map(|key| {
            value[*key]
                .as_str()
                .filter(|s| !s.is_empty())
                .map(String::from)
                .or_else(|| value[*key].as_u64().map(|n| n.to_string()))
        })
        .unwrap_or_default()
}
fn array(value: &Value) -> Option<&Vec<Value>> {
    if let Some(array) = value.as_array() {
        return Some(array);
    }
    for key in [
        "list",
        "items",
        "nodes",
        "files",
        "children",
        "spaces",
        "space_list",
        "spaceList",
        "node_list",
        "nodeList",
        "file_list",
        "fileList",
        "data",
        "result",
    ] {
        if let Some(result) = value.get(key).and_then(array) {
            return Some(result);
        }
    }
    None
}
fn page(platform: &str, value: &Value, location: &str, spaces: bool) -> Result<Value, String> {
    let list = array(value).ok_or("服务返回了无法识别的文件列表，请更新连接器后重试")?;
    let mut entries = Vec::new();
    for row in list.iter() {
        let item = row.get("file").or_else(|| row.get("node")).unwrap_or(row);
        let id = identifier(
            item,
            &[
                "file_id", "fileId", "fs_id", "fsid", "node_id", "nodeId", "space_id", "spaceId",
                "id",
            ],
        );
        let name = text(
            item,
            &[
                "title",
                "filename",
                "server_filename",
                "name",
                "file_name",
                "space_name",
            ],
        )
        .unwrap_or("");
        let path = text(item, &["path", "file_path"]).unwrap_or("");
        let kind = text(item, &["node_type", "nodeType", "type", "file_type"]).unwrap_or("");
        let directory = spaces || item["isdir"] == 1 || item["isdir"] == true || item["is_dir"] == true || item["is_dir"] == 1 || item["isFolder"] == true || matches!(kind, "folder" | "wiki_folder" | "directory")
            // Baidu MCP omits isdir. Its directory rows have no digest or bytes.
            || platform == "baidu" && item["category"] == 6 && item["size"].as_u64() == Some(0) && item["md5"].as_str() == Some("");
        if id.is_empty()
            || name.is_empty()
            || id.len() > 512
            || name.chars().count() > 240
            || path.len() > 2048
        {
            continue;
        }
        let url = if platform == "baidu" {
            let encoded = url::form_urlencoded::Serializer::new(String::new())
                .append_pair("path", path)
                .append_pair("fileid", &id)
                .finish();
            format!("https://pan.baidu.com/disk/main#/index?{encoded}")
        } else {
            let supplied = text(item, &["url", "file_url", "fileUrl", "doc_url", "docUrl"]);
            if let Some(url) = supplied.filter(|url| valid_url(platform, url)) {
                url.to_string()
            } else {
                format!(
                    "https://docs.qq.com/doc/{}",
                    url::form_urlencoded::byte_serialize(id.as_bytes()).collect::<String>()
                )
            }
        };
        let context: Value = serde_json::from_str(location).unwrap_or(Value::Null);
        let destination = if platform == "tencent" {
            json!({"spaceId":if spaces {json!(id)} else {context["spaceId"].clone()},"parentId":if spaces {""} else {&id}}).to_string()
        } else {
            path.to_string()
        };
        entries.push(Entry {
            id,
            name: name.to_string(),
            url,
            path: path.to_string(),
            directory,
            location: destination,
        });
    }
    if !list.is_empty() && entries.is_empty() {
        return Err("云端返回了未识别的文件格式，请更新客户端后重试".into());
    }
    let next = value["next"]
        .as_u64()
        .or_else(|| value["data"]["next"].as_u64())
        .filter(|n| *n <= 10_000_000);
    let more = value["has_more"]
        .as_bool()
        .or_else(|| value["hasMore"].as_bool())
        .or_else(|| value["has_next"].as_bool())
        .or_else(|| value["is_over"].as_bool().map(|over| !over))
        .or_else(|| value["data"]["has_more"].as_bool())
        .or_else(|| value["data"]["has_next"].as_bool())
        .unwrap_or(list.len() >= 20);
    Ok(json!({"items":entries,"hasMore":more,"nextCursor":next}))
}
fn valid_url(platform: &str, value: &str) -> bool {
    url::Url::parse(value).is_ok_and(|url| {
        url.scheme() == "https"
            && url.username().is_empty()
            && url.password().is_none()
            && url.port_or_known_default() == Some(443)
            && url.host_str()
                == Some(if platform == "tencent" {
                    "docs.qq.com"
                } else {
                    "pan.baidu.com"
                })
            && !url.query_pairs().any(|(key, _)| {
                matches!(
                    key.to_ascii_lowercase().as_str(),
                    "access_token" | "refresh_token" | "authorization" | "token" | "client_secret"
                )
            })
    })
}

#[tauri::command]
pub async fn library_cloud_sources(
    app: tauri::AppHandle,
    window: tauri::WebviewWindow,
) -> Result<Value, String> {
    control_only(&window)?;
    let items = crate::chat::connectors::ai_connector_installed(app, window).await?;
    Ok(sources(&items))
}
#[tauri::command]
pub async fn library_cloud_browse(
    app: tauri::AppHandle,
    window: tauri::WebviewWindow,
    platform: String,
    query: String,
    location: String,
    page_number: u32,
    confirmed: Option<bool>,
    cursor: Option<u32>,
) -> Result<Value, String> {
    control_only(&window)?;
    provider(&platform)?;
    if query.chars().count() > 80
        || location.len() > 2048
        || location.chars().any(char::is_control)
        || !(1..=100).contains(&page_number)
        || cursor.is_some_and(|n| n > 10_000_000)
    {
        return Err("查询范围无效".into());
    }
    let items = crate::chat::connectors::ai_connector_installed(app.clone(), window).await?;
    let item = source(&items, &platform)
        .filter(|item| item["status"] == "connected")
        .ok_or("请先连接此平台的 MCP 账号")?;
    let id = item["id"].as_str().ok_or("连接器编号无效")?.to_string();
    tauri::async_runtime::spawn_blocking(move || {
        let manager = crate::chat::connector_manager(&app)?;
        let spaces = platform == "tencent" && location.is_empty() && query.trim().is_empty();
        let names: &[&str] = if spaces {
            &["query_space_list", "space_list"]
        } else {
            allowed(&platform, !query.trim().is_empty())
        };
        let tools = names
            .iter()
            .find_map(|name| {
                let tools = manager.tools_named_for(&id, &[*name]);
                if tools.is_empty() {
                    None
                } else {
                    Some(tools)
                }
            })
            .unwrap_or_default();
        let tool: &Arc<dyn Tool> = tools.first().ok_or("当前连接器未提供此只读浏览功能")?;
        if tool.spec().confirmation_reason.is_some() && confirmed != Some(true) {
            return Ok(json!({"items":[],"hasMore":false,"confirmationRequired":true}));
        }
        let mut args = provider_arguments(
            &platform,
            &tool.spec().schema,
            query.trim(),
            location.trim(),
            page_number,
        )?;
        if args.get("from").is_some() {
            if let Some(cursor) = cursor {
                args["from"] = json!(cursor);
            }
        }
        let result = tool.execute_result(args, &ToolCtx::detached("library-cloud-browse"));
        if result.is_error {
            let error = result.output.to_ascii_lowercase();
            return Err(if error.contains("timeout")
                || error.contains("timed out")
                || error.contains("超时")
            {
                "云端读取超时，请稍后重试"
            } else if error.contains("400007") {
                "腾讯文档账号的 MCP 权限不足，请检查服务套餐权限"
            } else {
                "读取云端列表失败，请检查 MCP 连接、账号权限或网络后重试"
            }
            .into());
        }
        if result.output.len() > 1024 * 1024
            || result
                .structured_content
                .as_ref()
                .is_some_and(|v| v.to_string().len() > 1024 * 1024)
        {
            return Err("文件列表过大，请缩小搜索范围".into());
        }
        let value: Value = match result.structured_content {
            Some(value) => value,
            None => serde_json::from_str(&result.output).map_err(|_| "服务未返回结构化文件列表")?,
        };
        page(&platform, &value, &location, spaces)
    })
    .await
    .map_err(|e| e.to_string())?
}
#[tauri::command]
pub fn library_cloud_add(
    app: tauri::AppHandle,
    window: tauri::WebviewWindow,
    platform: String,
    entry: Entry,
    folder: Option<String>,
) -> Result<Library, String> {
    control_only(&window)?;
    provider(&platform)?;
    if entry.directory
        || entry.id.is_empty()
        || entry.id.len() > 512
        || entry.path.len() > 2048
        || entry.name.trim().is_empty()
        || entry.name.chars().count() > 240
        || !valid_url(&platform, &entry.url)
    {
        return Err("云文档引用无效，请重新选择文件".into());
    }
    change(&app, |library, _| {
        folder_exists(library, &folder)?;
        if library.links.iter().any(|link| {
            link.url == entry.url
                || link.remote.as_ref().is_some_and(|reference| {
                    reference.provider == platform && reference.id == entry.id
                })
        }) {
            return Err("这份云文档已经在资料库里了".into());
        }
        library.links.insert(
            0,
            Link {
                id: new_id(&app),
                url: entry.url,
                title: entry.name.chars().take(120).collect(),
                platform: platform.clone(),
                kind: "云端文件".into(),
                code: None,
                folder,
                added_ms: now_ms(),
                remote: Some(Reference {
                    provider: platform,
                    id: entry.id,
                    path: entry.path,
                }),
            },
        );
        Ok(())
    })
    .map(|(_, library)| library)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn only_named_read_operations_are_available_and_no_credentials_are_exposed() {
        assert!(matches_tool(
            "mcp_ext_docs_query_space_node",
            allowed("tencent", false)
        ));
        assert!(!matches_tool(
            "mcp_ext_docs_delete_space_node",
            allowed("tencent", false)
        ));
        assert!(!matches_tool(
            "mcp_ext_disk_file_sharelink_set",
            allowed("baidu", true)
        ));
        let result = sources(
            &json!([{"plugin":"tencent-docs","id":"docs","status":"connected","headers":{"Authorization":"private-token"},"tools":[{"name":"query_space_node"}]}]),
        );
        assert_eq!(result[0]["browse"], true);
        assert_eq!(result[0]["search"], false);
        assert!(!result.to_string().contains("private-token"));
    }
    #[test]
    fn browse_and_search_use_the_advertised_read_schema_and_bounded_pages() {
        let schema = json!({"properties":{"dir":{"type":"string"},"key":{"type":"string"},"page":{"type":"integer"},"num":{"type":"integer"}},"required":["dir"]});
        assert_eq!(
            arguments(&schema, "合同", "/工作", 2).unwrap(),
            json!({"dir":"/工作","key":"合同","page":2,"num":20})
        );
        assert_eq!(
            provider_arguments("baidu", &schema, "合同", "/工作", 2).unwrap(),
            json!({"dir":"%2F%E5%B7%A5%E4%BD%9C","key":"合同","page":2,"num":20})
        );
        assert_eq!(
            provider_arguments(
                "tencent",
                &json!({"properties":{"num":{"type":"integer"}}}),
                "",
                "",
                2
            )
            .unwrap(),
            json!({"num":1})
        );
        assert!(arguments(
            &json!({"properties":{"space_id":{"type":"string"}},"required":["space_id"]}),
            "",
            "",
            1
        )
        .is_err());
    }
    #[test]
    fn provider_list_shapes_preserve_directory_identity_and_reject_foreign_urls() {
        let result=page("baidu",&json!({"list":[{"fs_id":123,"server_filename":"工作","isdir":1,"path":"/工作"},{"fsid":456,"filename":"合同.pdf","path":"/工作/合同.pdf"}]}),"",false).unwrap();
        assert_eq!(result["items"][0]["directory"], true);
        assert_eq!(result["items"][1]["id"], "456");
        assert!(result["items"][1]["url"]
            .as_str()
            .unwrap()
            .starts_with("https://pan.baidu.com/"));
        let result=page("baidu",&json!({"data":{"list":[{"fsid":123,"filename":"工作","category":6,"size":0,"md5":"","path":"/工作"},{"fsid":456,"filename":"空文件","category":6,"size":0,"md5":"d41d8cd98f00b204e9800998ecf8427e","path":"/空文件"}]}}),"",false).unwrap();
        assert_eq!(result["items"][0]["directory"], true);
        assert_eq!(result["items"][1]["directory"], false);
        let result=page("tencent",&json!({"data":{"nodes":[{"file_id":"doc-id","title":"周报","url":"https://docs.qq.com.evil.example/doc-id"}]}}),"",false).unwrap();
        assert_eq!(result["items"][0]["url"], "https://docs.qq.com/doc/doc-id");
        assert!(!valid_url(
            "tencent",
            "https://user:pass@docs.qq.com/doc/id"
        ));
        let spaces = page(
            "tencent",
            &json!({"data":{"spaces":[{"space_id":"space","name":"工作"}]}}),
            "",
            true,
        )
        .unwrap();
        assert_eq!(spaces["items"][0]["directory"], true);
        let location = spaces["items"][0]["location"].as_str().unwrap();
        let schema = json!({"properties":{"space_id":{"type":"string"},"parent_id":{"type":"string"},"num":{"type":"integer"}},"required":["space_id"]});
        assert_eq!(
            arguments(&schema, "", location, 1).unwrap(),
            json!({"space_id":"space","parent_id":"","num":20})
        );
        let result=page("tencent",&json!({"has_next":false,"children":[{"node_id":"folder","node_type":"wiki_folder","title":"工作"},{"node_id":"doc","node_type":"wiki_tdoc","title":"周报","url":"https://docs.qq.com/aio/doc"}]}),location,false).unwrap();
        assert_eq!(result["hasMore"], false);
        assert_eq!(result["items"][0]["directory"], true);
        assert_eq!(
            serde_json::from_str::<Value>(result["items"][0]["location"].as_str().unwrap())
                .unwrap(),
            json!({"spaceId":"space","parentId":"folder"})
        );
        let result=page("tencent",&json!({"is_over":false,"next":43,"list":[{"file_id":"doc","title":"周报","url":"https://docs.qq.com/doc/doc"}]}),"",false).unwrap();
        assert_eq!(result["hasMore"], true);
        assert_eq!(result["nextCursor"], 43);
    }
    #[test]
    fn existing_saved_links_remain_readable_without_cloud_metadata() {
        let link: Link=serde_json::from_value(json!({"id":"old","url":"https://docs.qq.com/doc/id","title":"周报","platform":"tencent"})).unwrap();
        assert!(link.remote.is_none());
    }
}
