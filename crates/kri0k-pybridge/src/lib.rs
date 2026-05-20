//! `PyO3` bindings for kri0k Rust core.
#![allow(clippy::useless_conversion)] // PyResult type annotations trigger false positives

use kri0k_core::{
    audit::{AuditSink, NoopAuditSink, TtpExecutionEvent},
    scope::ScopeConfig,
    ttp::{subprocess::RealSubprocess, whois::WhoisTtp, Ttp, TtpOutput},
    Error, NodeId,
};
use kri0k_graph::{Edge, EdgeKind, Graph, Node, NodeKind};
use pyo3::exceptions::PyRuntimeError;
use pyo3::prelude::*;
use pyo3::types::PyDict;
use regex::Regex;
use serde_json::Value as JsonValue;
use std::collections::HashMap;
use std::sync::{Arc, Mutex, OnceLock};
use tokio_util::sync::CancellationToken;
use tracing::instrument;

/// Global Tokio runtime for async operations.
static TOKIO_RUNTIME: OnceLock<tokio::runtime::Runtime> = OnceLock::new();

/// D-63 Layer 2: regex de validação de domínio (case-insensitive).
/// Aplicado em `execute_proposal` ANTES do scope check (defense in depth).
const DOMAIN_REGEX: &str =
    r"^(?i)[a-z0-9](?:[a-z0-9-]{0,61}[a-z0-9])?(?:\.[a-z0-9](?:[a-z0-9-]{0,61}[a-z0-9])?)+$";

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

/// Convert a `serde_json::Value` to a Python object via JSON round-trip.
///
/// Requires the GIL (Python import + call).
fn json_value_to_pydict(py: Python<'_>, v: &JsonValue) -> PyResult<Py<PyAny>> {
    let json_str = serde_json::to_string(v)
        .map_err(|e| PyRuntimeError::new_err(format!("json serialize: {e}")))?;
    let json_module = py.import("json")?;
    let result = json_module.getattr("loads")?.call1((json_str,))?;
    Ok(result.into())
}

/// Convert a Python dict to `serde_json::Value` via JSON dumps round-trip.
///
/// Requires the GIL to be held (`PyDict` access).
fn pydict_to_json_value(py: Python<'_>, dict: &Bound<'_, PyDict>) -> PyResult<JsonValue> {
    let json_module = py.import("json")?;
    let dumps_fn = json_module.getattr("dumps")?;
    let py_str = dumps_fn.call1((dict,))?;
    let s: String = py_str.extract()?;
    serde_json::from_str(&s).map_err(|e| PyRuntimeError::new_err(format!("json parse: {e}")))
}

/// Map an `Error` to a `PyErr` (used by `#[new]`).
#[allow(clippy::needless_pass_by_value)] // owned `Error` is what `.map_err` hands us
fn error_to_pyerr(e: Error) -> PyErr {
    PyRuntimeError::new_err(e.to_string())
}

/// Map an `Error` to an outcome JSON value per D-47.
///
/// D-47 outcome status set: `executed | scope_violation | rate_limited | timeout | error | proposed`.
/// `Cancelled` and `ParseError` both map to `"error"` with a categorised message prefix.
fn error_to_outcome(e: &Error) -> JsonValue {
    let (status, error_msg) = match e {
        Error::ScopeViolation { target, reason } => (
            "scope_violation",
            format!("scope violation for {target:?}: {reason}"),
        ),
        Error::RateLimitExceeded {
            ttp_id,
            retry_in_ms,
        } => (
            "rate_limited",
            format!("rate limit exceeded for {ttp_id}: retry in {retry_in_ms}ms"),
        ),
        Error::SubprocessTimeout { ttp_id, timeout_ms } => (
            "timeout",
            format!("TTP {ttp_id} timed out after {timeout_ms}ms"),
        ),
        Error::UnknownTtp { ttp_id } => ("error", format!("unknown TTP id: {ttp_id}")),
        Error::ParseError { origin, detail } => {
            ("error", format!("parse_error in {origin}: {detail}"))
        },
        Error::Cancelled => ("error", "operation cancelled".to_string()),
        other => ("error", other.to_string()),
    };
    serde_json::json!({
        "status": status,
        "result": JsonValue::Null,
        "error": error_msg,
        "graph_delta": { "nodes_added": 0, "edges_added": 0 },
        "audit_id": JsonValue::Null,
    })
}

