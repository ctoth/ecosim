# Economies, Ecosystems, and a Goguen-Style Stock–Flow Institution

## Summary

The precise claim is slightly different from “an economy is an institution”:

- a particular economy or ecosystem can serve as a **model** once its
  signature-indexed interpretation and reduct behavior are defined;
- an economic or ecological modeling formalism is an **institution** when it
  supplies a lawful signature category, functorial sentence translation and
  model reduct, signature-indexed satisfaction, and proves the satisfaction
  condition;
- the reusable conservation kernel can itself be a stock–flow institution;
  future cross-institution adapters must define their variance, naturality, and
  satisfaction laws before they are called morphisms or comorphisms;
- more simply for an initial implementation, economics and ecology can be
  presentations or profiles inside one shared institution, promoted to closed
  theories only after semantic closure is represented.

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

where \(x\) is the vector of typed stocks, \(S\) gives internal process effects,
\(r\) gives process extents, and \(B b\) gives boundary exchanges. Let \(w\) be
a column coefficient vector. Extending Mahdi et al.'s closed continuous
structural-linear carrier with this project-specific discrete open equation,
\(w^\mathsf T S=0\) algebraically entails the corresponding open-balance
sentence

\[
w^\mathsf T(x_{k+1}-x_k)=w^\mathsf T B b_k.
\]

That one sentence shape covers carbon or energy through an ecosystem and money, claims, or inventories through an economy. In a closed boundary it reduces to conservation. In an open system it correctly says “change equals net boundary flow”; it does not falsely claim that energy or money inside the chosen boundary remains constant.

Noether's theorem belongs one level above the trace assertion. The following
is this project's Goguen-style reformulation of the discrete theorem as an
entailment schema:

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

Marsden and West supply the variational and forced momentum equations; they do
not supply the institutional framing. Under that framing, every model
satisfying the premises satisfies the conclusion. The runtime ledger is a
project-specific finite-trace witness or counterexample for a balance
sentence. It is not the definition of satisfaction and it is not the
derivation of the law.

## Approaches Found

### 1. One combined institution with domain theories

This is the cleanest first construction. Define one institution \(\mathcal N\)
for typed, open, process-based dynamical systems. Define an economic
presentation \(T_E\) and an ecosystem presentation \(T_C\) as sets of
\(\mathcal N\)-sentences. They become theories only after closure under semantic
consequence is represented. A signature-indexed economy can be a model of
\(T_E\); a signature-indexed ecosystem can be a model of \(T_C\).

This avoids pretending the two domains have different logics before a genuine difference has been found. It also lets the same kernel derive and check balances without erasing domain vocabulary.

### 2. Separate institutions connected by cross-institution adapters

If economics and ecology later require materially different sentence languages
or model categories, define institutions \(\mathcal E\), \(\mathcal C\), and
\(\mathcal N\). The following is a candidate variance convention for adapters,
not yet a proved comorphism construction:

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

Here \(\Phi_E\) translates economic signatures, \(\alpha_E\) translates
economic sentences, and \(\beta_E\) reads a shared model as an economic model.
The displayed equivalence is the required cross-institution satisfaction
condition under this proposed convention. Naturality and the precise variance
of every component must also be defined and tested before the adapter receives
a standard categorical name.

### 3. A multiplex of smaller institutions

One proposed multiplex design is a combination of three institutions:

- \(\mathcal D\): dimensions and semantic quantity kinds;
- \(\mathcal B\): stocks, processes, boundaries, traces, and linear balance;
- \(\mathcal V\): configurations, actions, symmetries, forces, and variational dynamics.

The combined Noether institution \(\mathcal N\) adds bridge entailments from symmetry sentences in \(\mathcal V\) to balance sentences in \(\mathcal B\), subject to typing in \(\mathcal D\). This keeps three different truths distinct:

- a dimensional equation is well-typed;
- a balance vector lies in a process matrix's left nullspace;
- a momentum map follows from an invariant action.

They can produce similarly shaped runtime balances without having the same mathematical origin.

## Project-Specific Proposed Stock–Flow and Variational Institution

The signature, sentence, and model language below is a project design rather
than a construction asserted by Goguen and Burstall. It becomes an institution
only after its category, functoriality, model reduct, and satisfaction laws are
proved.

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

