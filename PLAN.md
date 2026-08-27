# Institutional Stock–Flow Laws Plan

## Purpose

Move from checking properties of state traces to representing and checking the
laws that actually produce each state transition. The central sentence is:

```text
x[k + 1] - x[k] = S f[k] + B b[k]
```

where `x` is typed stock state, `f` is the vector of settled scalar internal
flow channels, `b` is the vector of settled scalar boundary-flow channels, `S`
is the internal effect matrix, and `B` is the boundary effect matrix. One
domain process may compile to several independently typed flow channels.

The existing invariant should then become a derived consequence:

```text
w^T S = 0
    entails
w^T (x[k + 1] - x[k]) = w^T B b[k]
```

This plan institutionalizes the transition equation first, then individual
process and boundary honesty, typed multi-kind balances, ecosystem kinetic
laws, and paired-run experiment sentences. It keeps mathematical laws,
operational guarantees, and numerical evidence policies distinct.

## Selection criteria

The work is ordered only by these project criteria:

- a law must be expressible as a Goguen-style sentence with defined
  satisfaction, and this project's checker must return an inspectable witness
  or first-offense violation;
- it must add correctness beyond the existing invariant, nonnegative, and
  nondecreasing sentences;
- it must be testable from the current exact runtime or a principled extension
  of its trace;
- domain-neutral mathematics belongs in `conservation` and `institution`, while
  ecosystem vocabulary and behavior remain in `ecosim`;
- an abstraction shared with economics is extracted only after a concrete
  economic client demonstrates the same data types and semantics.

Implementation convenience, novelty, popularity, and speculative reuse are
not selection criteria.

## Current baseline

The completed predecessor slice provides:

- exact and dense compiled trophic-network plans;
- exact stock traces with settled cumulative input and output;
- compiled invariant, nonnegative-stock, and nondecreasing-ledger sentences;
- typed `LawVerdict` witnesses and first-offense violations;
- a frozen producer–herbivore–predator experiment with exact evidence;
- a documented ecosystem-to-economics concept mapping.

The current institutional model sees stock states and cumulative ledgers. It
does not see the settled internal process vector or the per-step boundary
vector. Consequently, a state trace may satisfy global balance without proving
that the declared processes account for each individual stock change.

## Law taxonomy

The plan treats different kinds of correctness according to their actual
mathematical role.

| Candidate | Classification | Location |
|---|---|---|
| `x' - x = S f + B b` | Stock–flow transition sentence | Shared stock–flow institution |
| Feeding assimilation/waste ratio | Linear constraint over settled process flows | Shared carrier; ecosystem theory |
| Ledger increment equals settled boundary flow | Boundary-correspondence sentence | Shared stock–flow institution |
| `w^T S = 0` implies open balance | Checked entailment with derivation certificate | Conservation mathematics |
| Carbon, nitrogen, phosphorus balances | Typed open-balance sentences | Ecosystem theory over shared institution |
| Stored-energy input/heat/work/export balance | Typed open-balance sentence | Ecosystem theory over shared institution |
| Monod, Holling, mortality, decomposition | Constitutive kinetic sentences | Ecosystem theory |
| Source-limited proportional settlement | Allocation sentence | Conservation dynamics plus ecosystem theory |
| Intervention-minus-control response | Paired-model experiment sentence | Ecosystem experiment theory |
| Rejected-step atomicity | Operational transition semantics | Runtime contract, not an ordinary sentence |
| Binary64 tolerance | Evidence interpretation policy | Dense runtime, not the law |
| Noether momentum conservation | Variational entailment | Later variational institution |

This plan does not call a process-network nullspace result “Noether's theorem.”
A genuine Noether slice still requires a configuration space, action or
Lagrangian, symmetry, equations of motion, and momentum map.

## Intended architecture

### Repository ownership

`conservation` owns domain-neutral exact mathematics and evidence carriers:

- typed state, internal-flow, and boundary-flow symbols;
- exact internal-flow and boundary effect matrices;
- exact transition records;
- transition, flow-constraint, and boundary-correspondence sentence data;
- typed witnesses and violations;
- checked nullspace derivation certificates and open-balance entailments.

`institution` owns the Goguen-style logical system:

- stock–flow signatures;
- conservative, kind-preserving symbol renamings;
- the stock–flow sentence sum type;
- stock–flow models and reducts as an explicitly documented discrete-model
  specialization of the category-valued definition;
- sentence translation;
- satisfaction delegated to the conservation checkers;
- executable identity, composition, and satisfaction-condition laws.

`ecosim` owns the domain theory and runtime integration:

- ecological stock, process, and boundary vocabulary;
- compilation of trophic topology into `S` and `B`;
- exact traces carrying settled internal-flow and boundary-flow amounts;
- feeding, mortality, decomposition, and kinetic sentences;
- multi-kind ecosystem fixtures;
- paired-run ecosystem experiment sentences.

`bridgman` remains separate. Quantity kinds in the first stock–flow slice use
the existing exact `KindId` carrier. Dimensional analysis may later validate
kinetic expressions through an adapter, but the conservation and institution
crates must not acquire a Python package dependency or conflate dimension
checking with conservation satisfaction.

### New shared layer, not a replacement

Preserve the existing graded conservation layer for state-only traces, subject
to a fresh audit of its signature category, translation and reduct
functoriality, and satisfaction condition. Introduce a stock–flow layer rather
than making every `GradedLaw` variant understand processes and boundaries.

This shared layer is not an extraction of ecosim's experiment API: the
transition equation, typed flow carrier, and satisfaction condition are the
domain-neutral mathematical subject of the institution itself. Intervention
schedules, kinetic expressions, and response metrics remain domain-owned.

A candidate shared shape is:

```text
StockFlowSignature
    stock axes: AxisId -> KindId
    internal flow symbols: FlowId -> KindId
    boundary flow symbols: BoundaryId -> KindId
    internal effects: S
    boundary effects: B

TransitionRecord
    before: TraceState
    after: TraceState
    settled_flows: FlowId -> exact amount
    settled_boundaries: BoundaryId -> exact nonnegative magnitude

StockFlowSentence
    TransitionEquation
    LinearFlowConstraint
    BoundaryCorrespondence
    GradedStateLaw(GradedLaw)
```

Names are provisional until the upstream APIs are implemented and reviewed;
the semantic separation is not provisional.

Every scalar flow channel has one kind, and its matrix column may have nonzero
effects only on compatible stock axes. A feeding, production, or transaction
bundle that moves several kinds compiles to several scalar channels. Any
cross-kind stoichiometric ratio requires an explicitly typed conversion in the
domain theory; it cannot hide inside an untyped matrix coefficient.

### Transition satisfaction

For each accepted transition and each stock axis, satisfaction checks exactly:

```text
observed_delta(axis)
    == sum(S[axis, flow] * settled_flow[flow])
     + sum(B[axis, port] * settled_boundary[port])
```

A positive witness records at least:

- the number of transitions checked;
- the number of stock axes checked;
- the exact zero residual;
- the signature or matrix identity against which the trace was checked.

A violation records the first:

- transition index;
- stock axis;
- observed delta;
- accounted delta;
- exact residual.

Malformed signatures, missing symbols, mismatched kinds, discontinuous
before/after states, and invalid matrix shapes are structural errors, not
semantic falsity.

### Flow-constraint satisfaction

`LinearFlowConstraint` expresses exact relations between named settled flow
amounts. For one feeding event represented by assimilated and waste transfers:

```text
(1 - eta) * assimilated - eta * waste = 0
```

Together with the transition equation, this proves that resource withdrawal
is partitioned into consumer assimilation and detrital waste in the declared
ratio. Constraints carry explicit kinds and may not add incompatible
quantities.

### Boundary correspondence

Boundary ledgers remain stock coordinates when a model wants cumulative
accounts. The stronger sentence connects their increments to actual settled
ports:

```text
delta(cumulative_input) = settled(input_port)
delta(cumulative_output) = settled(output_port)
```

Nondecreasing ledger sentences remain useful but no longer stand in for this
correspondence.

### Derivation and satisfaction certificates

Keep two artifacts separate:

- a derivation certificate proves an exact algebraic fact such as `w^T S = 0`;
- a satisfaction witness proves that one concrete finite model satisfies one
  sentence.

As a project evidence policy, `Provenance` remains origin metadata. Only a
constructor that recomputes and checks the required exact matrix identity may
create a derivation certificate.
Callers cannot promote declared metadata into proof evidence.

