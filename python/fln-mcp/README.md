# fln-mcp

MCP server (stdio) exposing the Falsifier Ledger Network primitives —
thesis lifecycle, Ed25519 signing, Merkle ledger, causal DAG, causal-decay
weighting — to any MCP-aware client (Claude Code, Claude Desktop, IDEs).

```bash
pip install fln-mcp
fln-mcp     # speaks stdio
```

State persists to `$FLN_STATE_DIR` (default `~/.fln`).

## Tools

| name              | purpose                                       |
| ----------------- | --------------------------------------------- |
| `create_thesis`   | New thesis with domain-defaulted τ            |
| `add_falsifier`   | Popper condition + optional deadline          |
| `add_causal_node` | Pearl DAG node (cause/effect/confounder/mediator) |
| `add_causal_edge` | DAG edge (cycle-rejecting)                    |
| `causal_topo`     | Kahn topological order                        |
| `generate_key`    | Ed25519 keypair on disk                       |
| `sign_thesis`     | Sign canonical bytes; returns sig + pubkey hex |
| `append_ledger`   | Add thesis to a named ledger; returns root    |
| `decay_update`    | Apply Causal Decay weight update              |
| `get_thesis`      | Full thesis JSON                              |

## Claude Code registration

```json
{
  "mcpServers": {
    "fln": { "command": "fln-mcp" }
  }
}
```