A project certificate recomputes Mahdi et al.'s structural-linear condition
\(w^\mathsf T S=0\) exactly. Under the project's discrete open transition
equation, that checked premise entails `OpenBalance(w)`. A declared law whose
vector is not in the left nullspace receives no such certificate.

### Models: `Mod_N(Sigma)`

A structure can serve as a \(\Sigma\)-model once its signature-indexed
interpretation and reduct behavior are defined. It interprets:

- kinds as concrete quantity domains;
- pools as state coordinates;
- process and boundary symbols as measured or simulated event/flow values;
- an admissible set of states and traces;
- a transition relation or evolution law;
- optional configuration spaces, actions or discrete actions, group actions, forces, and momentum maps.

Models must be allowed to falsify sentences. An implementation trace with an unbalanced update is still data that can be interpreted as a model; it simply does not satisfy the balance sentence. Making “all models are balanced by construction” would prevent the logic from describing the bugs the checker exists to find.

Model morphisms preserve the relevant interpretations and admissible behavior.
For an initial kernel, the model category is an explicitly documented
discrete-category specialization—identity arrows only—while the object-level
semantics and reduct functoriality are established. Rich simulation
homomorphisms can be added later.

### Satisfaction: `M |=_Sigma phi`

Satisfaction is ordinary semantic truth under the model's interpretation. In particular:

\[
M\models_\Sigma\operatorname{OpenBalance}(w)
\]

means that every admissible transition in \(M\) obeys the displayed balance equation. For an observed finite trace, satisfaction quantifies over that trace. For a simulator specification, it quantifies over all executions admitted by the model.

As an additional project artifact, a ledger record can carry enough exact or
tolerance-qualified evidence to decide a ground, finite-trace instance. That
evidence is a **satisfaction certificate**. A failed record is a counterexample.
Neither one replaces the model-theoretic relation.

### Satisfaction-condition proof obligation

Let \(\sigma:\Sigma\to\Sigma'\) be a conservative rename or extension. `Sen(sigma)` renames every symbol in a sentence. `Mod(sigma)` forgets the extra \(\Sigma'\)-structure and reads the remaining interpretations under their \(\Sigma\) names.

For an atomic balance sentence, both sides of

\[
M'\models_{\Sigma'}\operatorname{Sen}(\sigma)(\varphi)
\Longleftrightarrow
\operatorname{Mod}(\sigma)(M')\models_\Sigma\varphi
\]

evaluate the same state coordinates, matrices, ports, and transition values.
The only difference is their spelling. This is a proof sketch for atomic
satisfaction preservation. Establishing the proposed institution additionally
requires the signature category, identity and composition laws, sentence
functoriality, contravariant reduct functoriality, and closure of every sentence
variant under translation.

For example, let \(\sigma\) rename `rabbit_carbon` to `consumer_carbon`. Translating the carbon-balance sentence and checking it in the renamed model gives exactly the same truth value as reducing that model back to the old vocabulary and checking the old sentence. This is a legitimate change of notation. Hiding a new wolf-to-rabbit flow without turning it into a boundary flow is not, because that would change the proposition.

## Economics as an Institution or Theory

Godley and Lavoie supply the closed-economy balance sheets and
transactions-flow matrix discipline; Caiani et al. supply a concrete
agent-based SFC architecture with individual balance sheets, markets, and
behavioral rules. The typed stock-flow table below is this project's proposed
encoding, not terminology from either source. Rest-of-world ports are a project
open-economy extension of the cited closed-economy chapter:

| Economic construct | Stock–flow construct |
|---|---|
| account or inventory | typed pool |
| currency, good, claim, labor-hour | semantic quantity kind |
| transfer or exchange | internal process column |
| production or consumption | transformation process column |
| issuance, retirement, import, export | boundary/source column |
| transaction history | event trace |

The proposed economic presentation includes:

- each double-entry transaction records equal total debits and credits;
  under this project's signed-vector encoding, its postings sum to zero;
- inter-sector transactions have counterpart source/use entries producing the
  entries needed to balance transaction rows and sector columns;
- closing stock equals opening stock plus transaction change plus revaluation
  change (or just transaction change for a fixed-price asset without capital
  gains);
