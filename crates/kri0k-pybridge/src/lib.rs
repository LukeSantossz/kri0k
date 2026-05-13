//! `PyO3` bindings for kri0k Rust core.
#![allow(clippy::useless_conversion)] // PyResult type annotations trigger false positives

use kri0k_graph::{Edge, EdgeKind, Graph, Node, NodeKind};
use pyo3::prelude::*;
use std::sync::OnceLock;

/// Global Tokio runtime for async operations.
static TOKIO_RUNTIME: OnceLock<tokio::runtime::Runtime> = OnceLock::new();

/// Get or initialize the global Tokio runtime.
#[allow(clippy::expect_used)] // Runtime failure is unrecoverable
fn runtime() -> &'static tokio::runtime::Runtime {
    TOKIO_RUNTIME.get_or_init(|| {
        tokio::runtime::Builder::new_multi_thread()
            .worker_threads(2)
            .thread_name("kri0k-tokio")
            .enable_all()
            .build()
            .expect("Failed to create Tokio runtime")
    })
}

/// Returns a greeting message.
#[pyfunction]
fn hello() -> String {
    "Hello from kri0k! Rust core initialized.".to_string()
}

/// Returns a dummy graph structure for testing cross-language serialization.
#[pyfunction]
#[allow(clippy::expect_used)] // Demo function, failure is acceptable
#[allow(clippy::useless_conversion)] // False positive with PyResult
fn get_dummy_graph(py: Python<'_>) -> PyResult<Py<PyAny>> {
    // Release GIL while building graph
    let json_value = py.allow_threads(|| {
        let mut graph = Graph::new();

        // Create nodes
        let host1 = Node::new(NodeKind::Host {
            ip: "192.168.1.10".to_string(),
        });
        let host2 = Node::new(NodeKind::Host {
            ip: "192.168.1.20".to_string(),
        });
        let network = Node::new(NodeKind::Network {
            cidr: "192.168.1.0/24".to_string(),
        });
        let service = Node::new(NodeKind::Service {
            port: 80,
            protocol: "http".to_string(),
        });

        let id1 = graph.add_node(host1);
        let id2 = graph.add_node(host2);
        let id_net = graph.add_node(network);
        let id_svc = graph.add_node(service);

        // Create edges
        let edge1 = Edge::new(id1, id_net, EdgeKind::BelongsTo);
        let edge2 = Edge::new(id2, id_net, EdgeKind::BelongsTo);
        let edge3 = Edge::new(id_svc, id1, EdgeKind::RunsOn);

        graph.add_edge(edge1).ok();
        graph.add_edge(edge2).ok();
        graph.add_edge(edge3).ok();

        // Serialize to JSON
        graph.to_json().expect("Failed to serialize graph")
    });

    // Convert JSON value to Python dict
    let json_str = serde_json::to_string(&json_value).expect("Failed to stringify JSON");
    let json_module = py.import_bound("json")?;
    let loads_fn = json_module.getattr("loads")?;
    let result = loads_fn.call1((json_str,))?;
    Ok(result.into())
}

/// Python module initialization.
#[pymodule]
fn _native(m: &Bound<'_, PyModule>) -> PyResult<()> {
    // Initialize runtime on module load
    let _ = runtime();

    m.add_function(wrap_pyfunction!(hello, m)?)?;
    m.add_function(wrap_pyfunction!(get_dummy_graph, m)?)?;

    Ok(())
}
