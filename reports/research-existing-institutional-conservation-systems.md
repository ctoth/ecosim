# Research: Existing Institutional Conservation Systems

## Summary

Most of what we have discussed already exists, but in separate research and software traditions. Hets implements Goguen-style institutions, theories, comorphisms, and heterogeneous proof management. Catlab, StockFlow.jl, and CatColab implement categorical modeling, open-system composition, stock-flow diagrams, Petri nets, and dynamical semantics; importantly, CatColab contains a substantial Rust categorical-logic core named `catlog`. Reaction-network systems compute conservation laws from stoichiometric nullspaces. Variational mechanics provides the symmetry-to-momentum theorem. A recent Lean development even makes institutional satisfaction executable and kernel-checked.

The search did not identify one system that combines all of these into an executable, typed, proof-provenance-preserving conservation institution usable by both economic and ecological simulations. That combination is what this project would add. It is best described as a **proof-carrying institutional semantics and runtime for typed open dynamical systems**, not as another generic category-theory library and not merely as another simulator.

The strongest existing foundation candidate is CatColab's `catlog`, not the small general-purpose Rust category crates. Before creating a separate category-theory library, we should test whether `catlog` can host the signature/model side of the proposed institution while we add sentences, satisfaction, conservation derivations, and certificates. `cargo info catlog` currently reports that the package is not in the crates.io registry, so reuse would require a Git dependency, an upstream package split/publication, or a separately negotiated architectural boundary.

## Approaches Found

### Goguen Institutions and Heterogeneous Specification

