# Stock–Flow Satisfaction-Condition Evidence

The implemented fragment uses total bijective, kind-preserving renamings of
stock axes, ledgers, internal flows, boundary ports, and quantity kinds.
`StockFlowRenaming` validates that target effect matrices, boundary roles, and
ledger port sets are exactly the coordinate-renamed source structure.

Sentence translation is covariant. It renames every referenced symbol in
transition, linear-flow, boundary-correspondence, graded-state, and certified
open-balance sentences. Model reduct is contravariant: it rebuilds every exact
transition record in canonical source order using inverse maps and revalidates
the result.

For every supported sentence family the two sides

```text
target_model |= translate(sentence)
    iff
reduct(target_model) |= sentence
```

delegate to the same conservation checker over equal rational coordinates.
The generated tests observe signature identity/composition, sentence
identity/composition, model identity/contravariant composition, and both true
and false directions of the satisfaction square. A separate fixture translates
one neutral source sentence into ecological and economic vocabulary.

Reproduction:

```text
cd ../institution
cargo test --workspace --all-targets
cargo clippy --workspace --all-targets -- -D warnings
```

This is an explicitly discrete model-category specialization. It does not yet
claim general aggregation, hiding, unit conversion, or refinement morphisms.
