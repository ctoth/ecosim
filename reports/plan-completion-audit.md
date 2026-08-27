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
| 7 — C/N/P/energy | `multikind_fixture.rs` declares one kind per scalar channel, separate C/N/P certificates, and explicit energy input/heat/work/export ports. Tests reject both cross-kind topology and cross-kind sentence construction and expose omitted heat. | Implemented synthetic fixture; no empirical validation claimed |
| 8 — kinetics/allocation | `kinetics.rs` checks producer, feeding, mortality, decomposition, settlement bounds, and common source-limitation scale from pre-state, elapsed time, proposals, and settlements. Generated tests include invalid domains and zero actor/resource cases. | Implemented ecosystem theory |
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

## Remaining publication and clean-source gate

The implementation is not publish-complete. Current public and local heads are:

| Repository | Public `master` | Local `master` |
|---|---|---|
| conservation | `dd21f6b1fee852b62328dac8aaf7138e45551272` | `35dc31b1e228b96d213a6a9229c8c02b7d867413` |
| institution | `16109396f9cc68c705a941b9b039758486597389` | `facdd0b` or a reviewed descendant |
| ecosim | `d752107310d40120b063ecd91c4f938bb3622492` | `HEAD` containing this audit or a reviewed descendant |

Consequently, institution and ecosim still use temporary sibling paths for the
new crates during local integration. Completion requires this order:

1. review and publish conservation;
2. replace institution's conservation paths with the exact published revision,
   verify from a clean source tree, commit, and publish institution;
3. replace ecosim's sibling paths and patches with both exact published
   revisions, verify the sdist/wheel from a clean source tree, commit, and
   publish ecosim;
4. confirm all three public heads equal the locally verified commits.

Publication is intentionally not inferred from local implementation authority.
