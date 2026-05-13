"""Type stubs for kri0k._native (Rust extension module).

Generated from Rust PyO3 bindings.
"""

from typing import Any

def hello() -> str:
    """Return a greeting message from the Rust core.
    
    Returns:
        A greeting string confirming initialization.
    """
    ...

def get_dummy_graph() -> dict[str, Any]:
    """Return a dummy graph structure for testing.
    
    Returns:
        A dictionary with 'nodes' and 'edges' keys, where:
        - nodes: list of dicts with 'id', 'kind', and optional 'metadata'
        - edges: list of dicts with 'id', 'src', 'dst', and 'kind'
    """
    ...
