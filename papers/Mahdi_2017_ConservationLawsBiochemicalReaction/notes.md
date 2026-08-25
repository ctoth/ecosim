---
title: "Conservation Laws in Biochemical Reaction Networks"
authors: "Adam Mahdi, Antoni Ferragut, Claudia Valls, and Carsten Wiuf"
year: 2017
venue: "SIAM Journal on Applied Dynamical Systems"
doi_url: "https://doi.org/10.1137/17M1138418"
---

# Reading notes — Mahdi et al. (2017)

## Pages 1–17

- A reaction network separates a structural stoichiometric representation from parameter-dependent reaction rates. This separation is central: structural conservation laws should not depend on rate constants or a chosen integrator. [PDF pp. 1–3]
- Species are coordinates, reactions contribute net reaction vectors, and the stoichiometric matrix `Gamma` has those vectors as columns. The stoichiometric subspace is the column span; trajectories remain in the nonnegative part of the affine compatibility class `x0 + S`. [PDF pp. 3–6]
- Proposition 7 states the exact linear-algebra contract relevant to the conservation crate: if a row vector `omega` satisfies `omega Gamma = 0`, then `H(x) = omega x` is conserved because `dH/dt = omega Gamma v(x) = 0`. The left kernel has dimension `n - rank(Gamma)`, yielding at least that many independent structural linear laws. [PDF pp. 6–7]
- The qualifier “at least” matters. Examples show kinetic, parameter-dependent linear conservation laws whose coefficient vectors are not in the left kernel, and show laws that exist only at special rate values. The left-nullspace algorithm is complete for parameter-independent structural laws under the paper's stated conditions, not for every first integral of every kinetics. [PDF p. 7]
- A first integral is a non-locally-constant function whose derivative along the vector field vanishes. Darboux theory extends the search beyond linear laws to polynomial, rational, exponential, and product-form integrals. [PDF pp. 8–10]
- Proposition 17 characterizes, structurally and independently of rate parameters, when each coordinate `x_i` is a Darboux polynomial: every reaction that produces species `i` must also consume it. This is outside the current exact linear kernel but indicates a principled nonlinear extension. [PDF pp. 11–13]
- Nonlinear first integrals can imply persistence and stable positive steady states under additional hypotheses. They are not interchangeable with the simple total-ledger invariant, and those dynamical conclusions should not be inferred merely from a linear balance check. [PDF pp. 13–16]
- The generalized Volpert examples combine one structural linear law with a nonlinear Darboux law. Again, the linear left-nullspace layer and nonlinear dynamics layer are complementary rather than competing implementations. [PDF pp. 16–17]

## Implementation obligations

1. For every returned coefficient vector `omega`, test the defining equation `omega * Gamma = 0` exactly.
2. Test that the number of independent returned basis vectors is exactly `rows(Gamma) - rank(Gamma)` for arbitrary exact matrices.
3. Phrase the API as deriving a basis for **structural linear conservation laws**, not all conservation laws.
4. Keep the result independent of rates and numerical integration; a transition is lawful whenever its change vector lies in the represented stoichiometric subspace.
5. Treat nonlinear first-integral discovery as a later, separate layer rather than generalizing the current nullspace API beyond what it proves.

## Pages 18–23

- Further families preserve exactly one parameter-independent linear total while admitting additional nonlinear laws depending on topology, parity, or rate constraints. These examples reinforce the boundary between exact stoichiometric linear algebra and kinetic integrability. [PDF pp. 18–20]
- The paper closes with persistence applications and references. No additional claim needed by the current linear conservation API appears in the reference section. [PDF pp. 20–23]

## Final assessment for this implementation

The conservation kernel's exact left-nullspace direction is paper-faithful if it returns an independent basis, proves each vector annihilates the transition matrix, and describes the result as structural linear conservation. Exact rational arithmetic is stronger for verification than floating approximations. The core should not promise all kinetic or nonlinear first integrals.

## Collection Cross-References

### Already in Collection

- (none found)

### New Leads (Not Yet in Collection)

- Feinberg (1987), "Chemical Reaction Network Structure and the Stability of Complex Isothermal Reactors" - foundational reaction-network structure.

### Supersedes or Recontextualizes

- (none)

### Conceptual Links (not citation-based)

- [Ecopath with Ecosim: Methods, Capabilities and Limitations](../Christensen_2004_EcopathEcosimMethodsCapabilities/notes.md) - Ecopath supplies domain-specific balance equations; Mahdi supplies the exact linear-algebraic criterion and limits it to structural linear invariants.
- [A structure-preserving numerical approach for simulating algae blooms in marine water bodies of western Patagonia](../Almonacid_2020_Structure-preservingNumericalApproachSimulating/notes.md) - instantiates a structural total-mass invariant in an NPZD production-destruction system and preserves it through a positive MPRK step while accounting separately for external input and export.

### Cited By (in Collection)

- (none found)