- a stock-flow-consistent transaction matrix has the required zero row/column sums;
- inventories and financial positions obey their instrument-specific balance equations;
- issuance, buy-backs, and revaluations appear explicitly rather than as
  unexplained residuals; imports, exports, loss, and depreciation require
  explicit project boundary or process symbols when such processes are in
  scope.

For Godley and Lavoie's rows-by-sectors transactions-flow matrix \(T\), the
accounting discipline is

\[
\mathbf 1^\mathsf T T=0,
\qquad
T\mathbf 1=0.
\]

This project's separate mapping from transactions to a stock-change operator
\(S_\$\), boundary operator \(B\), and coefficient vector \(w_\$\) then uses
the checked structural premise \(w_\$^\mathsf T S_\$=0\) to derive

\[
\Delta(w_\$^\mathsf T x)=w_\$^\mathsf T B b.
\]

The cited closed-economy accounting supports the internal asset/liability
counterpart when both parties are inside. Treating an outside issuer as a
boundary flow is the project's open-system extension. The sentence therefore
exposes, rather than conceals, the chosen accounting boundary.

A concrete agent-based or stock-flow-consistent economy can serve as a model
once its signature-indexed interpretation and reducts are defined. It satisfies
the accounting presentation when every admitted transaction trace preserves
the identities. Caiani et al.'s model demonstrates that accounting-consistent
balance sheets and transactions can coexist with distinct market interactions,
pricing, credit, investment, expectations, and behavioral heuristics. Godley
and Lavoie likewise separate behavioral determination, including price
dynamics, from accounting identities; prices may still enter valuation and
revaluation accounting.

Ellerman characterizes double entry as preservation of equation-representing
zero-terms in the Pacioli group. Translating that construction to signed
vectors or incidence matrices is a separate project encoding. Godley and
Lavoie supply the stock-flow-consistency matrix discipline. Ellerman supplies
the bookkeeping algebra, while Marsden and West delimit when a variational
Noether claim is available; an accounting identity is therefore not presented
as a consequence of Noether's theorem without a Lagrangian system and group
symmetry.

## Ecosystems as an Institution or Theory

Fath supports ecological-network descriptions in terms of compartmental
storages, inter-compartmental energy/matter/nutrient flows, and explicit
boundary inputs and outputs. The richer typed mapping below is this project's
proposed encoding, not a construction supplied by Fath:

| Ecosystem construct | Stock–flow construct |
|---|---|
| organism, guild, detritus, nutrient compartment | typed pool |
| carbon, nitrogen, phosphorus, chemical energy | semantic quantity kind |
| feeding, growth, death, decomposition, respiration | internal process column |
| sunlight, migration, harvest, runoff, heat | boundary port |
| observation or simulation history | trace |

The proposed ecosystem presentation includes:

- elemental mass balance for carbon, nitrogen, and phosphorus;
- open energy balance with sunlight or chemical energy entering and heat/work leaving;
- non-negativity of physically stored quantities;
- stoichiometric constraints on transformations;
- explicit boundary terms for migration, harvest, deposition, runoff, and measurement scope;
- optional thermodynamic or variational laws for physical submodels.

Combining the project's discrete open carrier with Mahdi et al.'s
structural-linear premise, a carbon process matrix \(S_C\) and coefficient
vector \(w_C\) may satisfy

\[
w_C^\mathsf T S_C=0.
\]

The corresponding ecosystem sentence is

\[
\Delta C_{\mathrm{inside}}=C_{\mathrm{imported}}-C_{\mathrm{exported}}.
\]

A forest plot, lake food web, chemostat, or ecosystem simulation can serve as a
model once its signature-indexed interpretation and reducts are defined. It
satisfies the carbon sentence when all its admitted traces balance relative to
the declared boundary. As a project physical-modeling rule, stored ecosystem
energy is modeled by an open input-output balance with every represented
radiation, chemical-input, heat, work, and export port explicit; this broader
thermodynamic claim requires a dedicated source before empirical use.

## One Sentence, Two Models

Let the shared signature fragment contain typed pools, internal process effects \(S\), boundary effects \(B\), and traces. Let

\[
\varphi_w \equiv
\forall k.\;w^\mathsf T(x_{k+1}-x_k)=w^\mathsf T B b_k.
\]

