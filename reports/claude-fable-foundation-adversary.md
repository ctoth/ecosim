# Claude Fable Adversarial Review: Institution and Conservation Foundations

## Verdict

Conditionally sound. The exact nullspace algorithm is mathematically correct and deterministic for a fixed axis ordering, trace checking is exact, and the Institution trait is a defensible executable slice. Integration should wait until the false-confidence paths below are repaired.

## Blocking findings

1. `BalanceLaw::new` accepts an empty or fully cancelling coefficient set. Its evaluation is zero for every state, allowing a universal but non-substantive conservation witness and a vacuous satisfaction square.
2. A one-state trace returns an ordinary witness even though no transition was checked.
3. Trace checking conflates semantic falsity (`ViolatedBalance`) with malformedness (`EmptyTrace`, `MissingAxis`). The bridge needs semantic violations to become `Ok(false)` and malformed models to become errors.

## Important findings

- Nullspace provenance is caller-selectable metadata rather than a derivation certificate. Copying it into a runtime witness can launder an unverified origin claim into apparent proof evidence.
- The non-vacuity helper only observes both truth values among supplied cases. It does not establish global non-vacuity and can be gamed by chosen or duplicated examples.
- A matrix with axes but no transitions correctly makes every axis invariant mathematically, but is likely a modeling error and should not silently generate a maximal family of laws.
- `TraceState::new` silently uses the final value for duplicate axes.
- Empty axis and kind identifiers are accepted.
- The Institution trait delegates all well-formedness checking to each implementation. Its generic square helper therefore cannot independently prove that translations, reducts, sentences, and models are well formed.
- The Institution slice does not yet encode the category of signatures or functoriality laws. That is acceptable only as an explicit non-claim.

## Nullspace judgment

No arithmetic correctness defect was found in the RREF-based left-nullspace implementation. It uses exact rational arithmetic, deterministic first-nonzero pivots, standard free-variable basis construction, denominator clearing, GCD reduction, and first-nonzero-positive sign normalization. Zero-rank, full-rank, rectangular, rational, dependent-row, and scaling cases were judged mathematically sound. Complexity and size limits are later concerns.

## Required repairs before integration

1. Reject empty identifiers and empty or fully cancelled balance laws.
2. Do not issue an ordinary witness for fewer than two trace states.
3. Separate semantic trace violations from malformed trace/model errors.
4. Treat provenance as unverified origin metadata until a derivation certificate type exists; do not copy it into runtime satisfaction evidence.
5. State the Institution trait's well-formedness preconditions and its category/functoriality non-claims explicitly.
6. Strengthen each concrete institution's malformed-input tests rather than treating the generic square helper as a proof.

## Required bridge test matrix

- Empty and fully cancelling laws are rejected.
- Empty and one-state traces do not yield ordinary witnesses.
- Missing law axes are structural errors, not semantic falsity.
- Duplicate trace axes are rejected.
- Partial, non-bijective, and signature-inconsistent renamings are rejected.
- A target model over the wrong signature is rejected.
- Ecological renaming of a conserving trace yields a square with both sides true.
- Economic renaming of a violating trace yields a square with both sides false.
- A deliberately broken reduct makes the square fail.
- Non-vacuity is observed across those true and false squares, not unrelated examples.
- Zero-transition, dependent-row, rational, and row-permutation nullspace cases have explicit contracts.
- Composed ecological/economic renamings agree with direct renaming.

## Integration judgment

The bridge is feasible without changing the conceptual cores. Use signatures containing an axis set and kind, balance laws as sentences, finite traces as models, and validated bijective axis maps as signature morphisms. Sentence translation renames law axes covariantly; model reduct applies the inverse renaming contravariantly. Satisfaction can be honest only after trace malformedness and semantic violation are separated.

Integration may begin after the three blocking findings are repaired. Provenance sealing must precede any published claim that a runtime witness carries a derivation certificate, but the first slice may use laws actually produced by exact nullspace derivation while labeling provenance honestly as metadata.
