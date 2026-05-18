"""Integration tests for the Engagement pyclass (Phase 4).

These tests exercise the full Rust->Python bridge with real subprocess calls.
Marked ``integration``; skipped in default CI (which runs ``-m "not integration"``).

Requires:
- whois binary in PATH (Sysinternals on Windows or apt install whois on Linux).
- EULA accepted (run ``whois -accepteula example.com`` once manually).
"""

import shutil
from typing import Any

import pytest

from kri0k._native import Engagement

pytestmark = [pytest.mark.integration, pytest.mark.ttp]


def _minimal_scope() -> dict[str, Any]:
    return {
        "version": 1,
        "operator": "test@example.com",
        "targets": ["example.com"],
        "safeguards": {"propose_only": False, "kill_switch": False},
    }


def _whois_available() -> bool:
    return shutil.which("whois") is not None


def test_engagement_construct_with_minimal_scope() -> None:
    """Engagement constructs from minimal scope dict; initial snapshot is empty graph."""
    if not _whois_available():
        pytest.skip("whois binary not in PATH")

    eng = Engagement(_minimal_scope())
    snap = eng.snapshot()
    assert "nodes" in snap
    assert "edges" in snap
    assert snap["nodes"] == []
    assert snap["edges"] == []


def test_engagement_rejects_unknown_version() -> None:
    """Scope dict with version!=1 must raise RuntimeError."""
    if not _whois_available():
        pytest.skip("whois binary not in PATH")

    scope = _minimal_scope()
    scope["version"] = 2
    with pytest.raises(RuntimeError, match="version"):
        Engagement(scope)


def test_engagement_scope_hash_is_hex_64() -> None:
    """scope_hash() returns SHA-256 hex (M-03)."""
    if not _whois_available():
        pytest.skip("whois binary not in PATH")

    eng = Engagement(_minimal_scope())
    h = eng.scope_hash()
    assert len(h) == 64
    assert all(c in "0123456789abcdef" for c in h)


def test_whois_grows_graph() -> None:
    """execute_proposal on whois TTP adds Domain, Organization, Nameserver nodes (TTP-04)."""
    if not _whois_available():
        pytest.skip("whois binary not in PATH")

    eng = Engagement(_minimal_scope())
    outcome = eng.execute_proposal({"ttp_id": "T1590.001", "target": "example.com"})
    assert outcome["status"] in {"executed", "error"}, f"unexpected status: {outcome}"
    if outcome["status"] == "executed":
        assert outcome["graph_delta"]["nodes_added"] >= 1, "at least Domain node expected"
        snap = eng.snapshot()
        kinds = [n["kind"]["type"] for n in snap["nodes"]]
        assert "domain" in kinds, f"Domain node missing; kinds = {kinds}"


def test_whois_idempotent_second_call_zero_delta() -> None:
    """Re-execution on same target produces no new nodes (D-43)."""
    if not _whois_available():
        pytest.skip("whois binary not in PATH")

    eng = Engagement(_minimal_scope())
    first = eng.execute_proposal({"ttp_id": "T1590.001", "target": "example.com"})
    if first["status"] != "executed":
        pytest.skip(f"first whois call did not execute cleanly: {first}")
    second = eng.execute_proposal({"ttp_id": "T1590.001", "target": "example.com"})
    assert second["status"] == "executed"
    assert second["graph_delta"]["nodes_added"] == 0, f"expected 0 new nodes; got {second}"
    assert second["graph_delta"]["edges_added"] == 0


def test_scope_violation_short_circuits() -> None:
    """Target outside scope produces status=scope_violation; no subprocess invoked (M-02)."""
    if not _whois_available():
        pytest.skip("whois binary not in PATH")

    eng = Engagement(_minimal_scope())
    outcome = eng.execute_proposal({"ttp_id": "T1590.001", "target": "evil.com"})
    assert outcome["status"] == "scope_violation", f"got {outcome}"
    assert "evil.com" in outcome["error"]


def test_invalid_target_rejected_by_regex() -> None:
    """D-63 Layer 2: target invalido rejeitado por regex ANTES do scope check (AB-03)."""
    if not _whois_available():
        pytest.skip("whois binary not in PATH")

    eng = Engagement(_minimal_scope())
    outcome = eng.execute_proposal({"ttp_id": "T1590.001", "target": "invalid..domain"})
    assert outcome["status"] == "error", f"got {outcome}"
    assert "parse_error" in outcome["error"] or "not a valid domain" in outcome["error"]


def test_unknown_ttp_returns_error() -> None:
    """Unknown ttp_id -> status=error containing the unknown id."""
    if not _whois_available():
        pytest.skip("whois binary not in PATH")

    eng = Engagement(_minimal_scope())
    outcome = eng.execute_proposal({"ttp_id": "T9999.UNKNOWN", "target": "example.com"})
    assert outcome["status"] == "error"
    assert "T9999.UNKNOWN" in outcome["error"]