/// Build the `executed` outcome JSON value per D-47.
fn build_executed_outcome(result: &TtpOutput, nodes_added: usize, edges_added: usize) -> JsonValue {
    let result_json = match result {
        TtpOutput::Whois(w) => serde_json::to_value(w).unwrap_or(JsonValue::Null),
    };
    serde_json::json!({
        "status": "executed",
        "result": result_json,
        "error": JsonValue::Null,
        "graph_delta": { "nodes_added": nodes_added, "edges_added": edges_added },
        "audit_id": JsonValue::Null,
    })
}

/// Internal state held inside `Arc` to keep `Engagement` cheap-to-clone-by-reference.
struct EngagementInner {
    graph: Mutex<Graph>,
    scope: ScopeConfig,
    /// Pitfall 5+12: `Mutex<Box<dyn AuditSink>>` provides `Sync` (`PyO3` 0.24 requirement).
    audit: Mutex<Box<dyn AuditSink>>,
    /// TTP registry — Phase 4: only T1590.001 (D-52, hardcoded `HashMap`).
    registry: HashMap<String, Box<dyn Ttp>>,
    /// Kill switch cancellation token (D-62, M-36).
    cancel: CancellationToken,
    /// Dedupe cache: (`kind_tag`, `natural_key`) → `NodeId` (D-43).
    dedupe: Mutex<HashMap<(String, String), NodeId>>,
}

impl EngagementInner {
    /// Apply a `TtpOutput::Whois` to the graph with idempotent deduplication (D-43).
    ///
    /// Lock order is deadlock-free: `dedupe` is acquired before `graph`.
    /// Edge invariant: an edge is added only if at least one of its endpoints
    /// is newly inserted in this call (otherwise the relationship is already
    /// represented).
    ///
    /// Returns `(nodes_added, edges_added)` for this call.
    #[allow(clippy::option_if_let_else)] // `if let Some else` reads clearer than `map_or_else` closures here
    #[allow(clippy::significant_drop_tightening)] // locks are intentionally held across the full mutation
    fn apply_whois_output(
        &self,
        target: &str,
        output: &TtpOutput,
    ) -> Result<(usize, usize), Error> {
        let TtpOutput::Whois(whois) = output;

        // PATTERN: lock dedupe before graph to avoid deadlock.
        let mut dedupe = self
            .dedupe
            .lock()
            .map_err(|e| Error::Generic(format!("dedupe mutex poisoned: {e}")))?;
        let mut graph = self
            .graph
            .lock()
            .map_err(|e| Error::Generic(format!("graph mutex poisoned: {e}")))?;

        let mut nodes_added = 0usize;
        let mut edges_added = 0usize;

        // 1. Domain node (always — target is the natural key).
        let domain_key = ("domain".to_string(), target.to_string());
        let (domain_id, domain_was_new) = if let Some(id) = dedupe.get(&domain_key).copied() {
            (id, false)
        } else {
            let id = graph.add_node(Node::new(NodeKind::Domain {
                name: target.to_string(),
            }));
            dedupe.insert(domain_key, id);
            nodes_added += 1;
            (id, true)
        };

        // 2. Organization node — only when Registrant present (D-42).
        let org_pair = if let Some(registrant) = &whois.registrant {
            let org_key = ("organization".to_string(), registrant.clone());
            let (oid, was_new) = if let Some(id) = dedupe.get(&org_key).copied() {
                (id, false)
            } else {
                let id = graph.add_node(Node::new(NodeKind::Organization {
                    name: registrant.clone(),
                }));
                dedupe.insert(org_key, id);
                nodes_added += 1;
                (id, true)
            };
            Some((oid, was_new))
        } else {
            None
        };

        // 3. Nameserver nodes — track was_new per NS.
        let mut ns_was_new = Vec::with_capacity(whois.nameservers.len());
        for ns in &whois.nameservers {
            let ns_key = ("nameserver".to_string(), ns.clone());
            let (nid, was_new) = if let Some(id) = dedupe.get(&ns_key).copied() {
                (id, false)
            } else {
                let id = graph.add_node(Node::new(NodeKind::Nameserver {
                    hostname: ns.clone(),
                }));
                dedupe.insert(ns_key, id);
                nodes_added += 1;
                (id, true)
            };
            ns_was_new.push((nid, was_new));
        }

        // 4. Domain -RegisteredBy-> Organization: only if either endpoint is new.
        if let Some((oid, org_was_new)) = org_pair {
            if domain_was_new || org_was_new {
                let edge = Edge::new(domain_id, oid, EdgeKind::RegisteredBy);
                graph.add_edge(edge)?;
                edges_added += 1;
            }
        }

        // 5. Domain -HasNameserver-> NS: only for newly-added NS nodes.
        for (nid, was_new) in &ns_was_new {
            if *was_new {
                let edge = Edge::new(domain_id, *nid, EdgeKind::HasNameserver);
                graph.add_edge(edge)?;
                edges_added += 1;
            }
        }

        Ok((nodes_added, edges_added))
    }
}

