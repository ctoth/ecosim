# Economies, Ecosystems, and a Goguen-Style Noether Institution

## Summary

The precise claim is slightly different from “an economy is an institution”:

- a particular economy or ecosystem is a **model**;
- an economic or ecological modeling formalism is an **institution** when it supplies signatures, sentences, models, satisfaction, and proves the satisfaction condition;
- the reusable conservation kernel can itself be a **Noether institution** into which the economic and ecological institutions are encoded by satisfaction-preserving institution comorphisms;
- alternatively, and more simply for an initial implementation, economics and ecology can be domain theories or profiles inside one shared institution.

This distinction matters. “Institution” in Goguen and Burstall's sense is not an organization or a system in the world. It is an abstract logical system

\[
\mathcal I=(\mathbf{Sign},\operatorname{Sen},\operatorname{Mod},\models).
\]

It consists of:

1. a category `Sign` of vocabularies and vocabulary translations;
2. a covariant sentence functor `Sen : Sign -> Set`;
3. a contravariant model functor `Mod : Sign^op -> Cat`;
4. a satisfaction relation between models and sentences at each signature.

For every signature morphism \(\sigma:\Sigma\to\Sigma'\), sentence \(\varphi\in\operatorname{Sen}(\Sigma)\), and model \(M'\in\operatorname{Mod}(\Sigma')\), these parts must obey the **satisfaction condition**:

\[
M'\models_{\Sigma'}\operatorname{Sen}(\sigma)(\varphi)
\quad\Longleftrightarrow\quad
\operatorname{Mod}(\sigma)(M')\models_\Sigma\varphi.
\]

Sentence translation goes forward; model reduct goes backward. The equation says truth is invariant under a change of notation. This is the load-bearing property, not a decorative interface convention.

The shared state-transition form is:

\[
x_{k+1}-x_k=S r_k+B b_k,
\]

where \(x\) is the vector of typed stocks, \(S\) gives internal process effects, \(r\) gives process extents, and \(B b\) gives boundary exchanges. For every row vector \(w\) such that \(w^\mathsf T S=0\), the corresponding open-balance sentence is

\[
w^\mathsf T(x_{k+1}-x_k)=w^\mathsf T B b_k.
\]

That one sentence shape covers carbon or energy through an ecosystem and money, claims, or inventories through an economy. In a closed boundary it reduces to conservation. In an open system it correctly says “change equals net boundary flow”; it does not falsely claim that energy or money inside the chosen boundary remains constant.

Noether's theorem belongs one level above the trace assertion. Institutionally it is an entailment schema:

\[
\{\operatorname{Invariant}(L,G),\operatorname{EulerLagrange}(L)\}
\models \operatorname{MomentumConserved}(J),
\]

or, for a forced/open system,

\[
\{\operatorname{Invariant}(L,G),\operatorname{ForcedEulerLagrange}(L,F)\}
\models
J_\xi(t_1)-J_\xi(t_0)
=\int_{t_0}^{t_1}\langle F,\xi_Q\rangle\,dt.
\]

The theorem says that every model satisfying the premises satisfies the conclusion. The runtime ledger is then a finite-trace witness or counterexample for a balance sentence. It is not the definition of satisfaction and it is not the derivation of the law.

## Approaches Found

### 1. One combined institution with domain theories

This is the cleanest first construction. Define one institution \(\mathcal N\) for typed, open, process-based dynamical systems. Define an economic theory \(T_E\) and an ecosystem theory \(T_C\) as sets of \(\mathcal N\)-sentences. An economy is a model of \(T_E\); an ecosystem is a model of \(T_C\).

This avoids pretending the two domains have different logics before a genuine difference has been found. It also lets the same kernel derive and check balances without erasing domain vocabulary.

### 2. Separate institutions connected by comorphisms

If economics and ecology later require materially different sentence languages or model categories, define institutions \(\mathcal E\), \(\mathcal C\), and \(\mathcal N\), with comorphisms

\[
(\Phi_E,\alpha_E,\beta_E):\mathcal E\to\mathcal N,
\qquad
(\Phi_C,\alpha_C,\beta_C):\mathcal C\to\mathcal N.
\]

For the economic encoding, for example:

\[
M_N\models_{\Phi_E(\Sigma)}^{\mathcal N}\alpha_E(\varphi)
\quad\Longleftrightarrow\quad
\beta_E(M_N)\models_\Sigma^{\mathcal E}\varphi.
\]

Here \(\Phi_E\) translates economic signatures, \(\alpha_E\) translates economic sentences, and \(\beta_E\) reads a Noether model as an economic model. The displayed equivalence is the comorphism satisfaction condition. It is the formal promise that the adapter cannot silently change what an economic sentence means.

### 3. A multiplex of smaller institutions

The most faithful full design is a combination of three institutions:

- \(\mathcal D\): dimensions and semantic quantity kinds;
- \(\mathcal B\): stocks, processes, boundaries, traces, and linear balance;
- \(\mathcal V\): configurations, actions, symmetries, forces, and variational dynamics.

The combined Noether institution \(\mathcal N\) adds bridge entailments from symmetry sentences in \(\mathcal V\) to balance sentences in \(\mathcal B\), subject to typing in \(\mathcal D\). This keeps three different truths distinct:

- a dimensional equation is well-typed;
- a balance vector lies in a process matrix's left nullspace;
- a momentum map follows from an invariant action.

They can produce similarly shaped runtime balances without having the same mathematical origin.

## The Proposed Noether Institution

### Signatures: `Sign_N`

A signature \(\Sigma\) is a finite typed open-process schema containing:

- semantic quantity kinds \(K\), each with a dimension vector and exact scale metadata;
- pools \(P\), with `kind : P -> K`;
- internal process symbols \(R\);
- boundary-port symbols \(A\);
- exact internal effect matrix \(S\) and boundary incidence matrix \(B\);
- optional configuration, Lagrangian or discrete-Lagrangian, force, group-action, and infinitesimal-generator symbols;
- names and provenance, which do not participate in the algebra unless explicitly referenced by sentences.

A conservative signature morphism maps kinds, pools, processes, ports, and variational symbols while preserving kinds, dimensions, coefficients, boundary orientation, and declared action structure. Renamings and conservative extensions are safe initial morphisms.

Many-to-one aggregation is deliberately not an ordinary renaming. Aggregating firms into a sector or species into a trophic guild can erase distinctions and alter truth. It therefore needs a separately proved interpretation with explicit lumpability or abstraction obligations.

### Sentences: `Sen_N(Sigma)`

The sentence language contains closed, typed assertions such as:

- dimensional and semantic-kind equalities;
- state-transition equations;
- `ClosedBalance(w)` and `OpenBalance(w, ports)`;
- non-negativity and capacity constraints;
- event- or trace-balance assertions;
- `ActionInvariant(L, G)`;
- Euler-Lagrange or discrete Euler-Lagrange equations;
- forced momentum-balance assertions;
- Boolean combinations and quantification over the sorts made available by the signature.

The important balance sentence is semantic, not merely matrix arithmetic:

\[
\operatorname{OpenBalance}(w)\equiv
\forall k.\;w^\mathsf T(x_{k+1}-x_k)=w^\mathsf T B b_k.
\]

The exact calculation \(w^\mathsf T S=0\) is a derivation certificate showing that `StateTransition(S,B)` entails `OpenBalance(w)`. A declared law whose vector is not in the left nullspace receives no such certificate.

### Models: `Mod_N(Sigma)`

A \(\Sigma\)-model interprets:

- kinds as concrete quantity domains;
- pools as state coordinates;
- process and boundary symbols as measured or simulated event/flow values;
- an admissible set of states and traces;
- a transition relation or evolution law;
- optional configuration spaces, actions or discrete actions, group actions, forces, and momentum maps.

Models must be allowed to falsify sentences. An implementation trace with an unbalanced update is still data that can be interpreted as a model; it simply does not satisfy the balance sentence. Making “all models are balanced by construction” would prevent the logic from describing the bugs the checker exists to find.

Model morphisms preserve the relevant interpretations and admissible behavior. For an initial kernel, the category may safely be locally discrete—identity arrows only—while the object-level semantics is established. Rich simulation homomorphisms can be added later.

### Satisfaction: `M |=_Sigma phi`

Satisfaction is ordinary semantic truth under the model's interpretation. In particular:

\[
M\models_\Sigma\operatorname{OpenBalance}(w)
\]

means that every admissible transition in \(M\) obeys the displayed balance equation. For an observed finite trace, satisfaction quantifies over that trace. For a simulator specification, it quantifies over all executions admitted by the model.

A ledger record can carry enough exact or tolerance-qualified evidence to decide a ground, finite-trace instance. That evidence is a **satisfaction certificate**. A failed record is a counterexample. Neither one replaces the model-theoretic relation.

### Why the satisfaction condition holds

Let \(\sigma:\Sigma\to\Sigma'\) be a conservative rename or extension. `Sen(sigma)` renames every symbol in a sentence. `Mod(sigma)` forgets the extra \(\Sigma'\)-structure and reads the remaining interpretations under their \(\Sigma\) names.

For an atomic balance sentence, both sides of

\[
M'\models_{\Sigma'}\operatorname{Sen}(\sigma)(\varphi)
\Longleftrightarrow
\operatorname{Mod}(\sigma)(M')\models_\Sigma\varphi
\]

evaluate the same state coordinates, matrices, ports, and transition values. The only difference is their spelling. The result extends through equality, inequalities, connectives, and quantifiers by structural induction. This is the required proof for the conservative-morphism fragment.

For example, let \(\sigma\) rename `rabbit_carbon` to `consumer_carbon`. Translating the carbon-balance sentence and checking it in the renamed model gives exactly the same truth value as reducing that model back to the old vocabulary and checking the old sentence. This is a legitimate change of notation. Hiding a new wolf-to-rabbit flow without turning it into a boundary flow is not, because that would change the proposition.

## Economics as an Institution or Theory

An economic signature supplies sectors, accounts, instruments, inventories, transaction and production types, issuers, and rest-of-world ports. Its Noether encoding is:

| Economic construct | Noether construct |
|---|---|
| account or inventory | typed pool |
| currency, good, claim, labor-hour | semantic quantity kind |
| transfer or exchange | internal process column |
| production or consumption | transformation process column |
| issuance, retirement, import, export | boundary/source column |
| transaction history | event trace |

Economic sentences include:

- each double-entry transaction has equal-and-opposite signed postings;
- stock changes equal the sum of transaction flows;
- a stock-flow-consistent transaction matrix has the required zero row/column sums;
- inventories and financial positions obey their instrument-specific balance equations;
- issuance, retirement, imports, exports, loss, and depreciation appear explicitly rather than as unexplained residuals.

For a transaction matrix \(S_\$\), a signed unit vector \(w_\$\) with

\[
w_\$^\mathsf T S_\$=0
\]

induces the sentence

\[
\Delta(w_\$^\mathsf T x)=w_\$^\mathsf T B b.
\]

If the issuer's liability is inside the boundary, issuance can be an internal equal-and-opposite posting. If the issuer is outside, it is a boundary flow. The sentence therefore exposes, rather than conceals, the chosen accounting boundary.

A concrete agent-based or stock-flow-consistent economy is a model. It satisfies the accounting theory when every admitted transaction trace preserves the identities. Preferences, prices, expectations, market-clearing assumptions, and behavioral equations are additional economic sentences; they are not implied by conservation.

Double entry is best classified as an incidence or Pacioli-group invariant, and stock-flow consistency as a process-network invariant. It should not be called a consequence of Noether's variational theorem unless an action and a continuous symmetry have actually been supplied.

## Ecosystems as an Institution or Theory

An ecosystem signature supplies species or functional groups, compartments, chemical elements, energy kinds, ecological processes, and environment ports. Its Noether encoding is:

| Ecosystem construct | Noether construct |
|---|---|
| organism, guild, detritus, nutrient compartment | typed pool |
| carbon, nitrogen, phosphorus, chemical energy | semantic quantity kind |
| feeding, growth, death, decomposition, respiration | internal process column |
| sunlight, migration, harvest, runoff, heat | boundary port |
| observation or simulation history | trace |

Ecosystem sentences include:

- elemental mass balance for carbon, nitrogen, and phosphorus;
- open energy balance with sunlight or chemical energy entering and heat/work leaving;
- non-negativity of physically stored quantities;
- stoichiometric constraints on transformations;
- explicit boundary terms for migration, harvest, deposition, runoff, and measurement scope;
- optional thermodynamic or variational laws for physical submodels.

For a carbon process matrix \(S_C\), a vector \(w_C\) that totals the carbon content of the included pools satisfies

\[
w_C^\mathsf T S_C=0.
\]

The corresponding ecosystem sentence is

\[
\Delta C_{\mathrm{inside}}=C_{\mathrm{imported}}-C_{\mathrm{exported}}.
\]

A forest plot, lake food web, chemostat, or ecosystem simulation is a model. It satisfies the carbon sentence when all its admitted traces balance relative to the declared boundary. Total stored ecosystem energy is generally not constant; energy conservation is an input-output balance including radiation, chemical input, heat, and work.

## One Sentence, Two Models

Let the shared signature fragment contain typed pools, internal process effects \(S\), boundary effects \(B\), and traces. Let

\[
\varphi_w \equiv
\forall k.\;w^\mathsf T(x_{k+1}-x_k)=w^\mathsf T B b_k.
\]

- In an ecosystem model, \(w\) totals carbon across plant, herbivore, detritus, and atmospheric pools. Feeding and death cancel internally; harvest and atmospheric exchange remain on the boundary.
- In an economy model, \(w\) totals signed positions in one instrument. Payments cancel internally; issuance, retirement, or rest-of-world flows remain on the boundary according to scope.

The two models satisfy the same abstract sentence for different typed signatures. This is the reusable core. The domain adapters determine the meaning of the pools and ports; the kernel preserves the proposition.

## Noether Sentences and Entailments

For a discrete variational system, include a configuration space \(Q\), discrete Lagrangian \(L_d:Q\times Q\to\mathbb R\), group action, generator \(\xi_Q\), force, and momentum map \(J_\xi\). Representative sentences are:

\[
\operatorname{Invariant}(L_d,\xi),
\qquad
\operatorname{DEL}(L_d),
\qquad
\operatorname{MomentumBalance}(J_\xi,F).
\]

The discrete Noether theorem is the institutional entailment

\[
\{\operatorname{Invariant}(L_d,\xi),\operatorname{DEL}(L_d)\}
\models
J_\xi(q_k,q_{k+1})=J_\xi(q_{k-1},q_k).
\]

With forcing, the conclusion becomes a balance with the projected force as its source term. This is the honest bridge from symmetry to a ledger law. By contrast, if \(w^\mathsf T S=0\), the balance follows by linear algebra from the process equation. Both conclusions can be checked by the same runtime machinery, but their proof artifacts should carry distinct origins:

- `Noether`;
- `StoichiometricNullspace`;
- `IncidenceNullspace`;
- `ForcedBalance`;
- `Declared`.

Fixed-step variational integrators normally preserve the discrete symplectic form and momenta associated with configuration symmetries, but not exact energy. Exact energy from time-translation symmetry requires treating time steps as dynamical variables or another extended formulation. The first genuine Noether slice should therefore derive linear momentum from spatial translation, not advertise exact fixed-step energy conservation.

## Key Papers

1. **Goguen and Burstall (1992), “Institutions: Abstract Model Theory for Specification and Programming.”** The original mature definition of institutions and the satisfaction condition; it motivates truth invariant under change of notation and combinations of logical systems. [LFCS record](https://publish.lfcs.inf.ed.ac.uk/reports/90/ECS-LFCS-90-106/) and [author project page](https://cseweb.ucsd.edu/~goguen/projs/inst.html).
2. **Marsden and West (2001), “Discrete Mechanics and Variational Integrators.”** Establishes discrete variational mechanics and the discrete Noether theorem, including forced systems. [CaltechAUTHORS record](https://authors.library.caltech.edu/records/1h96d-ymc40) and [DOI](https://doi.org/10.1017/S096249290100006X).
3. **Ellerman (1985), “The Mathematics of Double Entry Bookkeeping.”** Treats transactions as zero terms in an additive group, making the algebraic conservation structure of double entry explicit. [Author-hosted paper](https://www.ellerman.org/Davids-Stuff/Maths/DEB-Math-Mag.CV.pdf).
4. **Godley and Lavoie (2007), _Monetary Economics_, Chapter 2.** Gives the stock-flow-consistent transaction-matrix discipline in which rows and columns balance and stocks are linked to flows. [Levy Institute chapter](https://www.levyinstitute.org/pubs/Godley%20Lavoie%202007%20chap02-1.pdf).
5. **Caiani et al. (2016), “Agent Based-Stock Flow Consistent Macroeconomics.”** Demonstrates a concrete modeling architecture combining agent behavior with coherent real and financial stocks and flows. [Journal article](https://doi.org/10.1016/j.jedc.2016.06.001).
6. **Fath (2012), “Analyzing Ecological Systems Using Network Analysis.”** Describes ecological models in terms of inter-compartmental flows, storages, and system-boundary inputs and outputs. [IIASA record](https://pure.iiasa.ac.at/id/eprint/9834/).
7. **Mahdi, Ferragut, Valls, and Wiuf (2017), “Conservation Laws in Biochemical Reaction Networks.”** Develops linear conservation laws for reaction networks, including their relationship to the left nullspace of the stoichiometric matrix. [SIAM article](https://epubs.siam.org/doi/10.1137/17M1138418).
8. **Maarleveld et al. (2013), “Basic Concepts and Principles of Stoichiometric Modeling of Metabolic Networks.”** Provides an accessible account of stoichiometric matrices and conserved moieties obtained from the left nullspace. [Open article](https://pmc.ncbi.nlm.nih.gov/articles/PMC4671265/).

## Existing Implementations

- **Hets (Heterogeneous Tool Set)** implements institution-based heterogeneous specifications, logic translations, development graphs, and cross-logic proof obligations. It is the closest existing engineering reference for keeping multiple domain logics connected without flattening them into one syntax. [Official repository](https://github.com/spechub/Hets).
- **Variational-integrator libraries and research implementations** provide the numerical side of discrete mechanics, but they do not by themselves provide a general institution, typed conservation provenance, or economy/ecosystem adapters. Marsden and West provide the mathematical reference rather than a drop-in conservation kernel.
- **Stock-flow-consistent and ecological-network tools** embody pieces of the required semantics—balanced transaction matrices or compartment flows—but generally specialize the vocabulary and model category to one domain.
- The local `plenty/tally` design can inform event certificates and diagnostics, while Bridgman can inform dimension and semantic-kind descriptors. Neither should define the Noether entailments; both belong behind satisfaction-preserving adapters.

## Complexity vs Quality Tradeoffs

| Choice | Lower-complexity version | Higher-fidelity version |
|---|---|---|
| Domain structure | economics and ecology as theories in one institution | separate institutions plus proved comorphisms |
| Signature morphisms | renaming and conservative extension only | aggregation, hiding, unit conversion, and refinement with proof obligations |
| Models | locally discrete category of trace/evolution objects | behavior-preserving model morphisms and abstraction maps |
| Sentences | ground balance, transition, and typing assertions | quantified temporal, variational, and thermodynamic language |
| Arithmetic | exact rational law derivation; toleranced trace checking | certified interval/error semantics for numerical trajectories |
| Theorem support | kernel/nullspace derivations plus one discrete Noether theorem | mechanized proofs for combined institutions and bridge theorems |

The minimal version can still be a genuine institution. The line that must not be crossed is omitting the satisfaction condition or allowing adapters that alter truth without a proof obligation.

## Recommendations

1. Define \(\mathcal D\) and \(\mathcal B\) first, with conservative signature morphisms and a written proof of the satisfaction condition.
2. Represent economics and ecosystems initially as theories/profiles over the combined typed-balance institution. Promote either domain to a separate institution only when its language or model reduct genuinely demands it.
3. Add \(\mathcal V\) with the smallest true Noether entailment: spatial-translation invariance of a discrete Lagrangian entails preservation of the corresponding discrete momentum map.
4. Store a law's mathematical origin and derivation certificate separately from event-level satisfaction certificates.
5. Treat external flows, forces, issuance, harvest, radiation, heat, and imports/exports as explicit boundary terms. A residual is evidence of an incomplete model, not a conservation law.
6. Permit only renamings and conservative extensions as first-class signature morphisms initially. Require explicit proofs for aggregation and hiding.
7. Make exact rational matrices authoritative for derivation. Use floating-point or measured quantities only at the model/trace layer with declared error semantics.

## Estimated Implementation Effort

- **Foundational slice:** moderate. Formalize typed signatures, conservative morphisms, balance sentences, trace models, satisfaction, and the satisfaction-condition proof; implement exact nullspace derivation and finite-trace certificates.
- **First true Noether slice:** moderate to high. Formalize discrete Lagrangians, group generators, momentum maps, forced balance, and a test model demonstrating the entailment.
- **Domain profiles:** moderate per domain. Economic and ecological vocabularies are straightforward; correct boundaries, kinds, and aggregation semantics are the hard part.
- **Full institution-comorphism layer:** high. Natural transformations, abstraction/refinement laws, mechanized proof artifacts, and heterogeneous tooling are a separate maturity stage, not necessary for the first sound kernel.

## Open Questions

1. Should the first model semantics quantify over all admitted simulator executions, only committed event traces, or support both as distinct model classes?
2. Which aggregation maps are required first, and what domain-specific lumpability conditions make them satisfaction preserving?
3. Should uncertainty be represented by interval-valued models, probabilistic satisfaction, or ordinary models plus explicit measurement-error sentences?
4. Which economic quantities are conserved only as signed claims, which are transformed inventories, and which are boundary-created instruments?
5. Which ecosystem boundary is authoritative for each property: organism, plot, watershed, biome, or planet?
6. Should thermodynamic irreversibility be a separate institution combined with Noether, rather than another family of balance sentences?

## References

- Goguen, J. A., and Burstall, R. M. (1992). “Institutions: Abstract Model Theory for Specification and Programming.” _Journal of the ACM_, 39(1), 95-146. [LFCS](https://publish.lfcs.inf.ed.ac.uk/reports/90/ECS-LFCS-90-106/).
- Mossakowski, T., Maeder, C., Lüttich, K., and Wölfl, S. “Heterogeneous Theories and the Heterogeneous Tool Set.” [Hets repository and bibliography](https://github.com/spechub/Hets).
- Marsden, J. E., and West, M. (2001). “Discrete Mechanics and Variational Integrators.” _Acta Numerica_, 10, 357-514. [DOI](https://doi.org/10.1017/S096249290100006X).
- Ellerman, D. P. (1985). “The Mathematics of Double Entry Bookkeeping.” _Mathematics Magazine_, 58(4), 226-233. [PDF](https://www.ellerman.org/Davids-Stuff/Maths/DEB-Math-Mag.CV.pdf).
- Godley, W., and Lavoie, M. (2007). _Monetary Economics_. Chapter 2. [PDF](https://www.levyinstitute.org/pubs/Godley%20Lavoie%202007%20chap02-1.pdf).
- Caiani, A., Godin, A., Caverzasi, E., Gallegati, M., Kinsella, S., and Stiglitz, J. E. (2016). “Agent Based-Stock Flow Consistent Macroeconomics.” _Journal of Economic Dynamics and Control_, 69, 375-408. [DOI](https://doi.org/10.1016/j.jedc.2016.06.001).
- Fath, B. D. (2012). “Analyzing Ecological Systems Using Network Analysis.” [IIASA](https://pure.iiasa.ac.at/id/eprint/9834/).
- Mahdi, A., Ferragut, A., Valls, C., and Wiuf, C. (2017). “Conservation Laws in Biochemical Reaction Networks.” _SIAM Journal on Applied Dynamical Systems_, 16(4), 2213-2232. [DOI](https://doi.org/10.1137/17M1138418).
- Maarleveld, T. R., Khandelwal, R. A., Olivier, B. G., Teusink, B., and Bruggeman, F. J. (2013). “Basic Concepts and Principles of Stoichiometric Modeling of Metabolic Networks.” _Biotechnology Journal_, 8(9), 997-1008. [PubMed Central](https://pmc.ncbi.nlm.nih.gov/articles/PMC4671265/).
