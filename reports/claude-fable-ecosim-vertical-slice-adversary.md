# Adversarial Review: ecosim exact energy-ledger vertical slice

Static adversarial review of the uncommitted changes across the `conservation`,
`institution`, and `ecosim` repositories, against the intent in
`prompts/claude-fable-ecosim-vertical-slice-adversary.md`. Dynamic checks
(all three Rust workspaces, Python tests, Clippy, Pyright, wheel, sdist, and an
isolated wheel rebuild) were verified independently and are relied on here; this
report covers the semantic questions those runs cannot answer.

## Findings by attack surface

### 1. False or tautological conservation claim — not found

The law is genuinely derived, not declared. `ecosim_core::energy_law` builds a
3-axis × 6-transition `TransitionMatrix` and calls
`conservation_linear::derive_left_nullspace`, which performs exact Gauss–Jordan
elimination over the transposed matrix in `BigRational`, extracts the
free-column basis, and scales each vector to primitive integers with a
positive-leading-sign normalization. Verified by hand: the six columns (transfer
L→R, transfer R→L, input L, input R, output L, output R) have rank 2 over the
axes `{left, right, net_external}`, so the left nullspace is one-dimensional and
its normalized basis vector is exactly `(1, 1, −1)` — the intended law
`left + right − net_external = constant`. `TransitionMatrix::new` sorts rows by
axis after zipping axis names with their rows, so the reordering to
alphabetical `(left, net_external, right)` keeps each row attached to its axis;
the derived coefficients are keyed by `AxisId` and cannot be misassigned.
`energy_law` guards `laws.len() != 1` with `UnexpectedLawCount`, so a wrong
matrix could not silently yield a weaker or vacuous law set.

The trace check is not a restatement of the ledger. The ledger residual
(`initial + inputs − outputs − final`) constrains only the endpoint totals,
while `conservation_trace::check_trace` evaluates the law's weighted sum at
**every** state in the trace and reports the first index whose balance differs
from the initial state's. It errors (`TooShort`) on traces shorter than two
states and (`MissingAxis`) on any state omitting a law axis, so it cannot pass
vacuously. `Provenance`/`NullspaceSource::Stoichiometric` is explicitly
documented as origin metadata, not a correctness certificate — no circularity.

### 2. Wrong matrix, sign convention, nullspace, or ledger equation — not found

Sign conventions verified per column: transfers move stock between `left` and
`right` with `net_external` unchanged; inputs add `+1` to a compartment and
`+1` to `net_external`; outputs subtract from both. `World::net_external()`
returns `inputs − outputs`, matching the axis semantics recorded in each
`TraceState`. The `BalanceReport` residual is computed exactly as
`initial + inputs − outputs − final`, is tracked from independent cumulative
counters rather than derived from the trace, and `is_balanced()` tests exact
zero in `BigRational` — no tolerance anywhere.

### 3. Event failure partially mutating state — not found

In `transfer`, `input`, and `output`, all validations (`ensure_nonnegative`,
same-compartment, `ensure_available`) complete before any field is touched;
mutation then happens unconditionally and `append_state` follows. The only
theoretical post-mutation failure point is `TraceState::new` inside
`append_state`, but its two error cases (empty state, duplicate axis) are
unreachable for the three fixed, distinct, non-blank axis names — noted as an
observation, not a defect. The `rejected_events_leave_the_world_unchanged` test
asserts whole-`World` equality after each rejected event, covering trace length
as well as stocks and counters.

### 4. Satisfaction not delegated to the Institution adapter — not found

`World::satisfies_energy_law` constructs a `ConservationSignature`, wraps the
complete trace in `TraceModel::new`, and calls
`ConservationInstitution.satisfies(&signature, &model, &law)` through the
`Institution` trait — which delegates to `conservation_trace::check_trace` over
the full trace. The residual check and the institution check are asserted
together in both the example-run test and the property test, satisfying the
prompt's requirement that the residual alone is not the evidence.

