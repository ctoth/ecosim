# Review Task: Repository Architecture for Institutional Conservation Systems

## Context

We are deciding how to organize a new Rust-based formal systems project that will be shared by ecological and economic simulations. The intended mathematical separation is:

- category theory and Goguen-style institutions describe signatures, sentences, models, satisfaction, and truth-preserving translations;
- conservation theory describes balances, invariants, exact derivations, provenance, and trace evidence;
- open systems describe typed boundaries, compositional processes, and dynamical models;
- ecosystems and economies are domain theories/models using those neutral layers;
- Bridgman remains an independent quantity/dimension library;
- Tally is an existing ledger system under the Plenty economics repository.

Conservation itself must not inherently depend on institution theory or open-system machinery. Instead, a bridge interprets conservation laws as institutional sentences over suitable models.

The current proposal is to create two repositories now:

```text
categorical/
  crates/
    category/
    institution/
    institution-translation/
    institution-testing/
    catlog-adapter/

conservation/
  crates/
    conservation-core/
    conservation-linear/
    conservation-noether/
    conservation-trace/
    conservation-institution/
    conservation-bridgman/
    conservation-python/
```

Existing domain repositories would consume these:

```text
ecosim  ----> conservation
plenty  ----> conservation
bridgman <--- conservation-bridgman
tally   <---- Plenty-owned adapter
```

A possible third `open-systems` repository is deferred until research determines whether the required model structures should be reused from CatColab's Rust `catlog`, expressed behind a small neutral interface, or implemented independently. The research summary is in `reports/research-existing-institutional-conservation-systems.md`.

## Review Standard

Review for mathematical fidelity first, not implementation convenience. Do not reward fewer repositories merely for simplicity, or more repositories merely for conceptual purity. Distinguish crate boundaries from repository boundaries and identify cyclic or inverted dependencies.

## Questions

1. Is the separation between `categorical`, `conservation`, and the deferred `open-systems` responsibility mathematically coherent?
2. Should the generic Institution crate live in a broader category-theory repository, or should institutions have their own repository?
3. Is two repositories now the correct boundary, or should we create one or three? Give a concrete dependency graph.
4. Where should `conservation-institution` live, and which direction should adapter dependencies point?
5. Should `open-systems` be deferred pending the `catlog` compatibility study, or is it already a sufficiently independent abstraction to own now?
6. Which proposed crates are premature, incorrectly named, or combine distinct proof obligations?
7. What is the smallest theorem-shaped vertical slice that validates the architecture without biasing the result toward a particular simulator?
8. Give a final recommendation and identify any disagreement with the current proposal.

## Output

Return a concise but substantive architecture review with:

- verdict;
- corrected repository and crate graph;
- principal risks;
- recommended first vertical slice;
- explicit points of agreement or disagreement.

Do not edit the workspace or create implementation artifacts.
