//! Graph structures using petgraph with typed nodes and edges.

use kri0k_core::{EdgeId, NodeId, Result};
use petgraph::stable_graph::{NodeIndex, StableGraph};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Node kind enumeration.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum NodeKind {
    /// Target host node.
    Host {
        /// IP address of the host.
        ip: String,
    },
    /// Network segment.
    Network {
        /// CIDR notation for the network.
        cidr: String,
    },
    /// Service endpoint.
    Service {
        /// Port number.
        port: u16,
        /// Protocol name (e.g., "tcp", "udp").
        protocol: String,
    },
    /// Discovery finding.
    Finding {
        /// Human-readable description.
        description: String,
    },
    /// Domain name (e.g., "example.com") — natural key for whois targets.
    Domain {
        /// Fully-qualified domain name.
        name: String,
    },
    /// Organization name extracted from whois Registrant Organization (D-42).
    Organization {
        /// Organization name as reported by whois.
        name: String,
    },
    /// Nameserver hostname (e.g., "ns1.example.com") — case-insensitive natural key.
    Nameserver {
        /// Nameserver hostname.
        hostname: String,
    },
}

/// Edge kind enumeration.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum EdgeKind {
    /// Host belongs to network.
    BelongsTo,
    /// Service runs on host.
    RunsOn,
    /// Finding related to node.
    RelatesTo {
        /// Type of relationship.
        relation: String,
    },
    /// Domain -> Organization edge: domain is registered by this organization (D-39).
    RegisteredBy,
    /// Domain -> Nameserver edge: domain delegates DNS to this nameserver (D-39).
    HasNameserver,
}

/// Node in the graph.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Node {
    /// Stable external ID.
    pub id: NodeId,
    /// Node classification.
    pub kind: NodeKind,
    /// Additional metadata.
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub metadata: HashMap<String, String>,
}

impl Node {
    /// Create a new node.
    #[must_use]
    pub fn new(kind: NodeKind) -> Self {
        Self {
            id: NodeId::new(),
            kind,
            metadata: HashMap::new(),
        }
    }
}

/// Edge in the graph.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Edge {
    /// Stable external ID.
    pub id: EdgeId,
    /// Source node ID.
    pub src: NodeId,
    /// Destination node ID.
    pub dst: NodeId,
    /// Edge classification.
    pub kind: EdgeKind,
}

impl Edge {
    /// Create a new edge.
    #[must_use]
    pub fn new(src: NodeId, dst: NodeId, kind: EdgeKind) -> Self {
        Self {
            id: EdgeId::new(),
            src,
            dst,
            kind,
        }
    }
}

/// Graph wrapper around petgraph `StableGraph`.
#[derive(Debug)]
pub struct Graph {
    inner: StableGraph<Node, Edge>,
    /// Map from external `NodeId` to internal `NodeIndex`.
    node_map: HashMap<NodeId, NodeIndex>,
}

impl Graph {
    /// Create a new empty graph.
    #[must_use]
    pub fn new() -> Self {
        Self {
            inner: StableGraph::new(),
            node_map: HashMap::new(),
        }
    }

    /// Add a node to the graph.
    pub fn add_node(&mut self, node: Node) -> NodeId {
        let node_id = node.id;
        let index = self.inner.add_node(node);
        self.node_map.insert(node_id, index);
        node_id
    }

    /// Add an edge between two nodes.
    ///
    /// # Errors
    /// Returns error if either node ID is not found in the graph.
    pub fn add_edge(&mut self, edge: Edge) -> Result<EdgeId> {
        let src_idx = self.node_map.get(&edge.src).ok_or_else(|| {
            kri0k_core::Error::Generic(format!("Source node not found: {}", edge.src))
        })?;
        let dst_idx = self.node_map.get(&edge.dst).ok_or_else(|| {
            kri0k_core::Error::Generic(format!("Destination node not found: {}", edge.dst))
        })?;

        let edge_id = edge.id;
        self.inner.add_edge(*src_idx, *dst_idx, edge);
        Ok(edge_id)
    }

    /// Get node count.
    #[must_use]
    pub fn node_count(&self) -> usize {
        self.inner.node_count()
    }

    /// Get edge count.
    #[must_use]
    pub fn edge_count(&self) -> usize {
        self.inner.edge_count()
    }

    /// Serialize graph to JSON-serializable structure.
    ///
    /// # Errors
    /// Returns error if serialization fails.
    pub fn to_json(&self) -> Result<serde_json::Value> {
        let nodes: Vec<_> = self
            .inner
            .node_weights()
            .map(|n| {
                serde_json::json!({
                    "id": n.id.to_string(),
                    "kind": n.kind,
                    "metadata": n.metadata,
                })
            })
            .collect();

        let edges: Vec<_> = self
            .inner
            .edge_weights()
            .map(|e| {
                serde_json::json!({
                    "id": e.id.to_string(),
                    "src": e.src.to_string(),
                    "dst": e.dst.to_string(),
                    "kind": e.kind,
                })
            })
            .collect();

        Ok(serde_json::json!({
            "nodes": nodes,
            "edges": edges,
        }))
    }
}

