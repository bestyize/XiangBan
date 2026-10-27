use crate::package::{valid_path, Result};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Role { Core, Feature, Temporary }

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Node {
    pub id: String,
    pub title: String,
    pub page: String,
    pub role: Role,
    pub priority: f64,
    pub stability: f64,
    pub confidence: f64,
    #[serde(default = "yes")]
    pub enabled: bool,
    #[serde(default)]
    pub expires_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Edge {
    pub id: String,
    pub from: String,
    pub to: String,
    pub action: String,
    pub role: Role,
    pub cost: u32,
    pub stability: f64,
    pub confidence: f64,
    #[serde(default = "yes")]
    pub enabled: bool,
    #[serde(default)]
    pub expires_at: Option<DateTime<Utc>>,
}

fn yes() -> bool { true }
fn id(s: &str) -> bool {
    !s.is_empty() && s.len() <= 100 && s.bytes().all(|b| b.is_ascii_alphanumeric() || b"-_.".contains(&b))
}
fn weight(w: f64) -> bool { w.is_finite() && (0.0..=1.0).contains(&w) }

#[derive(Debug, Default, Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Graph {
    #[serde(default)]
    pub nodes: Vec<Node>,
    #[serde(default)]
    pub edges: Vec<Edge>,
}

impl Graph {
    pub fn has_temporary(&self) -> bool {
        self.nodes.iter().any(|n| n.role == Role::Temporary) || self.edges.iter().any(|e| e.role == Role::Temporary)
    }

    pub fn validate(&self, basic: bool) -> Result<()> {
        if self.nodes.len() > 256 || self.edges.len() > 512 { return Err("route graph exceeds budget".into()); }
        let mut nodes = BTreeSet::new();
        let mut edges = BTreeSet::new();
        for n in &self.nodes {
            if !id(&n.id) || !nodes.insert(&n.id) || n.title.is_empty() || n.title.len() > 200
                || !valid_path(&n.page) || !n.page.starts_with("pages/") || !n.page.ends_with(".md")
                || !weight(n.priority) || !weight(n.stability) || !weight(n.confidence)
                || (basic && (n.role == Role::Temporary || n.expires_at.is_some())) {
                return Err(format!("invalid route node or temporary data in basic/: {}", n.id));
            }
        }
        for e in &self.edges {
            if !id(&e.id) || !id(&e.from) || !id(&e.to) || !edges.insert(&e.id) || e.action.is_empty()
                || e.action.len() > 2000 || e.cost == 0 || e.cost > 1000 || !weight(e.stability) || !weight(e.confidence)
                || (basic && (e.role == Role::Temporary || e.expires_at.is_some())) {
                return Err(format!("invalid route edge or temporary data in basic/: {}", e.id));
            }
        }
        Ok(())
    }

    pub fn validate_resolved(&self) -> Result<()> {
        self.validate(false)?;
        if self.edges.iter().any(|e| !self.nodes.iter().any(|n| n.id == e.from) || !self.nodes.iter().any(|n| n.id == e.to)) {
            return Err("route edge refers to an unknown node".into());
        }
        Ok(())
    }

    /// Stable IDs replace a single node/edge. enabled=false is a tombstone.
    pub fn overlay(&mut self, other: Graph) {
        for node in other.nodes {
            if let Some(index) = self.nodes.iter().position(|n| n.id == node.id) { self.nodes[index] = node; }
            else { self.nodes.push(node); }
        }
        for edge in other.edges {
            if let Some(index) = self.edges.iter().position(|e| e.id == edge.id) { self.edges[index] = edge; }
            else { self.edges.push(edge); }
        }
    }

    pub fn filter(&mut self, temporary: bool, now: DateTime<Utc>) {
        let removed: BTreeSet<_> = self.nodes.iter().filter(|n| !n.enabled || (!temporary && n.role == Role::Temporary)
            || n.expires_at.is_some_and(|until| until <= now)).map(|n| n.id.clone()).collect();
        self.nodes.retain(|n| !removed.contains(&n.id));
        self.edges.retain(|e| e.enabled && (temporary || e.role != Role::Temporary)
            && e.expires_at.is_none_or(|until| until > now) && !removed.contains(&e.from) && !removed.contains(&e.to));
    }

    /// The graph suggests a route; it is never evidence that an action succeeded.
    pub fn plan(&self, from: &str, to: &str) -> Result<serde_json::Value> {
        self.validate_resolved()?;
        if !self.nodes.iter().any(|n| n.id == from) || !self.nodes.iter().any(|n| n.id == to) {
            return Err("unknown route start or destination".into());
        }
        let mut distances: BTreeMap<String, f64> = BTreeMap::from([(from.to_string(), 0.0)]);
        let mut previous: BTreeMap<String, &Edge> = BTreeMap::new();
        let mut visited = BTreeSet::new();
        loop {
            let next = distances.iter().filter(|(id, _)| !visited.contains(*id))
                .min_by(|a,b| a.1.total_cmp(b.1)).map(|(id,d)| (id.clone(),*d));
            let Some((current, cost)) = next else { return Err("no applicable route".into()); };
            if current == to {
                let mut steps = Vec::new();
                let mut at = to;
                while at != from {
                    let edge = previous.get(at).ok_or("incomplete route")?;
                    steps.push(*edge);
                    at = &edge.from;
                }
                steps.reverse();
                return Ok(serde_json::json!({"steps":steps,"cost":cost,"requires_live_verification":true}));
            }
            visited.insert(current.clone());
            for edge in self.edges.iter().filter(|e| e.from == current) {
                let node = self.nodes.iter().find(|n| n.id == edge.to).ok_or("missing route node")?;
                let risk = 1.0 + (1.0-edge.stability) + (1.0-edge.confidence)
                    + (1.0-node.stability) + (1.0-node.confidence) + (1.0-node.priority)*0.25;
                let candidate = cost + edge.cost as f64 * risk;
                if distances.get(&edge.to).is_none_or(|old| candidate < *old) {
                    distances.insert(edge.to.clone(), candidate);
                    previous.insert(edge.to.clone(), edge);
                }
            }
        }
    }
}