### Signature morphisms

For this proposed bijective-renaming fragment, the first stock–flow morphisms
are total bijective renamings of:

- stock axes;
- internal-flow symbols;
- boundary-flow symbols;
- quantity kinds.

Translation renames every symbol and both matrices covariantly. Model reduct
uses the inverse maps contravariantly. The satisfaction condition must be
tested for every sentence family:

```text
target_model |= translate(sentence)
    iff
reduct(target_model) |= sentence
```

Aggregation, hiding, unit conversion, and non-bijective refinement remain out
of scope until their preservation obligations are explicitly defined.

## Work plan

### Phase 0 — Formal specification and source audit

Write the exact mathematical definitions before changing public APIs. Verify
the stock–flow equation and nullspace entailment against the existing research
sources on institutions, stoichiometric networks, ecological networks, and
stock-flow consistency collected in `reports/research-noether-institutions.md`
and `reports/research-existing-institutional-conservation-systems.md`.

Deliverables:

- a symbol table for stocks, scalar flows, domain process bundles, ports,
  kinds, matrices, traces, and
  sentences;
- exact well-formedness rules;
- the satisfaction definition for each first-slice sentence;
- the algebraic proof that the project's discrete open transition equation plus
  Mahdi et al.'s structural-linear premise `w^T S = 0` entails open balance;
- a written satisfaction-condition argument for conservative renamings;
- explicit separation of semantic violation, malformed model, derivation
  certificate, and runtime failure.

Acceptance criteria:

- every public term has one role and one owner repository;
- nonnegative boundary magnitudes and matrix-encoded directions are
  unambiguous;
- zero-transition and one-state behavior is non-vacuous and specified;
- no claim invokes Noether's variational theorem;
- paper-derived claims cite the actual source passage or equation used.

### Phase 1 — Exact stock–flow carrier in `conservation`

Implement exact identifiers, matrices, transition records, and validation
without depending on an ecological model.

Acceptance criteria:

- stock, flow, and boundary identifiers are nonblank and deterministic;
- matrices are total over their declared symbol axes and reject duplicates;
- as a project representation policy, all matrix coefficients and flow amounts
  are exact rationals so annihilation and satisfaction can be checked exactly;
- nonzero matrix effects connect only kind-compatible stocks and flows;
- transition records reject missing, extra, or wrong-kind values;
- consecutive records form a continuous trace;
- empty transition traces do not receive ordinary positive witnesses;
- structural errors cannot be represented as semantic violations.

Property contracts:

- declaration-order changes preserve named matrix semantics;
- arbitrary valid exact transitions satisfy their constructed equation;
- perturbing one state coordinate yields a first-axis residual violation;
- perturbing one process or boundary amount yields a reproducible violation;
- independent traces cannot share mutable state;
- malformed dimensions and symbol sets always fail before checking semantics.

### Phase 2 — Stock–flow sentences and typed verdicts

Add exact checkers for transition equations, linear flow constraints, and
boundary correspondence. Reuse graded state laws through an explicit sentence
variant rather than duplicating their semantics.

Acceptance criteria:

- every sentence validates against its signature before evaluation;
- every checker returns a positive witness or first-offense violation;
- transition violations identify transition and stock axis;
- flow-constraint violations identify transition and constraint;
- boundary violations identify transition, ledger axis, and port;
- the public result never collapses a complete suite to only one Boolean;
- witness construction is impossible for a structurally invalid model.

### Phase 3 — Stock–flow institution

Implement a new institution adapter over the shared carrier and prove its
executable categorical obligations for the supported morphism fragment.

Acceptance criteria:

- identity and composition preserve validated signatures;
- sentence translation renames all stocks, flows, ports, kinds, and matrix
  coordinates;
- model reduct renames all corresponding model data through inverse maps;
- identity and composition laws hold on generated examples;
- sentence translation is functorial on generated examples;
- model reduct is contravariantly functorial on generated examples;
- the satisfaction condition holds in both truth directions for every
  sentence family;
- as an illustrative fixture in addition to the generated categorical-law
  tests, renamed ecosystem and economic accounting models translate one
  literally shared source sentence rather than independently rebuilding
  analogous laws.

### Phase 4 — Checked balance entailments