/// Stateful engagement container exposing graph + scope + TTP registry to Python (D-34).
///
/// All state mutations go through pyclass methods (D-35 facade — there is no
/// `add_node` / `add_edge` exposed). Wraps an `Arc<EngagementInner>` so future
/// methods can clone cheaply if needed.
///
/// # Safety invariants
/// - All blocking I/O in `#[new]` executes inside `py.allow_threads` (Pitfall 7).
/// - `Mutex<Box<dyn AuditSink>>` provides `Sync` (Pitfall 5 + 12).
#[pyclass]
#[allow(missing_debug_implementations)] // Ttp trait has no Debug bound; deriving Debug would force the bound on all impls.
pub struct Engagement(Arc<EngagementInner>);

#[pymethods]
impl Engagement {
    /// Construct an `Engagement` from a parsed scope dict (D-34, D-50).
    ///
    /// # Errors
    /// - `RuntimeError` with install hint if the `whois` binary is absent from
    ///   `PATH` (D-50).
    /// - `RuntimeError` if the scope dict fails version check or schema
    ///   validation.
    #[new]
    fn new(py: Python<'_>, scope_dict: &Bound<'_, PyDict>) -> PyResult<Self> {
        // Extract PyDict -> serde_json::Value with GIL held (PyDict requires GIL).
        let scope_value = pydict_to_json_value(py, scope_dict)?;

        // Pitfall 7 (CONTEXT.md D-50 update 2026-05-18): GIL released for blocking init.
        // which::which performs filesystem I/O; ScopeConfig::from_dict_value runs YAML round-trip;
        // registry construction is CPU-bound.
        let inner = py
            .allow_threads(|| -> Result<EngagementInner, Error> {
                // 1. M-36 (D-50): fail-fast on missing whois binary.
                which::which("whois").map_err(|_| Error::MissingDependency {
                    binary: "whois".to_string(),
                })?;

                // 2. Parse scope from JSON value.
                let scope = ScopeConfig::from_dict_value(scope_value)?;

                // 3. D-52: build registry HashMap (hardcoded T1590.001 for Phase 4).
                let subprocess = Arc::new(RealSubprocess);
                let mut registry: HashMap<String, Box<dyn Ttp>> = HashMap::new();
                registry.insert("T1590.001".to_string(), Box::new(WhoisTtp::new(subprocess)));

                Ok(EngagementInner {
                    graph: Mutex::new(Graph::new()),
                    scope,
                    audit: Mutex::new(Box::new(NoopAuditSink)),
                    registry,
                    cancel: CancellationToken::new(),
                    dedupe: Mutex::new(HashMap::new()),
                })
            })
            .map_err(error_to_pyerr)?;

        Ok(Self(Arc::new(inner)))
    }

    /// Return the current graph as a Python dict (`{"nodes": [...], "edges": [...]}`).
    #[instrument(skip(self, py))]
    fn snapshot(&self, py: Python<'_>) -> PyResult<Py<PyAny>> {
        // Pitfall 7: GIL released for blocking serialization.
        let json_value = py
            .allow_threads(|| -> Result<JsonValue, Error> {
                let g = self
                    .0
                    .graph
                    .lock()
                    .map_err(|e| Error::Generic(format!("graph mutex poisoned: {e}")))?;
                g.to_json()
            })
            .map_err(error_to_pyerr)?;
        json_value_to_pydict(py, &json_value)
    }

