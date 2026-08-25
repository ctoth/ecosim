# Adversarial Review: ecosim exact energy-ledger vertical slice

Review the uncommitted changes in these three repositories:

- the sibling `conservation` repository
- the sibling `institution` repository
- this `ecosim` repository

The intended change is narrowly defined:

1. Move the conservation Institution adapter out of `conservation` and into
   `institution` under the package name `institution-conservation`, without
   changing its established semantics.
2. Keep conservation-core, conservation-linear, and conservation-trace
   independent of institution theory.
3. Build `ecosim` as a mixed Rust/Python Maturin package managed with uv.
4. Its first model has two internal energy compartments plus cumulative net
   external flow. The supported transition matrix must derive exactly the law
   `left + right - net_external = constant`.
5. Accepted transfers, inputs, and outputs must be atomic, nonnegative, and
   exactly accounted. The ordinary ledger identity is
   `initial + inputs - outputs = final + residual`.
6. The complete trace must be checked as a model through
   `institution-conservation`; the residual check alone is not sufficient.
7. Rust and Python property tests should exercise arbitrary valid event
   sequences. Bridgman must be consumed at its pinned Git revision for the
   public energy dimension.

The institution adapter is temporarily a sibling path dependency because its
move has not been published. Maturin therefore vendors it into the current
sdist. Treat this as an acknowledged release-order condition, but block if the
result would remain architecturally or semantically wrong after replacing the
path with an immutable Git revision.

Review the actual diffs and relevant surrounding code. Look especially for:

- a false or tautological conservation claim;
- a wrong transition matrix, sign convention, nullspace, or ledger equation;
- event failure that partially mutates state;
- satisfaction that is not actually delegated to the Institution adapter;
- vacuous trace or property tests;
- duplicate or inverted cross-repository dependencies;
- Python/Rust integer conversion bugs and misleading API contracts;
- generated artifacts, accidental research-file inclusion, or broken sdists;
- missing cases that invalidate this first vertical slice.

Do not edit any source file. Write only
`reports/claude-fable-ecosim-vertical-slice-adversary.md`.
End with exactly one verdict: `MERGE` or `BLOCK`. A blocker must identify a
specific violated invariant or reproducible failure; keep future feature ideas
separate and non-blocking.