Turn exact structural-linear nullspace results into sealed derivation
certificates and use the project's discrete open-system extension to derive
open-balance sentences from transition structure.

Acceptance criteria:

- certificate construction recomputes `w^T S` exactly;
- a non-null vector cannot receive a certificate;
- an effective zero coefficient vector cannot receive a certificate because it
  does not define a non-locally-constant first integral, and empty or fully
  cancelled inputs are rejected as validation cases;
- a derived basis is independent and has size `rows(S) - rank(S)`;
- project provenance distinguishes Mahdi-style stoichiometric derivations from
  separately sourced incidence derivations;
- derivation evidence is not copied from caller-selected provenance metadata;
- generated models satisfying the project's discrete transition equation also
  satisfy every certified derived open-balance sentence;
- as a project semantic contract, deliberately corrupted models may falsify
  transition and balance sentences without becoming structurally malformed;
- existing graded invariant evidence remains available as the derived trace
  projection.

### Phase 5 — Ecosim settled-transition traces

Compile each trophic topology into stable internal-flow and boundary symbols
and record the kernel's actual settled amounts for every accepted exact step.

The compiled flow vocabulary must distinguish at least:

- nutrient-limited producer growth;
- producer and consumer mortality;
- assimilated feeding;
- unassimilated feeding waste;
- detrital decomposition;
- nutrient input;
- harvest for each consumer.

Acceptance criteria:

- flow and boundary symbol order is stable across runs from one plan;
- one transition record is appended after each successful exact step;
- rejected steps append nothing and leave all observables unchanged;
- transition records use settled amounts, never merely requested amounts;
- before/after states are continuous across the complete trace;
- current stock and cumulative-ledger traces remain reproducible projections;
- the frozen cascade produces the same exact response signs as its predecessor;
- dense runs do not acquire exact institutional evidence by implication.

### Phase 6 — Ecosystem process and boundary laws

Compile a named law suite from the trophic topology:

- the complete state-transition equation;
- one feeding-partition constraint per feeding relationship;
- input-ledger correspondence;
- one harvest-ledger correspondence per consumer;
- current invariant, nonnegative-stock, and nondecreasing-ledger sentences;
- certified open material balance derived from the process matrix.

Acceptance criteria:

- a globally balanced but misrouted flow fails the transition sentence;
- a balanced feeding event with the wrong assimilation ratio fails its process
  constraint;
- an unchanged cumulative ledger with a nonzero settled port fails boundary
  correspondence;
- requested but source-limited harvest records and proves only actual output;
- every valid generated exact trophic run satisfies the complete suite;
- individual verdicts retain law name, relevant symbols, grade or sentence
  family, and typed witness;
- sentence order is stable and independent of declaration order.

### Phase 7 — Multi-kind ecosystem balances

Introduce a deliberately abstract, reviewable multi-kind ecosystem fixture
with separate carbon, nitrogen, phosphorus, and stored-energy coordinates. Do
not retrofit the existing material-equivalent cascade with unexplained
conversion factors.

Required sentences:

```text
delta(C_inside) = C_imported - C_exported
delta(N_inside) = N_imported - N_exported
delta(P_inside) = P_imported - P_exported
delta(E_stored) = E_input - E_heat - E_work - E_exported
```

Acceptance criteria:

- every axis and scalar flow has exactly one semantic quantity kind;
- matrix effects connect only compatible stock and flow kinds;
- a law cannot combine incompatible kinds;
- carbon, nitrogen, and phosphorus receive separate checked certificates;
- stored energy is an open balance with explicit dissipative ports, not an
  invariant of living and detrital stocks;
- omitting heat or export produces a visible failed balance rather than a
  silently absorbed residual;
- the fixture is labeled mathematical and synthetic, not empirically
  validated;
- the report states the chosen system boundary for every quantity kind.

### Phase 8 — Ecosystem constitutive and allocation laws

Add domain-owned kinetic sentences only after transition traces expose both
proposed and settled process amounts.

The first kinetic theory covers:

- nutrient-limited producer growth;
- Holling-type feeding proposals;
- biomass-proportional mortality;
- detritus-proportional decomposition;
- proportional source-limited settlement of competing withdrawals.

Acceptance criteria:

- proposed and settled flows are distinct symbols;
- kinetic evaluation uses the pre-transition state and declared elapsed time;
- exact audits interpret submitted binary64 parameters as exact rationals and
  say so explicitly;