    /// Execute a TTP proposal against the canonical graph (D-47, D-48, D-49, D-63).
    ///
    /// # Defense layers (D-63)
    /// - Layer 2: regex domain validation (`DOMAIN_REGEX`) — applied first.
    /// - Layer 1: exact-match scope allowlist (`scope.validate_target`).
    /// - Layer 3: no shell expansion in subprocess (enforced by `RealSubprocess`).
    ///
    /// # Returns
    /// Python dict `{status, result, error, graph_delta, audit_id}` (D-47).
    #[instrument(skip(self, py, proposal))]
    fn execute_proposal(
        &self,
        py: Python<'_>,
        proposal: &Bound<'_, PyDict>,
    ) -> PyResult<Py<PyAny>> {
        // Extract proposal fields with GIL held (Pitfall 7: never inside allow_threads).
        let target: String = proposal
            .get_item("target")?
            .ok_or_else(|| pyo3::exceptions::PyKeyError::new_err("proposal.target missing"))?
            .extract()?;
        let ttp_id: String = proposal
            .get_item("ttp_id")?
            .ok_or_else(|| pyo3::exceptions::PyKeyError::new_err("proposal.ttp_id missing"))?
            .extract()?;
        let cancel = self.0.cancel.clone();
        let inner = Arc::clone(&self.0);

        // Pitfall 7: GIL released for async work.
        let outcome_result: Result<JsonValue, Error> = py.allow_threads(|| {
            runtime().block_on(async move {
                // 1. AB-03 / D-63 Layer 2: regex domain validation BEFORE scope check.
                //    Catches prompt-injection-style metacharacters before any I/O.
                let re = Regex::new(DOMAIN_REGEX).map_err(|e| {
                    Error::Generic(format!("static domain regex failed to compile: {e}"))
                })?;
                if !re.is_match(&target) {
                    return Err(Error::ParseError {
                        origin: "proposal.target".to_string(),
                        detail: format!("target {target:?} is not a valid domain (D-63 Layer 2)"),
                    });
                }

                // 2. M-02 (D-48 / D-63 Layer 1): scope allowlist check (fail-closed).
                inner.scope.validate_target(&target)?;

                // 3. D-52: TTP registry lookup.
                let ttp = inner
                    .registry
                    .get(&ttp_id)
                    .ok_or_else(|| Error::UnknownTtp {
                        ttp_id: ttp_id.clone(),
                    })?;

                // 4. D-44, D-51, D-62: execute via subprocess abstraction with cancel.
                let result = ttp.execute(&target, cancel).await?;

                // 5. D-43: idempotent graph mutation.
                let (nodes_added, edges_added) = inner.apply_whois_output(&target, &result)?;

                // 6. M-22 (D-38 slot): NoopAuditSink for Phase 4; hash chain lands in Phase 8.
                let timestamp = std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .map_or_else(
                        |_| "unknown".to_string(),
                        |d| format!("unix-ms:{}", d.as_millis()),
                    );
                let event = TtpExecutionEvent {
                    timestamp,
                    ttp_id: ttp_id.clone(),
                    target: target.clone(),
                    outcome: "executed".to_string(),
                    llm_provider: None,
                };
                {
                    let mut audit_lock = inner
                        .audit
                        .lock()
                        .map_err(|e| Error::Generic(format!("audit mutex poisoned: {e}")))?;
                    audit_lock.log_ttp_execution(event)?;
                }

                // 7. D-47: build outcome dict.
                Ok(build_executed_outcome(&result, nodes_added, edges_added))
            })
        });

        let outcome = match outcome_result {
            Ok(v) => v,
            Err(e) => error_to_outcome(&e),
        };
        json_value_to_pydict(py, &outcome)
    }

    /// Return the SHA-256 hex digest of the scope YAML (M-03).
    fn scope_hash(&self) -> String {
        self.0.scope.compute_hash()
    }

    /// D-62 / M-36: signal the cancellation token.
    ///
    /// Terminal — after `kill()`, all future `execute_proposal` calls return
    /// `Error::Cancelled`.
    fn kill(&self) {
        self.0.cancel.cancel();
    }
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
    let json_module = py.import("json")?;
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
    m.add_class::<Engagement>()?;

    Ok(())
}

#[cfg(test)]
#[allow(clippy::expect_used)]
mod tests {
    use super::*;
    use kri0k_core::ttp::whois::WhoisOutput;