- In the proposed ecosystem model, \(w\) totals carbon across declared internal
  pools. Inter-compartmental transfers are internal; declared external inputs
  and dissipative outputs cross the model boundary.
- In an economy model, \(w\) totals signed positions in one instrument. Payments cancel internally; issuance, retirement, or rest-of-world flows remain on the boundary according to scope.

The two models satisfy the same abstract sentence for different typed signatures. This is the reusable core. The domain adapters determine the meaning of the pools and ports; the kernel preserves the proposition.

## Noether Sentences and Entailments

For an unforced discrete variational system, include a configuration space
\(Q\), discrete Lagrangian \(L_d:Q\times Q\to\mathbb R\), group action,
generator \(\xi_Q\), discrete Euler–Lagrange evolution, and momentum map
\(J_\xi\). For the forced extension, add the two discrete forces \(f_d^-\) and
\(f_d^+\). Representative project sentences are:

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

With forcing, the conclusion becomes a balance with the projected force as its
source term. This is the honest bridge from symmetry to a ledger law. By
contrast, under the project's discrete transition equation,
\(w^\mathsf T S=0\) makes the open balance follow by linear algebra. Both
conclusions can be checked by related runtime machinery, but as a project
provenance taxonomy their proof artifacts should carry distinct origins:

- `Noether`;
- `StoichiometricNullspace`;
- `IncidenceNullspace`;
- `ForcedBalance`;
- `Declared`.

Unforced fixed-step variational integrators preserve the discrete symplectic
form and momenta associated with configuration symmetries, but generally should
not enforce exact conservation of the true Hamiltonian. An extended
time-translation formulation recovers a discrete energy conservation statement;
that discrete energy agrees with the true energy only under stronger
exact-discrete-Lagrangian conditions. The first genuine Noether slice should
therefore derive linear momentum from spatial translation, not advertise exact
fixed-step conservation of the physical Hamiltonian.

## Key Papers

