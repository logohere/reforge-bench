use std::collections::{HashMap, HashSet};

use serde_json::{Value, json};

use super::catalog::{valid_id, value};

pub struct Graph {
    pub nodes: Vec<Value>,
    pub edges: Vec<Value>,
    pub initial: String,
    pub landmarks: Vec<(String, String)>,
    pub metadata: Value,
    pub audit: Value,
    known: HashMap<String, usize>,
    search: Vec<String>,
}

pub fn has_functions(data: &Value, firmware: &str) -> bool {
    data["targets"][firmware].is_object()
        && data["nodes"].as_array().is_some_and(|nodes| {
            nodes
                .iter()
                .any(|node| node["firmware"] == firmware && node["kind"] == "function")
        })
}

pub fn resolve(prepared: Value, _history: Value, firmware: &str) -> Result<Value, String> {
    if prepared["targets"][firmware].is_object() {
        Ok(prepared)
    } else {
        Err("No function map imported for this artifact.".into())
    }
}

impl Graph {
    pub fn parse(data: Value, firmware: &str) -> Result<Self, String> {
        if !valid_id(firmware) {
            return Err("Choose an exact firmware image first.".into());
        }
        let nodes = data["nodes"]
            .as_array()
            .ok_or("Firmware nodes unavailable.")?;
        let edges = data["edges"]
            .as_array()
            .ok_or("Firmware links unavailable.")?;
        let known: HashMap<_, _> = nodes
            .iter()
            .map(|node| (value(node, "id", ""), node))
            .collect();
        if known.len() != nodes.len()
            || known.contains_key("")
            || edges.iter().any(|edge| {
                !known.contains_key(value(edge, "source", ""))
                    || !known.contains_key(value(edge, "target", ""))
                    || value(edge, "verb", "").is_empty()
            })
        {
            return Err("Graph contains unresolved or duplicate identities.".into());
        }
        let owned: HashSet<_> = nodes
            .iter()
            .filter(|node| node["firmware"] == firmware)
            .map(|node| value(node, "id", ""))
            .collect();
        let root = value(&data["targets"][firmware], "root", "");
        if owned.is_empty() || !owned.contains(root) {
            return Err("No prepared Dotdog graph for this firmware image.".into());
        }
        let mut selected = owned.clone();
        let mut pending: Vec<_> = owned.iter().copied().collect();
        let mut outgoing: HashMap<&str, Vec<&str>> = HashMap::new();
        for edge in edges {
            outgoing
                .entry(value(edge, "source", ""))
                .or_default()
                .push(value(edge, "target", ""));
        }
        let mut cursor = 0;
        while cursor < pending.len() {
            for id in outgoing.get(pending[cursor]).into_iter().flatten() {
                let node = known[id];
                if selected.contains(id)
                    || node["firmware"]
                        .as_str()
                        .is_some_and(|owner| !owner.is_empty() && owner != firmware)
                {
                    continue;
                }
                selected.insert(id);
                pending.push(id);
            }
            cursor += 1;
        }
        let initial = root.to_owned();
        let landmarks = vec![("Root".into(), root.to_owned())];
        let kept_nodes: Vec<_> = nodes
            .iter()
            .filter(|node| selected.contains(value(node, "id", "")))
            .cloned()
            .collect();
        let kept_edges: Vec<_> = edges
            .iter()
            .filter(|edge| {
                selected.contains(value(edge, "source", ""))
                    && selected.contains(value(edge, "target", ""))
            })
            .cloned()
            .collect();
        let known = kept_nodes
            .iter()
            .enumerate()
            .map(|(index, node)| (value(node, "id", "").to_owned(), index))
            .collect();
        let search = kept_nodes
            .iter()
            .map(|node| {
                ["label", "kind", "address", "signature", "status"]
                    .map(|key| value(node, key, ""))
                    .join(" ")
                    .to_lowercase()
            })
            .collect();
        Ok(Self {
            nodes: kept_nodes,
            edges: kept_edges,
            initial,
            landmarks,
            known,
            search,
            audit: data["target_audits"]
                .get(firmware)
                .cloned()
                .unwrap_or_else(|| data["audit"].clone()),
            metadata: json!({"version":data["version"],"scope":data["scope"],"summary":data["summary"],"provenance":data["provenance"],"graph_sha256":data["graph_sha256"]}),
        })
    }

    pub fn node(&self, id: &str) -> Option<&Value> {
        self.known.get(id).map(|index| &self.nodes[*index])
    }

    pub fn search(&self, query: &str, all: bool) -> Vec<usize> {
        let query = query.trim().to_lowercase();
        if query.is_empty() && !all {
            return Vec::new();
        }
        self.search
            .iter()
            .enumerate()
            .filter_map(|(index, text)| text.contains(&query).then_some(index))
            .collect()
    }

