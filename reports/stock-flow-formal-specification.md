# Institutional Stock–Flow Formal Specification

## Status

This document fixes the mathematical and API semantics for Phases 0–4 of
`PLAN.md`. It is normative for the first implementation slice. Names may be
refined during Rust implementation, but changing a semantic rule requires an
explicit update here.

The specification describes exact finite transition traces. It does not claim
empirical adequacy, numerical proof for binary64 runs, or a variational Noether
theorem.

## Source obligations

The institution structure follows Goguen and Burstall's Definition 1:
signatures form a category; sentence translation is covariant; model reduct is
contravariant; and satisfaction is invariant under signature morphisms. The
set-valued model implementation in this project is an explicit specialization
that does not yet represent model homomorphisms. [Goguen and Burstall 1992,
PDF pp. 7–8; journal pp. 101–102]

The linear-conservation result follows the structural stoichiometric contract:
if a row vector `w` lies in the left kernel of the internal effect matrix `S`,
then `w x` is unchanged by internal flows. This derives structural linear laws,
not all kinetic or nonlinear first integrals. [Mahdi et al. 2017, PDF pp. 6–7]

The source-by-source grading and any wording corrections are recorded in
`reports/stock-flow-source-verification.md`.

## Symbols and kinds

Let:

- `K` be a finite nonempty set of semantic quantity kinds;
- `X` be a finite nonempty set of scalar stock axes;
- `F` be a finite set of scalar internal flow channels;
- `P` be a finite set of scalar boundary-flow channels;
- `L` be a finite set of cumulative boundary-ledger axes.

Every scalar symbol has exactly one kind:

```text
kind_X : X -> K
kind_F : F -> K
kind_P : P -> K
kind_L : L -> K
```

As a project typing restriction, a domain process may compile to several scalar
flow channels. For example, a
multi-kind feeding process may have distinct carbon, nitrogen, phosphorus, and
energy channels. A scalar matrix column never silently converts one kind into
another.

Identifiers are nonblank and unique within their symbol class. All public
iteration orders are canonical and deterministic. Declaration order may not
change the named semantics of a compiled structure.

## Stock and flow topology

The project's first transfer-only carrier specializes a general stoichiometric
matrix to one-source/one-target unit incidence. Each internal flow `f in F` has
a source and target:

```text
source_F(f) in X
target_F(f) in X
source_F(f) != target_F(f)
kind_X(source_F(f)) = kind_F(f) = kind_X(target_F(f))
```

This project's external-flow ports are distinct from the fixed-concentration
"boundary metabolites" used by Maarleveld et al. Each boundary flow `p in P`
is exactly one of:

- input: no internal source and one target stock;
- output: one source stock and no internal target.

Disconnected boundary flows are malformed. Boundary inputs and outputs use
nonnegative magnitudes. Direction is encoded by the boundary effect matrix;
callers do not submit signed amounts.

As a project representation policy, the internal effect matrix `S` and
boundary effect matrix `B` are exact rational matrices:

```text
S in Q^(X x F)
B in Q^(X x P)
```

For the first carrier, topology compilation derives incidence coefficients:

```text
S[x, f] = -1  when x = source_F(f)
S[x, f] = +1  when x = target_F(f)
S[x, f] =  0  otherwise

B[x, p] = +1  for an input targeting x
B[x, p] = -1  for an output sourced at x
B[x, p] =  0  otherwise
```

Maarleveld et al. support general signed, nonunit stoichiometric coefficients.
General rational stoichiometric matrices may be admitted later, but exact
rational representation, explicit kind validation, and separate scalar
channels are project policies. Ecosim's current assimilation and waste
partition is represented by separate scalar transfer channels, not by a
kind-changing matrix coefficient.

## Exact transition model

An exact accepted transition record contains:

```text
before              x_k in Q_nonnegative^X
after               x_(k+1) in Q_nonnegative^X
requested_internal  q_k in Q_nonnegative^F
settled_internal    f_k in Q_nonnegative^F
requested_boundary  r_k in Q_nonnegative^P
settled_boundary    b_k in Q_nonnegative^P
ledger_before       ell_k in Q_nonnegative^L
ledger_after        ell_(k+1) in Q_nonnegative^L
```

The requested and settled vectors are retained separately. A transition trace
is a finite sequence of accepted records. Adjacent records must be continuous:

```text
record[k].after = record[k + 1].before
record[k].ledger_after = record[k + 1].ledger_before
```

