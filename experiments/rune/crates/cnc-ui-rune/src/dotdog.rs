use serde_json::{Value, json};

use super::{catalog::value, graph_model::Graph};

pub fn parse(bytes: &[u8], artifact: &str) -> Result<Value, String> {
    if bytes.len() > 16 * 1_048_576 {
        return Err("Graph export exceeds 16 MiB".into());
    }
    let data: Value = serde_json::from_slice(bytes).map_err(|_| "Invalid Dotdog JSON export")?;
    if data["version"] != "1.0" || !data["project"].is_string() {
        return Err("Use a Dotdog 1.0 JSON graph export".into());
    }
    let nodes = data["nodes"]
        .as_array()
        .filter(|nodes| !nodes.is_empty() && nodes.len() <= 16_384)
        .ok_or("Invalid graph node count")?;
    let edges = data["edges"]
        .as_array()
        .filter(|edges| edges.len() <= 32_768)
        .ok_or("Invalid graph relationship count")?;
    let mut edge_ids = std::collections::HashSet::new();
    if nodes.iter().any(|node| {
        ["id", "label", "kind"].iter().any(|key| {
            node[key]
                .as_str()
                .is_none_or(|s| s.is_empty() || s.len() > 2048)
        })
    }) || edges.iter().any(|edge| {
        edge["id"]
            .as_str()
            .is_none_or(|id| id.is_empty() || id.len() > 4096 || !edge_ids.insert(id))
    }) {
        return Err("Invalid or duplicate Dotdog identities".into());
    }
    let converted: Vec<_> = nodes.iter().map(|node| {
        let mut record = json!({"id":node["id"],"label":node["label"],"kind":node["kind"],"firmware":artifact,
            "status":value(node,"confidence","Not provided"),"observation":value(node,"description","Imported graph statement"),
            "source":value(node,"source",""),"dotdog":node});
        if let Some(address)=node["properties"]["address"].as_str() {record["address"]=json!(address);}
        record
    }).collect();
    let links: Vec<_> = edges.iter().map(|edge|json!({"source":edge["sourceId"],"target":edge["targetId"],"verb":edge["verb"],"dotdog":edge})).collect();
    let digest = crate::sha256(bytes);
    let result = json!({"producer":"Dotdog","version":data["version"],"nodes":converted,"edges":links,
        "targets":{artifact:{"root":nodes[0]["id"]}},"scope":"Imported statements, not execution or measured connectivity",
        "summary":data["project"],"graph_sha256":digest,
        "provenance":{"Graph export SHA-256":digest,
            "Identity":"Export identity; original binary identities remain in node properties",
            "Limitations":data["limitations"].to_string()},"dotdog":data});
    Graph::parse(result.clone(), artifact)?;
    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dotdog_export_preserves_identities_links_and_declared_limits() {
        let bytes = include_bytes!("../../../../../web/hardware/workbench-map.json");
        let id = crate::sha256(bytes);
        let data = parse(bytes, &id).unwrap();
        let graph = Graph::parse(data.clone(), &id).unwrap();
        assert!(!graph.linked(&graph.initial, false, 0).is_empty());
        assert_eq!(
            data["dotdog"]["limitations"],
            serde_json::from_slice::<Value>(bytes).unwrap()["limitations"]
        );
        let mut bad = data["dotdog"].clone();
        bad["edges"][0]["targetId"] = json!("missing-node");
        assert!(parse(&serde_json::to_vec(&bad).unwrap(), &id).is_err());
        bad["nodes"][0]["id"] = bad["nodes"][1]["id"].clone();
        assert!(parse(&serde_json::to_vec(&bad).unwrap(), &id).is_err());
    }
}