### 5. Vacuous trace or property tests — not found

The Rust proptest generates 1–64 events across six kinds with amounts up to
9,999, pairing withdrawals with a preceding input so sequences remain valid,
and asserts both `report().is_balanced()` and `satisfies_energy_law()`. Since
`history_len` grows with every accepted event, the institution check runs over
genuinely long traces, and `check_trace`'s per-state loop makes it sensitive to
any intermediate imbalance. The Python Hypothesis test mirrors this structure
against the compiled extension. The adapter's own tests moved verbatim (import
path only), preserving its established violated/satisfied coverage.

### 6. Duplicate or inverted cross-repository dependencies — not found

Dependency direction is strictly `conservation` ← `institution-conservation`
(in the `institution` workspace) ← `ecosim-core` ← `ecosim-python`. No
conservation crate depends on institution theory, and the updated conservation
`README.md` states the boundary correctly: core/linear/trace are independent of
institution models and Bridgman, and the executable adapter now lives
downstream as `institution-conservation`. Both `institution-conservation` and
`ecosim-core` pin the conservation crates at the same git revision `7a06538…`,
and the ecosim lockfile unifies them into single package instances — no
type-identity split. Bridgman is consumed at pinned revision `984c6e8…` in both
`pyproject.toml` and `uv.lock`, and the exported `ENERGY_DIMENSIONS`
(M L² T⁻²) is asserted by test. The independently verified isolated wheel
rebuild confirms the pinned git revisions resolve outside a warm cache.

### 7. Python/Rust integer conversion bugs or misleading API contracts — not found

The facade is `BigInt` end-to-end; the `exact_integer` guard raises
`PyRuntimeError` on any fractional value escaping to Python, and since every
Python-reachable operation is integer-only, the `.pyi` contract of `int` is
sound. Invalid events raise `ValueError` and are tested (same-compartment,
insufficient energy, negative amount, unknown compartment name), including the
assertion that history length is unchanged after each rejection.

### 8. Generated artifacts, research-file inclusion, broken sdist — acknowledged condition only

`[tool.maturin]` explicitly excludes `src/ecosim/*.pdb` from sdist and wheel
and `src/ecosim/*.pyd` from the sdist, and the independently verified sdist
contains no generated `.pyd`/`.pdb`. The sdist does vendor the sibling
`institution` path dependency; per the prompt this is the acknowledged
pre-publication release-order condition, and nothing about the design would
remain wrong once the path is replaced with an immutable git revision — the
dependency direction, revision unification, and adapter semantics are already
in their final shape. Maturin's manifest-driven sdist (cargo package list plus
`python-source` plus the two explicit includes) keeps `papers/`, `notes-*.md`,
`prompts/`, and `reports/` out of the distribution.

## Non-blocking observations

- **Repository hygiene, not packaging:** `.gitignore` covers build outputs but
  not `papers/` (~24 MB of paper directories), `notes-*.md`, `prompts/`, or
  `reports/`. The repo has zero commits, so a bare `git add .` would commit all
  research material. This never reaches the sdist or wheel, so it is not a
  packaging defect — but worth deciding deliberately before the first commit.
- `append_state` runs after mutation; its failure modes are unreachable with
  the fixed axis set, but if axes ever become dynamic this ordering would need
  revisiting.
- The Rust property test only generates valid sequences (by construction); the
  violated-trace path is covered by the adapter's moved tests and the explicit
  rejection tests rather than property-based generation. Adequate for this
  slice.

## Verdict

Every attack surface named in the prompt was probed and none produced a
violated invariant or reproducible failure. The conservation claim is derived
and independently checked per-state, events are atomic, satisfaction is
genuinely delegated through the Institution adapter, tests are non-vacuous,
dependencies point one way at one pinned revision, and the packaging condition
is exactly the one the prompt pre-acknowledged.

MERGE
