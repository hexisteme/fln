from fln import MerkleNode, merkle_root


def test_node_hash_is_deterministic():
    node = MerkleNode(payload=b"thesis-A")
    assert node.hash() == node.hash()


def test_merkle_root_handles_odd_leaves():
    leaves = [bytes([1]) * 32, bytes([2]) * 32, bytes([3]) * 32]
    root = merkle_root(leaves)
    assert root is not None
    assert root != bytes(32)


def test_merkle_root_empty_is_none():
    assert merkle_root([]) is None
