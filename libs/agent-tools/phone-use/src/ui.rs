//! Exact semantic selectors against a fresh Android UI tree. Ambiguity fails closed.
use crate::package::Result;
use quick_xml::{events::{BytesStart,Event},Reader};
use serde::{Deserialize,Serialize};
use std::collections::BTreeMap;

#[derive(Debug,Default,Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Selector {
    pub text:Option<String>,
    pub description:Option<String>,
    pub resource_id:Option<String>,
    pub class:Option<String>,
}
#[derive(Debug,Serialize)]
pub struct Target {
    pub text:String,
    pub description:String,
    pub resource_id:String,
    pub bounds:[u32;4],
    pub x:u32,
    pub y:u32,
}
struct Node {attributes:BTreeMap<String,String>,parent:Option<usize>}

fn attributes(start:&BytesStart<'_>)->Result<BTreeMap<String,String>> {
    start.attributes().map(|item| {
        let item=item.map_err(|e|e.to_string())?;
        let key=item.key.as_ref().to_string();
        let value=item.normalized_value(quick_xml::XmlVersion::Explicit1_0).map_err(|e|e.to_string())?.into_owned();
        Ok((key,value))
    }).collect()
}

pub fn target(xml:&str,selector:&Selector)->Result<Target> {
    let filters=[("text",&selector.text),("content-desc",&selector.description),("resource-id",&selector.resource_id),("class",&selector.class)];
    if filters.iter().all(|(_,value)|value.is_none()) || filters.iter().any(|(_,value)|value.as_ref().is_some_and(|s|s.is_empty()||s.len()>512)) {
        return Err("provide at least one nonempty exact element selector (at most 512 bytes per field)".into());
    }
    let mut reader=Reader::from_str(xml);let mut nodes:Vec<Node>=Vec::new();let mut parents=Vec::new();
    loop {
        match reader.read_event().map_err(|e|e.to_string())? {
            Event::Start(start) if start.name().as_ref()=="node" => {
                nodes.push(Node {attributes:attributes(&start)?,parent:parents.last().copied()});parents.push(nodes.len()-1);
            },
            Event::Empty(start) if start.name().as_ref()=="node" => {
                nodes.push(Node {attributes:attributes(&start)?,parent:parents.last().copied()});
            },
            Event::End(end) if end.name().as_ref()=="node" => {parents.pop();},
            Event::DocType(_)=>return Err("UI document types are not supported".into()),
            Event::Eof=>break,
            _=>{},
        }
        if nodes.len()>10000 {return Err("UI node budget exceeded".into());}
    }
    if !parents.is_empty() {return Err("incomplete UI tree".into());}
    let matches=nodes.iter().enumerate().filter(|(_,node)| {
        node.attributes.get("enabled").is_none_or(|s|s=="true")
            && node.attributes.get("visible-to-user").is_none_or(|s|s=="true")
            && filters.iter().all(|(key,value)|value.as_ref().is_none_or(|value|node.attributes.get(*key)==Some(value)))
    }).map(|(index,_)|index).collect::<Vec<_>>();
    if matches.len()!=1 {return Err(format!("element selector matched {} nodes; refine it or re-observe, no input dispatched",matches.len()));}
    let mut index=matches[0];
    while nodes[index].attributes.get("clickable").is_none_or(|s|s!="true") {
        index=nodes[index].parent.ok_or("matched element has no clickable ancestor; no input dispatched")?;
        if nodes[index].attributes.get("enabled").is_some_and(|s|s!="true")
            || nodes[index].attributes.get("visible-to-user").is_some_and(|s|s!="true") {return Err("clickable ancestor is disabled or hidden".into());}
    }
    let node=&nodes[index].attributes;
    let bounds=node.get("bounds").ok_or("element has no bounds")?;
    let pieces=bounds.trim_start_matches('[').trim_end_matches(']').split("][").flat_map(|s|s.split(',')).collect::<Vec<_>>();
    if pieces.len()!=4 {return Err("invalid element bounds".into());}
    let bounds=pieces.into_iter().map(|s|s.parse::<u32>().map_err(|_|"invalid element bounds".to_string())).collect::<Result<Vec<_>>>()?;
    if bounds.iter().any(|v|*v>16000)||bounds[2]<=bounds[0]||bounds[3]<=bounds[1] {return Err("element is offscreen or has empty bounds".into());}
    Ok(Target {text:node.get("text").cloned().unwrap_or_default(),description:node.get("content-desc").cloned().unwrap_or_default(),
        resource_id:node.get("resource-id").cloned().unwrap_or_default(),bounds:[bounds[0],bounds[1],bounds[2],bounds[3]],x:(bounds[0]+bounds[2])/2,y:(bounds[1]+bounds[3])/2})
}