    impl EngagementInner {
        /// Test-only constructor: skips `which::which` and builds with a minimal scope.
        fn for_test() -> Self {
            let scope = ScopeConfig::from_dict_value(serde_json::json!({
                "version": 1,
                "targets": ["example.com"],
            }))
            .expect("test scope");
            Self {
                graph: Mutex::new(Graph::new()),
                scope,
                audit: Mutex::new(Box::new(NoopAuditSink)),
                registry: HashMap::new(),
                cancel: CancellationToken::new(),
                dedupe: Mutex::new(HashMap::new()),
            }
        }
    }

    #[test]
    fn test_apply_whois_output_adds_domain_org_ns() {
        let e = EngagementInner::for_test();
        let output = TtpOutput::Whois(WhoisOutput {
            registrant: Some("Acme Inc".into()),
            nameservers: vec!["ns1.acme.com".into(), "ns2.acme.com".into()],
            ..Default::default()
        });
        let (nodes, edges) = e
            .apply_whois_output("acme.com", &output)
            .expect("apply whois");
        assert_eq!(nodes, 4, "expected 1 Domain + 1 Org + 2 NS");
        assert_eq!(edges, 3, "expected 1 RegisteredBy + 2 HasNameserver");
    }

    #[test]
    fn test_apply_whois_output_idempotent_second_call() {
        let e = EngagementInner::for_test();
        let output = TtpOutput::Whois(WhoisOutput {
            registrant: Some("Acme Inc".into()),
            nameservers: vec!["ns1.acme.com".into()],
            ..Default::default()
        });
        let _ = e.apply_whois_output("acme.com", &output).expect("first");
        let (nodes, edges) = e.apply_whois_output("acme.com", &output).expect("second");
        assert_eq!(nodes, 0, "idempotent: no new nodes on second call");
        assert_eq!(edges, 0, "idempotent: no new edges on second call");
    }

    #[test]
    fn test_apply_whois_output_no_registrant_no_org_no_edge() {
        let e = EngagementInner::for_test();
        let output = TtpOutput::Whois(WhoisOutput {
            registrant: None,
            nameservers: vec!["ns1.x".into()],
            ..Default::default()
        });
        let (nodes, edges) = e.apply_whois_output("x.com", &output).expect("apply");
        assert_eq!(nodes, 2, "expected Domain + NS only");
        assert_eq!(edges, 1, "expected only HasNameserver");
    }

    #[test]
    fn test_error_to_outcome_scope_violation() {
        let v = error_to_outcome(&Error::ScopeViolation {
            target: "x".into(),
            reason: "y".into(),
        });
        assert_eq!(v["status"], "scope_violation");
        assert!(
            v["error"].as_str().expect("error is str").contains('x'),
            "error message must mention target"
        );
    }

    #[test]
    fn test_error_to_outcome_unknown_ttp() {
        let v = error_to_outcome(&Error::UnknownTtp {
            ttp_id: "T9999".into(),
        });
        assert_eq!(v["status"], "error");
        assert!(
            v["error"].as_str().expect("error is str").contains("T9999"),
            "error must mention ttp_id"
        );
    }

    #[test]
    fn test_error_to_outcome_cancelled() {
        let v = error_to_outcome(&Error::Cancelled);
        assert_eq!(v["status"], "error");
        assert!(
            v["error"]
                .as_str()
                .expect("error is str")
                .contains("cancelled"),
            "error must mention cancellation"
        );
    }

    #[test]
    fn test_error_to_outcome_parse_error() {
        let v = error_to_outcome(&Error::ParseError {
            origin: "proposal.target".into(),
            detail: "bogus".into(),
        });
        assert_eq!(v["status"], "error");
        let msg = v["error"].as_str().expect("error is str");
        assert!(
            msg.contains("parse_error in proposal.target"),
            "unexpected parse_error message: {msg}"
        );
    }

    #[test]
    fn test_domain_regex_accepts_valid_domains() {
        let re = Regex::new(DOMAIN_REGEX).expect("DOMAIN_REGEX compiles");
        for ok in &["example.com", "sub.example.co.uk", "a.b", "EXAMPLE.COM"] {
            assert!(re.is_match(ok), "expected match for {ok:?}");
        }
        for bad in &[
            "",
            "invalid..domain",
            "no_underscores.com",
            "-leadinghyphen.com",
            "trailinghyphen-.com",
        ] {
            assert!(!re.is_match(bad), "expected NO match for {bad:?}");
        }
    }
}
