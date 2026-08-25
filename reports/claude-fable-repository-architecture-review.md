# Claude Fable Architecture Review

Model: Claude Fable

## Verdict

Conditionally approved. The layering direction is mathematically correct: conservation is a self-contained semantic theory, institutions provide a presentation-invariance layer, and a one-way bridge connects them. The proposed contents need correction:

1. Do not create a generic category crate now; that contradicts the catlog compatibility investigation and risks reinventing categorical machinery.
2. Put the conservation–institution bridge in the institution repository, so the conservation repository remains structurally independent.
3. Defer crates whose proof obligations or APIs do not yet exist.

Two repositories are appropriate, but they should be repartitioned.

## Corrected repository graph

    institution/
      crates/
        institution
        institution-testing
        institution-conservation

    conservation/
      crates/
        conservation-core
        conservation-linear
        conservation-trace
        conservation-bridgman

Deferred until explicit triggers:

- category and catlog-adapter: outcome of the catlog compatibility study;
- institution-comorphism: creation of a second institution;
- conservation-noether: start of the discrete variational fragment;
- conservation-python: stabilization of the vertical-slice API;
- open-systems repository: result of the catlog study.

Dependencies remain one-directional:

    conservation-linear -> conservation-core <- conservation-trace
    conservation-bridgman -> conservation-core, bridgman
    institution-testing -> institution
    institution-conservation -> institution, conservation-core,
                                conservation-linear, conservation-trace

At repository level, institution depends on conservation through the bridge only. Conservation does not depend on institution.

## Findings

### Separation of concepts

Conservation, institutions, and open systems are mathematically independent:

- conservation laws are semantic facts about models;
- institutions express invariance of truth under changes of notation;
- open systems describe typed boundaries and composition.

Forced balances couple conservation to open boundaries. Conservation-core should retain ForcedBalance as an abstract provenance tag without importing boundary machinery. A later open-systems-side bridge should provide the actual boundary interpretation.

### Institution repository ownership

The institution should own the repository. A general category library is not yet justified. An institution needs a category of signatures, sentence and model functors, and satisfaction, but Rust can initially express these as the traits required by the institution rather than as a universal Category/Functor hierarchy.

Signature morphisms belong in institution core because the satisfaction condition is stated over them. Morphisms and comorphisms between different institutions may become a separate crate after a second institution exists.

### Bridge ownership

Rename conservation-institution to institution-conservation and place it in the institution repository. Interpreting conservation laws as sentences is an institution constructed over conservation semantics, not a feature intrinsic to conservation.

Adapters depend on the foundations they connect. Neither foundation imports adapter types.

### Open systems

Defer the open-systems repository. This is the area with the greatest overlap with catlog, CatColab, and AlgebraicJulia, so creating it now would prejudge the reuse study. Keep Mod abstract so native or catlog-backed open-system models can later be supplied without redesigning sentences or satisfaction.

## Crate audit

- category: premature; do not create.
- institution: keep; include signature morphisms.
- institution-translation: split conceptually; signature morphisms move into core, inter-institution translations wait.
- institution-testing: keep, initially possibly as a dev-dependency module; test satisfaction squares and non-vacuity.
- catlog-adapter: wait for a successful compatibility study.
- conservation-core: keep; use a small quantity-kind trait rather than a hard Bridgman dependency.
- conservation-linear: keep; use exact integer or rational arithmetic and preserve distinct incidence and stoichiometric provenance.
- conservation-noether: mathematically separate but defer until its proof obligation begins.
- conservation-trace: keep; own traces, residuals, and tolerance-aware witness/counterexample evidence, but not satisfaction.
- institution-conservation: keep in the institution repository.
- conservation-bridgman: keep with dependency toward Bridgman and conservation-core.
- conservation-python: defer until the Rust API stabilizes.

## First theorem-shaped vertical slice

Use a Goguen satisfaction square over a derived balance law:

1. Define one domain-neutral typed flow-network signature with compartments, typed flows, and incidence structure.
2. Derive a conserved quantity from the exact incidence nullspace, recording IncidenceNullspace provenance.
3. Interpret the result as a balance sentence through institution-conservation.
4. Supply a finite trace model; produce a witness for a satisfying trace and a counterexample for a violating trace.
5. Define a nontrivial signature morphism and demonstrate:

       M' satisfies sigma(phi) iff reduct_sigma(M') satisfies phi

6. Instantiate the renaming twice: ecological vocabulary as trophic pools and economic vocabulary as ledger accounts.

The two renamings prevent the first simulator from defining the supposedly neutral abstraction. This slice exercises every retained crate without requiring Noether theory, open-system composition, Python bindings, or catlog.

## Principal risks

1. Rebuilding catlog through a premature generic category crate.
2. Inverting repository dependencies by housing the bridge under conservation.
3. Creating a vacuous institution whose satisfaction relation is never challenged by nontrivial morphisms or counterexamples.
4. Mixing tolerance-aware numerical evidence with exact derivation certificates.
5. Publishing against an unpublished catlog Git dependency.
6. Allowing the first ecological or economic example to bias the shared vocabulary.

## Agreement and disagreement

Agreement:

- create two repositories;
- keep conservation independent of institution theory;
- keep Bridgman independent behind a one-way adapter;
- leave the Tally adapter owned by Plenty;
- defer open systems pending the catlog study;
- keep linear, Noether, and trace proof obligations distinct.

Disagreement:

- replace the proposed categorical repository with an institution repository;
- do not create category yet;
- move and rename conservation-institution;
- include signature morphisms in institution core;
- defer catlog-adapter, conservation-noether, and conservation-python.
