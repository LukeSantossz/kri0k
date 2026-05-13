"""Smoke tests for kri0k Python package."""

import kri0k


def test_hello() -> None:
    """Test hello() function from Rust core."""
    result = kri0k.hello()
    assert isinstance(result, str)
    assert "kri0k" in result.lower()
    assert len(result) > 0


def test_get_dummy_graph_structure() -> None:
    """Test get_dummy_graph() returns valid structure."""
    graph = kri0k.get_dummy_graph()
    
    # Verify top-level structure
    assert isinstance(graph, dict)
    assert "nodes" in graph
    assert "edges" in graph
    
    # Verify nodes structure
    nodes = graph["nodes"]
    assert isinstance(nodes, list)
    assert len(nodes) > 0
    
    # Check first node has required fields
    node = nodes[0]
    assert "id" in node
    assert "kind" in node
    assert isinstance(node["id"], str)
    assert isinstance(node["kind"], dict)
    
    # Verify edges structure
    edges = graph["edges"]
    assert isinstance(edges, list)
    
    if len(edges) > 0:
        edge = edges[0]
        assert "id" in edge
        assert "src" in edge
        assert "dst" in edge
        assert "kind" in edge


def test_graph_node_kinds() -> None:
    """Test that graph contains expected node types."""
    graph = kri0k.get_dummy_graph()
    nodes = graph["nodes"]
    
    # Check we have at least one host node
    kinds = [node["kind"]["type"] for node in nodes if "type" in node["kind"]]
    assert "host" in kinds or "network" in kinds or "service" in kinds