**Sources:** [Goguen and Burstall](https://publish.lfcs.inf.ed.ac.uk/reports/90/ECS-LFCS-90-106/), [Hets](https://github.com/spechub/Hets)

**Description:** Institution theory packages signatures, sentence translation, model reduct, and satisfaction, with truth invariant under changes of notation. Hets implements a graph of logics and logic translations, treats translations as first-class, structures theories through development graphs, and connects multiple theorem provers.

**Pros:** This is the established answer to `Sign`, `Sen`, `Mod`, satisfaction, comorphisms, and heterogeneous proof borrowing. Hets has real implementations of multiple institutions and translations rather than merely defining interfaces.

**Cons:** Hets is a large GPL-licensed Haskell specification/proof-management system. It does not provide a small Rust runtime for stock/flow simulation, numerical traces, conservation-law derivation, or event certificates.

**Complexity:** High.

### Categorical Scientific Modeling: Catlab and AlgebraicJulia

**Sources:** [Catlab](https://github.com/AlgebraicJulia/Catlab.jl), [AlgebraicJulia](https://www.algebraicjulia.org/), [StockFlow.jl](https://github.com/AlgebraicJulia/StockFlow.jl)

**Description:** Catlab implements applied/computational category theory using generalized algebraic theories, categorical algebra, attributed C-sets, rewriting, diagrams, and functorial data migrations. StockFlow.jl represents and composes stock-flow diagrams and gives them ODE semantics. AlgebraicPetri and AlgebraicDynamics cover Petri nets and compositional dynamical systems.

**Pros:** This is already the categorical, compositional modeling tradition we independently approached. It cleanly separates model syntax from semantics and handles open systems, composition, stratification, and multiple dynamical interpretations.

**Cons:** Catlab explicitly says it is not a theorem prover or proof assistant and does not produce formal correctness certificates. Its GAT axioms have historically been the programmer's responsibility. It is Julia-centered and does not supply Goguen-style satisfaction or trace-level conservation witnesses.

**Complexity:** High, but mature in the relevant mathematical direction.

### CatColab and the Rust `catlog` Core

**Sources:** [CatColab](https://github.com/ToposInstitute/CatColab), [`catlog` developer documentation](https://next.catcolab.org/dev/rust/catlog/dbl/), [`DblModel` source documentation](https://next.catcolab.org/dev/rust/src/catlog/dbl/model.rs), [DoubleTT RFC](https://next.catcolab.org/rfc/0002)

**Description:** CatColab is a formal, interoperable conceptual-modeling application built around double-categorical logic. Its Rust `catlog` package implements ordinary and double category theory, double theories, models, model morphisms, diagrams, type-theoretic elaboration, simulation, and standard model logics. CatColab supports Petri nets, stock-flow diagrams, polynomial ODEs, mass-action analyses, stochastic simulation, and composition by identifying shared stocks or places.

**Pros:** This is the closest software match. It is Rust, dual MIT/Apache-2.0, explicitly separates theories from models, and already contains the categorical machinery and dynamical model forms we would otherwise have to recreate. Its architecture is based on active research by specialists in categorical logic and compositional modeling.

**Cons:** `catlog` is a package inside the CatColab monorepo and is not currently published on crates.io. Its documented core is double-theoretic categorical logic, not a Goguen institution with explicit `Sen`, `Mod`, `|=`, and satisfaction-condition proofs. CatColab is primarily a collaborative modeling application, while our target is an embeddable exact derivation/runtime kernel with Python bindings and domain adapters.

**Complexity:** High internally; potentially medium to extend if the abstractions align.

### Executable and Certified Institutions

**Source:** [Goodman and Veselov, “Truth Invariant Under Change of Notation: A Certified Category of Logics”](https://www.researchgate.net/publication/405835750_Truth_Invariant_Under_Change_of_Notation_A_Certified_Category_of_Logics)

**Description:** This 2026 preprint constructs institutions and comorphisms in Lean 4, adds an operational institution whose satisfaction relation executes compiled code, and treats satisfaction-condition proofs as programs. It also requires non-vacuity/discrimination results so that trivial satisfaction relations cannot masquerade as meaningful institutions.

**Pros:** It directly validates the idea of executable satisfaction and proof-carrying translations. Its separation between denotational truth, executable checking, and kernel-checked preservation is highly relevant.

**Cons:** It is a proof-of-concept logic development, not a production simulation or conservation library, and it does not address typed open dynamical systems, economics, ecology, stoichiometry, or variational mechanics.

**Complexity:** High if adopted as a proof-assistant foundation; moderate as a design reference.

### Compositional Open Dynamical and Reaction Networks

**Sources:** [Baez and Pollard](https://arxiv.org/abs/1704.02051), [structured cospans](https://arxiv.org/abs/1911.04630), [categorical stock-flow modeling](https://arxiv.org/abs/2211.01290)

**Description:** Open reaction networks are morphisms composed by gluing boundaries; functors map reaction-network syntax to open dynamical-system semantics. Structured cospans generalize this compositional construction. Categorical stock-flow work applies the same ideas to system dynamics.

**Pros:** This supplies the correct categorical account of boundaries, composition, and multiple semantics. It applies naturally to ecosystems and to economic stock-flow models.

**Cons:** It does not itself define satisfaction, derive all conservation laws with proof provenance, or check runtime ledger events. Composition can preserve model structure without certifying every desired law.

**Complexity:** High mathematically; implemented substantially in AlgebraicJulia and CatColab.

### Structural Conservation Analysis

**Sources:** [libStructural paper](https://pmc.ncbi.nlm.nih.gov/articles/PMC6051435/), [Catalyst conservation-law API](https://docs.sciml.ai/Catalyst/dev/api/network_analysis_api/), [Macaulay2 ReactionNetworks](https://macaulay2.com/doc/Macaulay2/share/doc/Macaulay2/ReactionNetworks/html/index.html)

**Description:** Reaction-network packages compute stoichiometric structure, conserved moieties, steady-state equations, and left-nullspace conservation laws. Petri-net P-invariants encode the same structural idea.

**Pros:** The exact nullspace-to-conservation derivation is established and implemented. We should reuse its mathematics and use existing systems as conformance oracles.

**Cons:** These tools are domain-specific. They do not unify accounting incidence, open boundary balances, variational Noether laws, dimensions, institutional translations, and runtime event evidence.

**Complexity:** Low to medium for linear laws; higher for nonlinear or approximate invariants.

### Rust Category-Theory Libraries

**Sources:** [`lau-category-theory`](https://github.com/SuperInstance/lau-category-theory), [`comp-cat-rs`](https://github.com/MavenRain/comp-cat-rs), [Karpal](https://github.com/Industrial-Algebra/Karpal), [`categories`](https://github.com/TheMesocarp/categories)

**Description:** Several Rust packages implement categories, functors, natural transformations, monads, arrows, or related categorical constructions.

**Pros:** They demonstrate viable Rust encodings and offer reusable pieces. Current crate metadata reports MIT for `lau-category-theory` and `comp-cat-rs`, Apache-2.0 for Karpal 0.9, and AGPL-3.0 for `categories` 0.1.

**Cons:** They solve different problems. `lau-category-theory` represents explicit finite categories and checks laws at runtime. Karpal and `comp-cat-rs` emphasize higher-kinded functional-programming abstractions/effects. `categories` describes itself as work in progress. None found in this search implements Goguen institutions. None is an obvious foundation for `Mod : Sign^op -> Cat` merely because it defines a Rust trait named `Category` or `Functor`.

**Complexity:** Low to adopt superficially; potentially high if their encodings do not match indexed institution semantics.

## Key Papers

- [Goguen and Burstall (1992)](https://publish.lfcs.inf.ed.ac.uk/reports/90/ECS-LFCS-90-106/) — Defines institutions and truth invariant under change of notation.
- [Goguen and Roşu (2002)](https://ntrs.nasa.gov/citations/20010097127) — Systematizes institution morphisms, comorphisms, and theoroidal variants.
- [Mossakowski et al., “Heterogeneous Theories and the Heterogeneous Tool Set”](https://github.com/spechub/Hets) — Describes the implemented institution/comorphism logic graph behind Hets.
- [Baez and Pollard (2017)](https://arxiv.org/abs/1704.02051) — Makes open reaction networks compositional and maps them functorially to open dynamical systems.
- [Baez, Courser, and Vasilakopoulou (2020)](https://arxiv.org/abs/1911.04630) — Develops structured cospans for compositional open systems.
- [Baez et al. (2022)](https://arxiv.org/abs/2211.01290) — Gives categorical syntax, semantics, composition, and stratification for stock-flow diagrams.
- [Marsden and West (2001)](https://doi.org/10.1017/S096249290100006X) — Provides the discrete variational and Noether foundation for symmetry-derived laws.
- [Goodman and Veselov (2026)](https://www.researchgate.net/publication/405835750_Truth_Invariant_Under_Change_of_Notation_A_Certified_Category_of_Logics) — Demonstrates executable, kernel-checked institutional satisfaction and translations.

## Existing Implementations

- **Hets** ([repository](https://github.com/spechub/Hets)): Haskell/GPL-2.0-or-later; institutions, logic translations, development graphs, and proof-tool integration.
- **Catlab.jl** ([repository](https://github.com/AlgebraicJulia/Catlab.jl)): Julia/MIT; computational applied category theory, GATs, categorical algebra, ACsets, and functorial data migrations.
- **StockFlow.jl** ([repository](https://github.com/AlgebraicJulia/StockFlow.jl)): Julia/MIT; categorical stock-flow construction, execution, composition, and stratification.
- **CatColab / `catlog`** ([repository](https://github.com/ToposInstitute/CatColab)): Rust plus TypeScript, MIT or Apache-2.0; double theories, models, categorical logic, stock-flow/Petri/ODE model support, simulation, and collaborative composition. `catlog` is not currently found in crates.io by `cargo info catlog`.
- **Catalyst.jl** ([documentation](https://docs.sciml.ai/Catalyst/dev/api/network_analysis_api/)): Julia; reaction networks and computed conservation-law matrices.
- **libStructural** ([paper](https://pmc.ncbi.nlm.nih.gov/articles/PMC6051435/)): portable reaction-network structural analysis and mass-conservation facilities.
- **Macaulay2 ReactionNetworks** ([documentation](https://macaulay2.com/doc/Macaulay2/share/doc/Macaulay2/ReactionNetworks/html/index.html)): symbolic steady-state and conservation equations for reaction networks.
- **Rust category packages:** `lau-category-theory` 0.1.0, `comp-cat-rs` 0.5.1, Karpal 0.9.0, and `categories` 0.1.0. These are prior art for encodings, not identified institution implementations.

## Complexity vs Quality Tradeoffs

| Choice | What it gains | What it risks |
|---|---|---|
| Build generic category theory from scratch | Total control over Rust types | Reinventing `catlog`, Catlab, proof-assistant libraries, and subtle coherence machinery |
| Use a small crates.io CT crate | Immediate `Category`/`Functor` names | A programming functor or finite-category container may not model institution-indexed `Sen` and `Mod` correctly |
| Extend CatColab `catlog` | Existing Rust categorical logic, theories/models, open-system model forms, permissive license | Unpublished monorepo package; double-theory semantics may not expose the exact institution seams we need |
| Reuse Hets directly | Mature institution/comorphism tooling | Haskell/GPL application architecture and theorem-prover workflow do not fit an embedded Rust/Python runtime |
| Build a narrow institution/conservation layer, interoperating with `catlog` and Hets | Exact semantics and runtime fit without rebuilding all categorical modeling | Requires carefully specifying the boundary and proving translations |
| Formalize everything in Lean first | Strongest theorem trust | Large proof effort before a useful simulation runtime; extraction/interoperation design remains |

The highest-quality path is not “maximum new abstraction.” It is to reuse the categorical structures whose semantics match and add only the missing institutional and conservation structure. A dependency is not principled merely because its types have categorical names; it must preserve the exact mathematical objects and laws needed by the institution.

## Recommendations

### What we are building

Build an **executable conservation institution for typed open dynamical systems** with four connected roles:

1. **Institutional semantics:** signatures, signature morphisms, sentences, models, satisfaction, theories, and satisfaction-preserving translations.
2. **Law compiler:** derive balance laws from exact incidence/stoichiometric nullspaces and derive momentum laws from genuine variational symmetries; record distinct provenance.
3. **Satisfaction runtime:** decide ground finite-trace sentences, emit witnesses or counterexamples, and distinguish mathematical derivation certificates from numerical/event evidence.
4. **Domain theories:** express economic and ecosystem theories over the same typed open-process institution, with particular simulations as models.

This can be summarized as:

> Hets-style truth preservation applied to CatColab-style compositional dynamical models, with Catalyst-style conservation derivation and a proof-carrying runtime.

It is not primarily:

- a new general category-theory library;
- a replacement for CatColab or StockFlow.jl;
- a general-purpose theorem prover;
- a single ecological simulator;
- a claim that all conservation is Noetherian.

### Architectural decision

Do not create the proposed generic category-theory library yet. First perform a compatibility design against `catlog`:

1. represent a typed open-process signature and model using `catlog`'s double theory/model APIs;
2. define a balance sentence and executable satisfaction externally;
3. define a conservative rename and demonstrate the Goguen satisfaction square;
4. test whether sentence translation and model reduct can be expressed naturally without distorting `catlog`;
5. determine whether `catlog` can become a published upstream dependency or whether its small ordinary-category substrate should be separated upstream.

If this succeeds, the new library should be the institution/conservation layer above `catlog`. If it fails for a mathematical reason—rather than packaging inconvenience—then create the smallest standalone institution foundation required by the failed proof obligation. That is a principled extraction criterion.

### Preserve the novel boundary

The project's distinctive contribution should remain explicit:

- quantity-kind and dimension authority at signatures;
- open boundaries and forced balances as first-class syntax;
- multiple honest law origins (`Noether`, `StoichiometricNullspace`, `IncidenceNullspace`, `ForcedBalance`, `Declared`);
- exact derivation followed by tolerance-aware trace satisfaction;
- satisfaction-preserving domain renaming and, later, proved aggregation/lumping;
- Rust execution with thin Python bindings and adapters to Bridgman, Tally, CatColab/StockFlow formats, and reaction-network formats.

## Estimated Implementation Effort

- **Compatibility investigation:** small to moderate. It produces a worked `catlog` encoding, a satisfaction-square proof sketch, and a go/no-go dependency decision.
- **Minimal useful institution:** moderate. Conservative signature morphisms, balance sentences, raw trace models, executable satisfaction, exact nullspace derivation, and witness/counterexample records.
- **Domain profiles:** moderate. Economics and ecosystems share the core, but boundary and quantity-kind choices require domain-specific care.
- **Noether fragment:** moderate to high. A real discrete variational signature, symmetry premise, momentum-map conclusion, and forced balance should be derived rather than declared.
- **Certified translations and aggregation:** high. Nontrivial comorphisms, lumpability, hiding, error semantics, and proof-assistant checking are separate maturity stages.

## Open Questions

- [ ] Can `catlog` express our `Sign`, sentence translation, and contravariant model reduct without an unnatural wrapper?
- [ ] Will the CatColab maintainers publish `catlog` independently or accept a reusable package boundary?
- [ ] Is double-categorical logic the correct base for the institution, or should it be one model-form institution connected by a comorphism?
- [ ] Should the satisfaction-condition proofs live in Lean while Rust carries checked proof artifacts, or begin as executable law tests plus paper proofs?
- [ ] Which CatColab/StockFlow interchange representation preserves process identity, quantity kinds, and boundaries?
- [ ] Does a generic sentence language need temporal operators, or are transition and finite-trace assertions sufficient initially?
- [ ] What is the precise mathematical abstraction for aggregation: institution comorphism, theoroidal comorphism, or a separate lumping/refinement relation?

## References

- Goguen, J. A., and Burstall, R. M. (1992). “Institutions: Abstract Model Theory for Specification and Programming.” [LFCS report](https://publish.lfcs.inf.ed.ac.uk/reports/90/ECS-LFCS-90-106/).
- Goguen, J. A., and Roşu, G. (2002). “Institution Morphisms.” [NASA record](https://ntrs.nasa.gov/citations/20010097127).
- Mossakowski, T., Maeder, C., Lüttich, K., and Wölfl, S. “Heterogeneous Theories and the Heterogeneous Tool Set.” [Hets](https://github.com/spechub/Hets).
- Patterson, E., and collaborators. CatColab and `catlog`. [Repository](https://github.com/ToposInstitute/CatColab) and [developer documentation](https://next.catcolab.org/dev/).
- Catlab and AlgebraicJulia contributors. [Catlab](https://github.com/AlgebraicJulia/Catlab.jl) and [AlgebraicJulia](https://www.algebraicjulia.org/).
- Baez, J. C., and Pollard, B. S. (2017). “A Compositional Framework for Reaction Networks.” [arXiv](https://arxiv.org/abs/1704.02051).
- Baez, J. C., Courser, K., and Vasilakopoulou, C. (2020). “Structured Cospans.” [arXiv](https://arxiv.org/abs/1911.04630).
- Baez, J. C., Li, X., Libkind, S., Osgood, N. D., and Redekopp, E. (2022). “A Categorical Framework for Modeling with Stock and Flow Diagrams.” [arXiv](https://arxiv.org/abs/2211.01290).
- Goodman, R., and Veselov, V. (2026). “Truth Invariant Under Change of Notation: A Certified Category of Logics.” [Preprint](https://www.researchgate.net/publication/405835750_Truth_Invariant_Under_Change_of_Notation_A_Certified_Category_of_Logics).
- Sauro, H. M., et al. (2018). “A Portable Structural Analysis Library for Reaction Networks.” [PMC](https://pmc.ncbi.nlm.nih.gov/articles/PMC6051435/).
