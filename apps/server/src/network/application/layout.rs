use crate::network::domain::{GraphNode, NodeType};

/// Assigns tiered positions for React Flow layout.
pub fn layout_graph(nodes: &mut [GraphNode]) {
    let mut tiers: std::collections::HashMap<i32, Vec<usize>> = std::collections::HashMap::new();

    for (index, node) in nodes.iter().enumerate() {
        let tier = node.node_type.unwrap_or(NodeType::Unknown).layout_tier();
        tiers.entry(tier).or_default().push(index);
    }

    for (tier, indices) in tiers {
        let count = indices.len() as f64;
        for (position, index) in indices.into_iter().enumerate() {
            let x = if count <= 1.0 {
                250.0
            } else {
                80.0 + (position as f64) * (440.0 / (count - 1.0))
            };
            nodes[index].position.x = x;
            nodes[index].position.y = 80.0 + f64::from(tier) * 160.0;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::network::domain::{GraphNodeData, GraphPosition};

    #[test]
    fn layout_spreads_nodes_by_tier() {
        let mut nodes = vec![
            GraphNode {
                id: "browser".into(),
                label: "Browser".into(),
                node_type: Some(NodeType::Browser),
                service_type: None,
                position: GraphPosition { x: 0.0, y: 0.0 },
                data: GraphNodeData {
                    event_count: 1,
                    total_bytes: 1,
                },
            },
            GraphNode {
                id: "db".into(),
                label: "Database".into(),
                node_type: Some(NodeType::Database),
                service_type: None,
                position: GraphPosition { x: 0.0, y: 0.0 },
                data: GraphNodeData {
                    event_count: 1,
                    total_bytes: 1,
                },
            },
        ];

        layout_graph(&mut nodes);
        assert!(nodes[0].position.y < nodes[1].position.y);
    }
}
