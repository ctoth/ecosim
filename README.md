# ecosim

Exact, institution-aware ecosystem simulation with a Rust core and a Python
interface built by Maturin.

The first vertical slice is an energy ledger with two internal compartments
and one cumulative external-flow coordinate. Its transition matrix derives the
exact law

```text
left + right - net_external = constant
```

from a rational left nullspace. Every accepted event appends a trace state,
and the trace is checked through the conservation institution. Consequently,
the ordinary ledger presentation is also exact:

```text
initial_stock + inputs - outputs = final_stock + residual
```

The residual is zero for a satisfying run. Internal transfers change neither
side of that equation.

## Development

```powershell
uv sync
uv run maturin develop
uv run pytest
cargo test --workspace --all-targets
```

During the unpublished cross-repository migration, the Rust workspace uses a
local path to `institution-conservation`. Once that reviewed migration is on
the public repository, the path is replaced by an immutable Git revision like
the existing conservation and Bridgman dependencies.