Malformed vector sizes, missing or extra symbols, kind mismatches, negative
amounts, and discontinuous records are structural errors. They are not models
that semantically violate a sentence.

A trace with no accepted transition cannot receive an ordinary positive
satisfaction witness. It produces a structural `TooShort`-style result. A
rejected runtime operation creates no transition record; rejection atomicity is
an operational property of the runtime, not a sentence satisfied by a model.

## Settlement contract

For every scalar internal or boundary flow:

```text
0 <= settled <= requested
```

Boundary inputs have no internal source and therefore settle at the requested
amount after validation. Withdrawals compete only with other withdrawals from
the same pre-transition source stock. For each source `x`, define:

```text
demand_x = sum(requested[j] for withdrawals j sourced at x)

alpha_x = 1                         when demand_x = 0 or demand_x <= before[x]
alpha_x = before[x] / demand_x      otherwise

settled[j] = requested[j] * alpha_x
```

Thus all withdrawals from one depleted source receive the same exact
proportional scale. Boundary inputs are not available to withdrawals until the
next transition. These rules specify the existing `conservation-dynamics`
kernel and later become allocation sentences in the ecosystem theory.

## Sentence family

### Transition equation

For every accepted transition:

```text
after - before = S settled_internal + B settled_boundary
```

Satisfaction is exact and axis-wise. A positive witness records the signature
identity, transitions checked, axes checked, and exact zero residual. A
violation records the first transition in trace order and first axis in
canonical order, plus observed delta, accounted delta, and exact residual.

### Linear flow constraint

A linear flow constraint over scalar channels of one kind is:

```text
sum(c_i * settled[channel_i]) = expected
```

where every `c_i` and `expected` is exact. The first implementation uses
`expected = 0`. Cross-kind constraints are structurally invalid unless a later
domain theory supplies an explicitly typed conversion.

For trophic feeding with assimilation efficiency `eta`, the assimilated and
waste channels satisfy:

```text
(1 - eta) * assimilated - eta * waste = 0
```

Because both withdrawals share the same source, proportional settlement
preserves this requested ratio exactly.

### Boundary correspondence

A boundary-correspondence sentence maps one cumulative ledger to a nonempty
set of same-kind boundary ports and checks:

```text
ledger_after - ledger_before
    = sum(settled_boundary[p] for mapped ports p)
```

Inputs and outputs use distinct ledgers. Ecosim may expose one aggregate output
ledger and additional per-consumer harvest ledgers; each correspondence states
exactly which ports it accounts for. Nondecreasing graded laws remain separate
sentences over the ledger state.

### Graded state law

The stock–flow sentence sum embeds the existing `GradedLaw` unchanged. Its
carrier state is the projection of each transition trace to the initial state
followed by committed `after` states, including declared cumulative ledger
axes. No new checker reimplements invariant, nonnegative, or nondecreasing
semantics.

## Typed verdicts

Every semantically checked sentence returns exactly one of:

```text
Satisfied(typed witness)
Violated(first-offense typed violation)
```

Structural validation returns an error before semantic checking. Suite-level
convenience may report `all_satisfied`, but public evidence retains every named
sentence and full typed verdict.

Required violation coordinates are:

| Sentence | First-offense coordinates |
|---|---|
| Transition equation | transition, stock axis, observed delta, accounted delta, residual |
| Linear flow constraint | transition, constraint, observed value, expected value |
| Boundary correspondence | transition, ledger, mapped ports, observed increment, settled total |
| Graded state law | existing typed graded violation |

## Structural open-balance entailment

Extending Maarleveld et al.'s closed left-nullspace relation with the project's
discrete open carrier, let `w in Q^X` be an exact column coefficient vector over
stock axes of one kind. A checked nullspace certificate exists only when exact
multiplication proves:

```text
w^T S = 0
```

For every model satisfying the transition equation:

```text
w^T (after - before)
    = w^T S settled_internal + w^T B settled_boundary
    = w^T B settled_boundary
```

This is an entailment from a transition sentence and a checked algebraic
certificate to an open-balance sentence. It is not inferred from caller-chosen
`Provenance` metadata.

To reuse the existing state-trace checker, cumulative boundary ledgers may be
introduced as axes. For a boundary port `p`, its ledger coefficient is the
negative of `w^T B[:, p]`. The resulting extended balance form is invariant
exactly when the transition equation and boundary correspondence hold.

