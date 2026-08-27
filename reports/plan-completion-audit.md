# PLAN Completion Audit

This audit maps every work-plan phase to current authoritative artifacts. It
does not treat checklist marks as proof. The status snapshot is local to the
three sibling repositories on 2026-08-27.

| Phase | Authoritative evidence | Audit result |
|---|---|---|
| 0 — formal/source | `stock-flow-formal-specification.md`, `stock-flow-source-verification.md`, and `stock-flow-satisfaction-condition.md` define symbols, well-formedness, exact satisfaction, the open-balance derivation, and conservative renaming; citations are graded against the retrieved sources. | Implemented |
| 1–2 — carrier/sentences | `conservation-stock-flow/src/lib.rs` owns typed identifiers, total exact matrices, immutable transition traces, mandatory carrier-checked sentences, and typed verdicts. `tests/stock_flow.rs` attacks malformed shapes, discontinuity, perturbations, empty traces, and generated exact settlements. | Implemented |
| 3 — Institution | `institution-conservation/src/stock_flow.rs` owns signatures, bijective renamings, sentence translation, reducts, and delegated satisfaction. `tests/stock_flow.rs` exercises identity/composition and true and false satisfaction squares for every family, plus one neutral ecological/economic source theory. | Implemented |
| 4 — entailment | `CheckedNullspace` can be constructed only by exact recomputation against its carrier. The unit-transfer carrier seals incidence provenance internally. `conservation-linear/tests/nullspace.rs` proves basis annihilation, independence, and dimension; stock-flow properties project certified open balances back to graded laws. | Implemented |
| 5–6 — trophic evidence/laws | `trophic_network.rs` persists proposed and settled semantic channels and compiles transition, feeding-partition, input, harvest, graded, and open-balance laws. Rust and Python tests cover atomicity, generated satisfaction, balanced misrouting, wrong partition, dishonest ledgers, wrong harvest-port attribution, and source limitation. | Implemented |
| 7 — C/N/P/energy | `multikind_fixture.rs` declares one kind per scalar channel, separate C/N/P certificates, and explicit energy input/heat/work/export ports. Tests reject both cross-kind topology and cross-kind sentence construction and separately expose omitted heat and omitted export. | Implemented synthetic fixture; no empirical validation claimed |
| 8 — kinetics/allocation | `kinetics.rs` checks producer, feeding, mortality, decomposition, settlement bounds, and common source-limitation scale from pre-state, elapsed time, proposals, and settlements. Generated tests include invalid domains and zero actor/resource cases; an adversarial unit test corrupts a settled withdrawal beyond its proposal and obtains a typed violation. | Implemented ecosystem theory |
| 9 — paired runs | `paired.rs` seals records from complete positive exact run audits. Plan identity, horizon/time grid, and axes are checked while each arm retains its actual intervention schedule. `PairedResponseMetric` makes terminal observation explicit. The Python frozen cascade routes exact sign claims through the Rust sentence evaluator; dense confirmation has no evidence API. | Implemented |
| 10 — economics handoff | `ecosystem-economics-handoff.md` maps implemented types to a prospective accounting client and rejects further extraction until one concrete economic experiment proves identical construction, satisfaction, evidence, and errors. | Decision recorded; economics intentionally not implemented |
| 11 — review/gates | Adversarial attacks cover provenance forgery, false satisfaction, cross-kind construction, wrong partitions and ports, mismatched paired runs, renaming truth/falsity, and exact/dense separation. `reproduction-commands.md` records formatting, test, lint, type, package, diff, and media-history gates. | Locally implemented and verified |

## Definition-of-done audit

- Every accepted exact trophic transition is persisted with its declared
  settled internal and boundary flows and checked against the exact transition
  equation.
- Feeding partitions, per-port ledgers, and derived open balance have separate
  named evidence rather than relying on total balance alone.
- The multi-kind fixture distinguishes conserved material coordinates from an
  open dissipative stored-energy account.
- Constitutive proposals, allocation, and selected exact cascade comparisons
  are inspectable domain sentences with typed witnesses or violations.
- Conservative renamings preserve both true and false satisfaction examples.
- The economics handoff names the next concrete client without extracting an
  unproved shared experiment or kinetic layer.

## Remote revision and clean-source result

The dependency chain uses these immutable published revisions:

| Repository | Published dependency revision |
|---|---|
| conservation | `50f0364473053e8d9f381d53884dcc72cbb2862c` |
| institution | `a75660f5ab711356fbec1ae1c15deb806be2c048` |

Institution resolves every conservation crate from the conservation revision.
Ecosim resolves every conservation and institution crate from the two recorded
Git revisions. No manifest or workspace patch refers to a sibling conservation
or institution checkout; the remaining path from `ecosim-python` to
`ecosim-core` is an internal workspace dependency.

Fresh isolated checkouts were used for the final workspace format, test, and
clippy gates. Ecosim's isolated source additionally passed Python tests,
Pyright, and `uv build`; the latter reconstructed both the source distribution
and wheel from the source distribution. Publication remains subject to the
separate read-before-publish guard and exact remote-head confirmation.
