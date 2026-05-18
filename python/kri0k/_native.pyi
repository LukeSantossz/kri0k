"""Type stubs for kri0k._native (Rust extension module).

Generated from Rust PyO3 bindings.
"""

from typing import Any

def hello() -> str:
    """Return a greeting message from the Rust core.

    Returns:
        A greeting string confirming initialization.
    """

def get_dummy_graph() -> dict[str, Any]:
    """Return a dummy graph structure for testing.

    Returns:
        A dictionary with 'nodes' and 'edges' keys, where:
        - nodes: list of dicts with 'id', 'kind', and optional 'metadata'
        - edges: list of dicts with 'id', 'src', 'dst', and 'kind'
    """


class Engagement:
    """Stateful engagement container exposed by the Rust core.

    Holds the canonical graph, scope config, audit sink, TTP registry, and
    cancellation token for a single engagement. All execution flows through
    ``execute_proposal``; ``snapshot`` returns a read-only view.

    Phase 4: only the whois TTP (T1590.001) is registered.
    """

    def __new__(cls, scope_dict: dict[str, Any]) -> Engagement:
        """Construct an Engagement from a parsed scope dict.

        Raises:
            RuntimeError: If whois binary not found in PATH (D-50 fail-fast).
            RuntimeError: If scope_dict.version is not 1, or required fields missing.
        """

    def snapshot(self) -> dict[str, Any]:
        """Return current graph state as a dict.

        Returns:
            A dictionary with 'nodes' and 'edges' keys (same shape as get_dummy_graph).
        """

    def execute_proposal(self, proposal: dict[str, Any]) -> dict[str, Any]:
        """Validate scope, dispatch TTP, apply graph mutations, return outcome.

        Args:
            proposal: dict with keys 'target' (str) and 'ttp_id' (str), at minimum.

        Returns:
            Outcome dict with keys: status, result, error, graph_delta, audit_id.
        """

    def scope_hash(self) -> str:
        """Return SHA-256 hex digest of the raw scope YAML (M-03)."""

    def kill(self) -> None:
        """Signal the cancellation token. All in-flight executions abort.

        Terminal: after kill(), the engagement cannot execute further proposals.
        """