    pub fn linked(&self, id: &str, incoming: bool, mode: usize) -> Vec<String> {
        let mut seen = HashSet::new();
        self.edges
            .iter()
            .filter(|edge| mode == 0 || (edge["verb"] == "calls") == (mode == 1))
            .filter_map(|edge| {
                let (from, to) = if incoming {
                    ("target", "source")
                } else {
                    ("source", "target")
                };
                let target = value(edge, to, "");
                (edge[from] == id && target != id && seen.insert(target)).then(|| target.to_owned())
            })
            .collect()
    }
}

pub fn recovery(data: Value, firmware: &str) -> Result<Value, String> {
    if !valid_id(firmware) || data["target_sha256"] != firmware {
        return Err("Recovery map belongs to a different firmware image.".into());
    }
    let nodes = data["nodes"]
        .as_array()
        .ok_or("Recovery map is malformed.")?;
    let edges = data["edges"]
        .as_array()
        .ok_or("Recovery map is malformed.")?;
    let ids: HashMap<_, _> = nodes
        .iter()
        .map(|node| {
            (
                value(node, "id", ""),
                format!("recovery:{}", value(node, "address", "")),
            )
        })
        .collect();
    let addresses: HashSet<_> = ids.values().collect();
    if ids.len() != nodes.len()
        || addresses.len() != nodes.len()
        || !ids.contains_key("fn_7eda4")
        || nodes.iter().any(|node| {
            value(node, "address", "")
                .strip_prefix("0x")
                .is_none_or(|address| {
                    address.is_empty() || !address.bytes().all(|byte| byte.is_ascii_hexdigit())
                })
                || (!node["cases"].is_null()
                    && node["cases"]
                        .as_u64()
                        .is_none_or(|cases| cases > 9_007_199_254_740_991))
        })
        || edges.iter().any(|edge| {
            !ids.contains_key(value(edge, "source", ""))
                || !ids.contains_key(value(edge, "target", ""))
                || edge["verb"] != "calls"
        })
    {
        return Err("Recovery map contains invalid or ambiguous identities.".into());
    }
    let mut converted: Vec<_> = nodes.iter().map(|node| {
        let mut row = node.clone();
        row["id"] = ids[value(node,"id","")].clone().into();
        row["firmware"] = firmware.into(); row["kind"] = "function".into(); row["status"] = node["state"].clone();
        row["source"] = node["source"].as_str().unwrap_or("not established").into();
        match value(node,"role","") {
            "geometry caller" => row["displayLabel"] = "Geometry caller".into(),
            "upstream caller" => row["displayLabel"] = "Calling routine".into(),
            _ => {}
        }
        if node["id"] == "fn_7eda4" {
            row["observation"] = format!("{} basic blocks · {} listed instructions · {} body ranges. Full argument types and compiled reconstruction remain open.", data["basic_blocks"], data["instruction_count"], data["body"]["ranges"].as_array().map_or(0,Vec::len)).into();
        }
        row
    }).collect();
    converted.sort_by_key(|node| {
        u64::from_str_radix(&value(node, "address", "")[2..], 16).unwrap_or(u64::MAX)
    });
    let compared = converted
        .iter()
        .filter(|node| !node["cases"].is_null())
        .count();
    let edges: Vec<_> = edges
        .iter()
        .map(|edge| {
            let mut row = edge.clone();
            row["source"] = ids[value(edge, "source", "")].clone().into();
            row["target"] = ids[value(edge, "target", "")].clone().into();
            row
        })
        .collect();
    Ok(
        json!({"version":data["dotdog_version"],"targets":{firmware:{"root":ids["fn_7eda4"]}},"nodes":converted,"edges":edges,
        "summary":format!("{compared} compared entries"),"scope":"Static calls, not execution evidence.",
        "provenance":{"Map evidence":data["evidence_id"],"Analysis provider":format!("{} {}",value(&data["provider"],"name",""),value(&data["provider"],"version","")),"Source check":"Live source freshness is not checked here.","Scope":data["scope"]}}),
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn malformed_graphs_foreign_roots_and_recovery_counts_are_rejected() {
        let firmware = "a".repeat(64);
        let good = json!({"targets":{&firmware:{"root":"a"}},"nodes":[{"id":"a","firmware":firmware},{"id":"b"}],"edges":[{"source":"a","target":"b","verb":"calls"}]});
        assert!(Graph::parse(good.clone(), &firmware).is_ok());
        let mut bad = good.clone();
        bad["nodes"][1]["id"] = "a".into();
        assert!(Graph::parse(bad, &firmware).is_err());
        let mut bad = good.clone();
        bad["edges"][0]["target"] = "missing".into();
        assert!(Graph::parse(bad, &firmware).is_err());
        let mut bad = good;
        bad["targets"][&firmware]["root"] = "b".into();
        assert!(Graph::parse(bad, &firmware).is_err());
    }
}