1. **Goguen and Burstall (1992), “Institutions: Abstract Model Theory for Specification and Programming.”** Defines institutions and the satisfaction condition and develops change of notation and combinations of logical systems. [LFCS record](https://publish.lfcs.inf.ed.ac.uk/reports/90/ECS-LFCS-90-106/) and [author project page](https://cseweb.ucsd.edu/~goguen/projs/inst.html).
2. **Marsden and West (2001), “Discrete Mechanics and Variational Integrators.”** Establishes discrete variational mechanics and the discrete Noether theorem, including forced systems. [CaltechAUTHORS record](https://authors.library.caltech.edu/records/1h96d-ymc40) and [DOI](https://doi.org/10.1017/S096249290100006X).
3. **Ellerman (1985), “The Mathematics of Double Entry Bookkeeping.”** Constructs the Pacioli group and represents valid transactions as transactional zero-terms that preserve the encoded accounting equation. [Author-hosted paper](https://www.ellerman.org/Davids-Stuff/Maths/DEB-Math-Mag.CV.pdf).
4. **Godley and Lavoie (2007), _Monetary Economics_, Chapter 2.** Gives the stock-flow-consistent transaction-matrix discipline in which rows and columns balance and stocks are linked to flows. [Levy Institute chapter](https://www.levyinstitute.org/pubs/Godley%20Lavoie%202007%20chap02-1.pdf).
5. **Caiani et al. (2016), “Agent Based-Stock Flow Consistent Macroeconomics.”** Demonstrates a concrete modeling architecture combining agent behavior with coherent real and financial stocks and flows. [Journal article](https://doi.org/10.1016/j.jedc.2016.06.001).
6. **Fath (2012), “Analyzing Ecological Systems Using Network Analysis.”** Describes ecological models in terms of inter-compartmental flows, storages, and system-boundary inputs and outputs. [IIASA record](https://pure.iiasa.ac.at/id/eprint/9834/).
7. **Mahdi, Ferragut, Valls, and Wiuf (2017), “Conservation Laws in Biochemical Reaction Networks.”** Develops linear conservation laws for reaction networks, including their relationship to the left nullspace of the stoichiometric matrix. [SIAM article](https://epubs.siam.org/doi/10.1137/17M1138418).
8. **Maarleveld et al. (2013), “Basic Concepts and Principles of Stoichiometric Modeling of Metabolic Networks.”** Provides an accessible account of stoichiometric matrices and conserved moieties obtained from the left nullspace. [Open article](https://pmc.ncbi.nlm.nih.gov/articles/PMC4671265/).

## Existing Implementations

- **Hets (Heterogeneous Tool Set)** implements institution-based heterogeneous specifications, logic translations, development graphs, and cross-logic proof obligations. It is the closest existing engineering reference for keeping multiple domain logics connected without flattening them into one syntax. [Official repository](https://github.com/spechub/Hets).
- Marsden and West is a mathematical review of discrete mechanics and
  variational integrators, not an implementation specification for this
  project's Institution or conservation kernel. Claims about individual
  software libraries require separate inspection.
- Fath's Network Environ Analysis embodies domain-specific compartment,
  storage, internal-flow, and boundary-flow semantics. Claims about other
  ecological or stock-flow software require individual inspection.
- The local `plenty/tally` design can inform event certificates and diagnostics, while Bridgman can inform dimension and semantic-kind descriptors. Neither should define the Noether entailments; both belong behind satisfaction-preserving adapters.

## Complexity vs Quality Tradeoffs

| Choice | Lower-complexity version | Higher-fidelity version |
|---|---|---|
| Domain structure | economics and ecology as presentations in one institution | separate institutions plus proved cross-institution adapters |
| Signature morphisms | renaming and conservative extension only | aggregation, hiding, unit conversion, and refinement with proof obligations |
| Models | locally discrete category of trace/evolution objects | behavior-preserving model morphisms and abstraction maps |
| Sentences | ground balance, transition, and typing assertions | quantified temporal, variational, and thermodynamic language |
| Arithmetic | exact rational law derivation; toleranced trace checking | certified interval/error semantics for numerical trajectories |
| Theorem support | kernel/nullspace derivations plus one discrete Noether theorem | mechanized proofs for combined institutions and bridge theorems |

The minimal version can still be a genuine institution only with all four
components, a lawful signature category, functorial sentence translation and
model reduct, and the satisfaction condition. Adapters additionally need their
own variance, naturality, and cross-institution satisfaction obligations.

## Recommendations

1. Define \(\mathcal D\) and \(\mathcal B\) first, with conservative signature morphisms and a written proof of the satisfaction condition.
2. Represent economics and ecosystems initially as presentations/profiles over
   the combined typed-balance institution. Promote a presentation to a closed
   theory only when semantic closure is represented, and promote a domain to a
   separate institution only when its language or model reduct demands it.
3. Add \(\mathcal V\) with the smallest true Noether entailment: spatial-translation invariance of a discrete Lagrangian entails preservation of the corresponding discrete momentum map.
4. Store a law's mathematical origin and derivation certificate separately from event-level satisfaction certificates.
5. Under this project's open-system semantics, treat external flows, forces,
   issuance, harvest, radiation, heat, and imports/exports as explicit boundary
   terms. Treat a residual as evidence of an incomplete model, not as a law.
6. Permit only renamings and conservative extensions as first-class signature morphisms initially. Require explicit proofs for aggregation and hiding.
7. As a project verification policy, make exact rational matrices authoritative
   for derivation. Use floating-point or measured quantities only at the
   model/trace layer with declared error semantics.

## Estimated Implementation Effort

- **Foundational slice:** moderate. Formalize typed signatures, conservative morphisms, balance sentences, trace models, satisfaction, and the satisfaction-condition proof; implement exact nullspace derivation and finite-trace certificates.
- **First true Noether slice:** as a local engineering estimate, moderate to
  high. Formalize discrete Lagrangians, group generators, momentum maps, forced
  balance, and a test model demonstrating the entailment.
- **Domain profiles:** moderate per domain. Economic and ecological vocabularies are straightforward; correct boundaries, kinds, and aggregation semantics are the hard part.
- **Full cross-institution adapter layer:** high. Variance choices, natural
  transformations, cross-institution satisfaction, abstraction/refinement
  laws, mechanized proof artifacts, and heterogeneous tooling are a separate
  maturity stage, not necessary for the first sound kernel.

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