- undefined division and invalid parameter domains are structural errors;
- a kinetic mismatch reports the first transition, process, proposed value,
  and expected value;
- settlement proves `0 <= settled <= proposed` for every withdrawal;
- withdrawals sharing a depleted source use the kernel's declared common
  proportional scale;
- zero actor or zero required resource cannot generate a positive proposal;
- kinetics remain an ecosim theory until another domain demonstrates the same
  expression and error semantics.

### Phase 9 — Paired-run experiment sentences

Turn the exact portion of the frozen cascade's hypotheses into sentences over
paired models sharing one compiled topology, parameter set, observation rule,
and intervention schedule.

The initial sentence family supports named signed comparisons such as:

```text
terminal_delta(producer) > epsilon
terminal_delta(herbivore) < -epsilon
```

Acceptance criteria:

- a paired model rejects mismatched signatures, parameters, axes, or horizons;
- sentence satisfaction is defined only for explicitly audited exact pairs;
- witnesses identify both runs, the response metric, observed exact delta, and
  threshold;
- violations retain the first failed named comparison;
- dense confirmatory comparisons remain numerical observations rather than
  exact institutional witnesses;
- cascade hypotheses are labeled conditional properties of the frozen model,
  not universal ecological laws;
- translation and reduct preserve comparative truth under conservative
  renaming.

### Phase 10 — Economics handoff and extraction decision

Update the economics mapping using the implemented stock–flow types:

| Implemented ecosystem construct | Candidate economic client |
|---|---|
| Stock axis and semantic kind | Account, inventory, currency, good, or claim |
| Settled internal process vector | Balanced transaction postings |
| Boundary port | Issuance, retirement, import, export, loss, or leakage |
| Transition equation | Stock-flow consistency equation |
| Flow constraint | Double-entry or transaction-bundle constraint |
| Boundary correspondence | Cumulative issuance, tax, or loss account |
| Checked nullspace certificate | Incidence-derived accounting identity |
| Proposed versus settled flow | Order versus executed trade or rationed allocation |
| Paired-run sentence | Audited policy counterfactual |

Do not implement economics in this plan. Decide only whether the completed
ecosystem APIs now provide a precise contract for its first client.

Extraction criteria remain conjunctive: both concrete clients must require the
same data type, law construction, translation, satisfaction evidence, and
error semantics. Vocabulary resemblance alone is insufficient.

### Phase 11 — Documentation, adversarial review, and gates

Produce separate reports for:

- formal stock–flow definitions and entailment proof;
- satisfaction-condition evidence;
- ecosystem process and boundary laws;
- multi-kind balance boundary choices;
- kinetic and allocation semantics;
- paired-run institutional evidence;
- updated economics handoff and extraction decision.

Adversarial review must attack at least:

- a balanced but misrouted transition;
- wrong feeding partitions that preserve total material;
- dishonest cumulative ledgers;
- caller-forged derivation provenance;
- cross-kind law construction;
- proposed/settled flow confusion;
- false comparative evidence from mismatched runs;
- accidental attribution of exact evidence to dense runs;
- satisfaction changes under symbol renaming.

Run formatting, linting, unit, integration, property, type, packaging, and media
history gates in every affected repository before completion. Dependency pins
in ecosim may move only after the corresponding upstream revisions are
committed, available from their configured remotes, and migrated together.

## TODO checklist

### Formal foundation

- [x] Define stock, flow, process-bundle, boundary, and kind symbol tables.
- [x] Specify exact matrices, nonnegative boundary magnitudes, and encoded directions.
- [x] Specify transition, flow-constraint, and boundary sentences.
- [x] Specify positive witnesses, violations, and structural errors.
- [x] Write the project open-system extension of the structural-nullspace entailment proof.
- [x] Write the conservative-renaming satisfaction-condition argument.
- [x] Verify mathematical claims against the cited sources.

### Conservation carrier

- [ ] Add validated flow and boundary identifiers.
- [ ] Add exact internal and boundary effect matrices.
- [ ] Add immutable exact transition records and traces.
- [ ] Add transition-equation checking and typed verdicts.
- [ ] Add linear flow-constraint checking and typed verdicts.
- [ ] Add boundary-correspondence checking and typed verdicts.
- [ ] Seal checked nullspace derivation certificates.
- [ ] Prove derived basis independence and `rows(S) - rank(S)` size.
- [ ] Derive open-balance sentences from valid certificates.
- [ ] Add generated carrier, checker, and certificate tests.