impl Default for Graph {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
#[allow(clippy::expect_used)] // expect is ok in tests
mod tests {
    use super::*;

    // --- NodeKind new variant tests (Task 1) ---

    #[test]
    fn test_node_kind_domain_serialization() {
        let kind = NodeKind::Domain {
            name: "example.com".into(),
        };
        let value = serde_json::to_value(&kind).expect("serialize");
        assert_eq!(value, serde_json::json!({"type": "domain", "name": "example.com"}));
    }

    #[test]
    fn test_node_kind_organization_serialization() {
        let kind = NodeKind::Organization {
            name: "Example Inc.".into(),
        };
        let value = serde_json::to_value(&kind).expect("serialize");
        assert_eq!(value, serde_json::json!({"type": "organization", "name": "Example Inc."}));
    }

    #[test]
    fn test_node_kind_nameserver_serialization() {
        let kind = NodeKind::Nameserver {
            hostname: "ns1.example.com".into(),
        };
        let value = serde_json::to_value(&kind).expect("serialize");
        assert_eq!(value, serde_json::json!({"type": "nameserver", "hostname": "ns1.example.com"}));
    }

    #[test]
    fn test_node_kind_deserialization_roundtrip() {
        let variants: Vec<NodeKind> = vec![
            NodeKind::Domain { name: "example.com".into() },
            NodeKind::Organization { name: "Example Inc.".into() },
            NodeKind::Nameserver { hostname: "ns1.example.com".into() },
        ];
        for kind in variants {
            let serialized = serde_json::to_value(&kind).expect("serialize");
            let deserialized: NodeKind = serde_json::from_value(serialized).expect("deserialize");
            assert_eq!(deserialized, kind);
        }
    }

    #[test]
    fn test_node_kind_existing_variants_still_work() {
        let kind = NodeKind::Host { ip: "1.2.3.4".into() };
        let value = serde_json::to_value(&kind).expect("serialize");
        assert_eq!(value, serde_json::json!({"type": "host", "ip": "1.2.3.4"}));
    }

    // --- EdgeKind new variant tests (Task 2) ---

    #[test]
    fn test_edge_kind_registered_by_serialization() {
        let kind = EdgeKind::RegisteredBy;
        let value = serde_json::to_value(&kind).expect("serialize");
        assert_eq!(value, serde_json::json!({"type": "registered_by"}));
    }

    #[test]
    fn test_edge_kind_has_nameserver_serialization() {
        let kind = EdgeKind::HasNameserver;
        let value = serde_json::to_value(&kind).expect("serialize");
        assert_eq!(value, serde_json::json!({"type": "has_nameserver"}));
    }

    #[test]
    fn test_edge_kind_new_variants_roundtrip() {
        let variants: Vec<EdgeKind> = vec![EdgeKind::RegisteredBy, EdgeKind::HasNameserver];
        for kind in variants {
            let serialized = serde_json::to_value(&kind).expect("serialize");
            let deserialized: EdgeKind = serde_json::from_value(serialized).expect("deserialize");
            assert_eq!(deserialized, kind);
        }
    }

    #[test]
    fn test_edge_kind_existing_variants_still_work() {
        let kind = EdgeKind::BelongsTo;
        let value = serde_json::to_value(&kind).expect("serialize");
        assert_eq!(value, serde_json::json!({"type": "belongs_to"}));
    }

    // --- Graph tests (existing) ---

    #[test]
    fn test_graph_creation() {
        let graph = Graph::new();
        assert_eq!(graph.node_count(), 0);
        assert_eq!(graph.edge_count(), 0);
    }

    #[test]
    fn test_add_nodes() {
        let mut graph = Graph::new();
        let node1 = Node::new(NodeKind::Host {
            ip: "192.168.1.1".to_string(),
        });
        let node2 = Node::new(NodeKind::Network {
            cidr: "192.168.1.0/24".to_string(),
        });

        graph.add_node(node1);
        graph.add_node(node2);

        assert_eq!(graph.node_count(), 2);
    }

    #[test]
    fn test_add_edge() {
        let mut graph = Graph::new();
        let node1 = Node::new(NodeKind::Host {
            ip: "192.168.1.1".to_string(),
        });
        let node2 = Node::new(NodeKind::Network {
            cidr: "192.168.1.0/24".to_string(),
        });

        let id1 = graph.add_node(node1);
        let id2 = graph.add_node(node2);

        let edge = Edge::new(id1, id2, EdgeKind::BelongsTo);
        let result = graph.add_edge(edge);

        assert!(result.is_ok());
        assert_eq!(graph.edge_count(), 1);
    }

    #[test]
    fn test_graph_serialization() {
        let mut graph = Graph::new();
        let node = Node::new(NodeKind::Host {
            ip: "192.168.1.1".to_string(),
        });
        graph.add_node(node);

        let json = graph.to_json().expect("serialize");
        assert!(json["nodes"].is_array());
        assert!(json["edges"].is_array());
        assert_eq!(json["nodes"].as_array().expect("array").len(), 1);
    }
}