As project verification rules, certificate construction must:

1. validate all axes and kinds;
2. canonicalize the nonzero exact vector;
3. recompute `w^T S` exactly;
4. reject empty, fully cancelled, or non-null forms;
5. retain the exact matrix identity and structural origin;
6. keep runtime satisfaction evidence separate.

This derives structural linear conservation laws only. Kinetic or nonlinear
first integrals remain outside the certificate API.

## Stock–flow institution

### Existing graded-layer audit

The current `institution` trait exposes explicit source and target signatures,
identity, composition, sentence translation, model reduct, and satisfaction.
Its `laws` module observes signature identity, sentence identity/composition,
model identity/contravariant composition, and both sides of the satisfaction
square. `institution-conservation` tests these obligations for invariant,
nonnegative, and nondecreasing sentences in both satisfying and violating
models, including one literally shared neutral source law translated into
ecological and economic names.

This is an executable set-valued/discrete-model specialization, not the full
category-valued model functor with nonidentity model homomorphisms described in
Goguen and Burstall. The new stock–flow Institution must retain the same
explicit limitation and generated law coverage.

### Signature

A stock–flow signature contains the typed symbol sets, exact effect matrices,
ledger declarations, and their port mappings. It is nonempty and completely
validated at construction.

### Morphism

The first morphisms are total bijective, kind-preserving renamings of stocks,
internal flows, boundary flows, ledgers, and kinds. Matrix entries and ledger
mappings translate covariantly with their symbols.

Aggregation, hiding, unit conversion, and non-bijective refinement are not
first-slice morphisms.

### Sentence translation

`Sen(sigma)` renames every referenced symbol and translates embedded graded
forms. Coefficients, grades, comparison operators, and sentence family remain
unchanged.

### Model reduct

`Mod(sigma)` uses the inverse bijections to read a target transition model
under the source vocabulary. Before/after states, requested/settled vectors,
and ledger states are all reduced.

### Satisfaction condition

For every supported morphism `sigma : Sigma -> Sigma'`, source sentence `phi`,
and target model `M'`:

```text
M' |= Sen(sigma)(phi)
    iff
Mod(sigma)(M') |= phi
```

Both evaluations reduce to the same exact rational equalities after bijective
renaming. The implementation must property-test both truth directions,
identity, composition, sentence functoriality, and contravariant reduct
functoriality for every sentence variant.

## Ecosystem theory boundary

The shared institution knows exact scalar flows, linear constraints, ledger
correspondence, and graded state forms. Ecosim retains:

- names and grouping of growth, feeding, mortality, decomposition, input, and
  harvest processes;
- compilation of feeding partition constraints;
- kinetic expressions over pre-transition state and elapsed time;
- proposed-versus-settled allocation sentences;
- paired-run response metrics and conditional cascade hypotheses.

These domain sentences are not promoted to shared crates until a concrete
economic client demonstrates identical syntax, satisfaction, translation, and
error semantics.

## Multi-kind boundary

Maarleveld et al. support atom-wise balancing, including C, N, and P. The first
synthetic multi-kind fixture adds the project policy of separate scalar C, N,
P, and stored-energy axes and flow channels. Each element receives its own
structural open balance. Stored energy includes explicit input, heat, work, and
export ports and is not asserted invariant over internal biomass alone.

As a project type-safety policy, cross-kind stoichiometric ratios require
explicit typed conversions in the ecosystem presentation. They do not weaken
the rule that the shared incidence matrix connects only same-kind scalar stocks
and flows.

## Dense evidence policy

Binary64 runs may expose named numerical residuals under declared absolute and
relative tolerances. They do not construct exact stock–flow models, exact
derivation certificates, or institutional satisfaction witnesses. A dense
result and an exact result may agree on a response sign without being the same
model.

## Required negative models

The implementation and adversarial review must retain structurally valid
models that falsify each sentence family:

- balanced total with one stock delta misrouted;
- correct stock delta with one settled flow amount corrupted;
- feeding channels with the wrong assimilation/waste ratio;
- settled boundary input with an unchanged input ledger;
- output ledger increment attributed to the wrong harvest port;
- non-null `w` presented as though it had a nullspace certificate;
- same numerical arrays interpreted under mismatched signatures;
- renamed sentence whose model was not correspondingly reduced.

These must produce semantic violations or certificate rejection, not panics or
vacuous witnesses.