### Stock–flow institution

- [ ] Add stock–flow signatures and validated renamings.
- [ ] Add the stock–flow sentence sum type.
- [ ] Add stock–flow models and reducts.
- [ ] Translate every stock, flow, boundary, kind, and matrix coordinate covariantly.
- [ ] Delegate satisfaction to typed conservation checkers.
- [ ] Test identity, composition, translation, and reduct laws.
- [ ] Test the satisfaction condition for every sentence family.
- [ ] Demonstrate one shared neutral sentence in ecological and economic names.

### Ecosim transition evidence

- [ ] Compile stable trophic flow and boundary symbols.
- [ ] Record actual settled internal-flow and boundary-flow amounts.
- [ ] Preserve exact atomicity and independent-run contracts.
- [ ] Expose transition witnesses and violations through Rust.
- [ ] Expose faithful evidence summaries through Python.
- [ ] Retain exact-versus-dense evidence separation.
- [ ] Re-run the frozen cascade without changing its declared scenario.

### Ecosystem law suite

- [ ] Compile the complete transition equation.
- [ ] Compile one feeding-partition constraint per feeding edge.
- [ ] Compile input-ledger correspondence.
- [ ] Compile per-consumer harvest-ledger correspondence.
- [ ] Derive and check open material balance.
- [ ] Combine new verdicts with the existing graded state laws.
- [ ] Add generated valid-run satisfaction properties.
- [ ] Add balanced-but-misrouted and wrong-partition counterexamples.

### Multi-kind balances

- [ ] Declare a synthetic C/N/P/energy ecosystem fixture.
- [ ] Assign exactly one kind to every scalar axis and flow.
- [ ] Reject matrix effects between incompatible stock and flow kinds.
- [ ] Derive separate C, N, and P open balances.
- [ ] Declare energy input, heat, work, and export ports.
- [ ] Reject cross-kind sentence construction.
- [ ] Produce exact positive and negative evidence cases.
- [ ] Document every system boundary and non-validation claim.

### Constitutive laws

- [ ] Record proposed flows separately from settled flows.
- [ ] Define domain-owned exact kinetic sentence semantics.
- [ ] Check producer-growth, feeding, mortality, and decomposition proposals.
- [ ] Check settlement bounds and proportional source limitation.
- [ ] Prove zero-actor and zero-required-resource cases cannot create flow.
- [ ] Add generated kinetic and allocation counterexamples.
- [ ] Keep nonlinear ecosystem theory out of shared crates pending a second client.

### Paired-run sentences

- [ ] Define exact paired-model well-formedness.
- [ ] Define named signed terminal-response sentences.
- [ ] Return exact comparative witnesses and violations.
- [ ] Institutionalize the frozen cascade's selected exact sign claims.
- [ ] Prove comparative satisfaction under conservative renaming.
- [ ] Keep dense comparisons explicitly numerical and unaudited.

### Handoff and completion

- [ ] Update the ecosystem-to-economics mapping from implemented types.
- [ ] Decide whether a concrete economics client is sufficiently specified.
- [ ] Decide whether any additional shared extraction is justified.
- [ ] Complete adversarial review in every affected repository.
- [ ] Document commands sufficient to reproduce every committed result.
- [ ] Verify that no PDF or PNG artifact is tracked or introduced into history.
- [ ] Run all Rust, Python, property, type, package, and diff gates.
- [ ] Commit only scoped project artifacts; preserve append-only notes uncommitted.

## Definition of done

This plan is complete when the exact trophic runtime proves not only that its
states balance, but that every accepted state change is exactly realized by
its declared settled processes and boundary flows; feeding partitions and
boundary ledgers are individually honest; open balances are derived through
sealed exact certificates; a synthetic multi-kind ecosystem distinguishes
C/N/P conservation from open dissipative energy balance; ecosystem kinetics
and allocation are inspectable domain sentences; selected exact cascade
comparisons are paired-model sentences; all supported translations preserve
satisfaction; and the economics handoff identifies the next concrete client
without prematurely extracting its unproven abstractions.
