"""Kri0k — AI-driven reconnaissance orchestrator.

This package provides Python bindings to the kri0k Rust core.
"""

from kri0k._native import get_dummy_graph, hello

__version__ = "0.1.0"
__all__ = ["hello", "get_dummy_graph"]
