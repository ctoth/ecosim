# Institutionally Certified Ecosystem Experiment Plan

## Purpose

Build the first complete ecosystem experiment that exhibits recognizable
trophic behavior and produces executable institutional evidence for every run.
The same work should reveal the genuine reuse seam for later economic models
without extracting another generic library prematurely.

## Design criteria

This plan uses only the criteria established for the project:

- exhibit actual ecosystem behavior, not merely balance a ledger;
- exercise invariant, nonnegative, and nondecreasing conservation sentences;
- preserve an exact evidence path and a high-throughput dense experiment path;
- create a concrete seam that economics can reuse later;
- wait for a second concrete client before extracting more generic machinery.

## Scope

The vertical slice is a nutrient–producer–herbivore–predator system with
detrital recycling, nutrient input, and consumer harvest. It will compare an
unforced control with nutrient-pulse and predator-harvest interventions, expose
the resulting trophic responses, and attach typed conservation evidence to the
exact runs.

The work includes:

- institutional traces for exact configurable trophic networks;
- a compiled suite of graded sentences derived from network topology;
- property-based verification of trace and state-transition contracts;
- dense ensemble exploration followed by frozen confirmatory scenarios;
- exact replay of selected scenarios for institutional evidence;
- a documented mapping from the resulting concepts to a future economics
  client.

The work does not include:

- empirical validation for a real ecosystem;
- claiming that binary64 trajectories constitute exact proofs;
- implementing the economics client;
- extracting a new category-theory or conservation abstraction;
- tuning an implementation until it reproduces a desired paper value.

## Intended architecture

### One compiled plan, two execution roles

`ExactTrophicNetworkPlan` remains the evidence-oriented reference. Its compiled
topology will also own or construct the immutable signature and graded sentence
suite used to audit its runs.

`DenseTrophicNetworkPlan` remains the numerical workload path. It runs
exploratory and confirmatory ensembles using the same declared topology and
process parameters, but its floating-point balance residual is numerical
diagnostic evidence rather than institutional satisfaction.

Selected dense scenarios are replayed through the exact path using the same
submitted binary64 values interpreted as exact rationals. This certifies the
declared scenario and forcing schedule; it does not assert bit-identical dense
and exact trajectories.

### Trace signature

Each exact trace state contains:

- one axis for every compiled physical stock, in `stock_names` order;
- `cumulative_input`, containing actual settled boundary input;
- `cumulative_output`, containing actual settled harvest/export.

The initial state is recorded before the first step. A new state is appended
after every successfully committed step. A rejected step appends nothing.
Requested but source-limited harvest is never recorded as output; evidence uses
the amount actually settled by the conservation kernel.

### Graded sentence suite

The compiled suite contains:

1. **Invariant total material**

   ```text
   sum(physical stocks) - cumulative_input + cumulative_output
   ```

   Its exact value must remain equal to the initial total.

2. **Nonnegative physical stocks**

   One sentence per physical stock asserts that its axis remains at least zero
   throughout the trace.

3. **Nondecreasing boundary ledgers**

   Separate sentences assert that `cumulative_input` and
   `cumulative_output` never decrease.

The public evidence result reports the `LawVerdict` for every sentence, keeping
positive witnesses and first-offense violations rather than collapsing the
result to one Boolean.

## Work plan

### Phase 1 — Exact trophic traces

Add the trace representation at the configurable trophic-network boundary.
Compile stable evidence axes from the network plan and record the initial and
post-commit states of each exact run.

Acceptance criteria:

- axis order and names are stable across runs from one plan;
- trace length is `successful_steps + 1`;
- a failed step leaves stocks, ledgers, time, and trace unchanged;
- independent runs from one plan own independent traces;
- trace values use settled state and boundary amounts.

### Phase 2 — Institutional evidence

Compile the signature and sentence suite once per exact plan. Construct a
`TraceModel` from a run and evaluate every sentence through
`ConservationInstitution`.

Acceptance criteria:

- invariant evidence returns an invariant witness for valid runs;
- every physical stock returns a nonnegative witness;
- both cumulative boundary axes return nondecreasing witnesses;
- a deliberately corrupted trace can satisfy the invariant while failing a
  stock sentence;
- a reversed cumulative ledger produces a first-decrease violation;
- callers can inspect individual verdicts and their associated axis names.

### Phase 3 — Property-based contracts

Generate valid network structures, initial states, forcing schedules, and
rejected operations. Exercise exact institutional evidence and dense numerical
contracts separately.

Properties to test:

- arbitrary valid exact runs satisfy every compiled sentence;
- arbitrary rejected steps are fully atomic, including trace state;
- declaration-order changes do not change named stocks or evidence;
- source-limited feeding preserves assimilation and detrital-waste partitions;
- source-limited harvest records actual output, not requested output;
- independent runs from one compiled plan cannot contaminate each other;
- dense batched execution matches repeated dense stateful execution;
- dense trajectories remain finite and nonnegative;
- dense open-balance residuals remain within the preregistered numerical
  tolerance.

Shrunk failures must retain the generated topology, initial state, forcing
schedule, and sentence verdict needed to reproduce them.

### Phase 4 — Three-level trophic experiment

Declare a producer, herbivore, and predator, with the herbivore feeding on the
producer and the predator feeding on the herbivore. Nutrient and detritus close
the internal material cycle; nutrient input and harvest remain the only model
boundary flows.

Run three comparisons:

1. predator-present versus predator-free control;
2. nutrient pulse versus unforced control;
3. predator-harvest press versus predator-present control.

Candidate qualitative hypotheses:

- predator presence suppresses herbivore abundance and releases the producer;
- a nutrient pulse propagates into producer and then consumer compartments;
- sustained predator harvest releases the herbivore and reduces the producer
  relative to the predator-present control.

The hypotheses are conditional on a declared parameter regime. Dense ensemble
search is exploratory: it may identify a regime and observation window, but it
must not count as confirmation. Before confirmatory execution, freeze:

- the complete network specification and initial stocks;
- intervention magnitudes, durations, and timing;
- observation times and response metrics;
- expected response signs;
- numerical tolerances;
- exact-replay scenarios.

Store the frozen scenario in reviewable source data rather than embedding an
unexplained parameter tuple inside a test.

Acceptance criteria:

- the frozen comparisons exhibit their preregistered qualitative responses;
- every exact replay returns positive witnesses for the applicable sentences;
- dense and exact paths agree on the response signs being claimed;
- the report separates exploratory findings from confirmatory results;
- model behavior is described as a property of this declared model, not as
  validation of a real ecosystem.

### Phase 5 — Experiment API and results

Expose a small experiment layer that accepts a compiled plan, initial states,
and immutable intervention schedules. Return trajectories and named response
observations without embedding one food-web topology in the conservation core.

Acceptance criteria:

- controls and interventions use identical compiled topology and parameters;
- response metrics are named and signed relative to their controls;
- ensemble inputs retain stock and consumer axis metadata;
- exact evidence is attached to audited scenarios without being implied for
  unaudited dense runs;
- reports are reproducible from committed scenario data and commands.

### Phase 6 — Economics handoff

After the ecosystem slice is complete, document—not implement—the corresponding
economic concepts:

| Ecosystem concept | Candidate economic analogue |
|---|---|
| Physical stock | Account or inventory balance |
| Internal feeding/transfer | Transaction between accounts |
| Boundary input/output | Issuance, import/export, or leakage |
| Invariant total | Closed-currency or inventory identity |
| Nonnegative stock sentence | No-overdraft or solvency constraint where applicable |
| Nondecreasing cumulative output | Cumulative tax, waste, or loss ledger |
| Intervention schedule | Policy or market shock |
| Exact evidence replay | Audited accounting scenario |

Extraction into a shared library becomes eligible only when both clients need
the same data type, law construction, verdict handling, and error semantics.
Similar vocabulary alone is not sufficient.

## TODO checklist

### Exact evidence foundation

- [x] Define stable trophic trace axes, including cumulative input and output.
- [x] Record the initial exact trace state.
- [x] Append one trace state after each successful exact step.
- [x] Prove rejected steps leave the trace unchanged.
- [x] Compile the conservation signature once per exact plan.
- [x] Compile the invariant total-material sentence.
- [x] Compile one nonnegative sentence per physical stock.
- [x] Compile nondecreasing input and output sentences.
- [x] Return named typed verdicts for the complete sentence suite.

### Verification

- [x] Test invariant success alongside a deliberately negative stock failure.
- [x] Test first-offense reporting for a decreasing cumulative ledger.
- [x] Test actual settled output under source-limited harvest.
- [x] Add generated exact-run satisfaction properties.
- [x] Add generated rejected-step atomicity properties.
- [x] Add declaration-order and independent-run properties.
- [x] Retain dense batch/stateful parity properties.
- [x] Declare and test the dense residual tolerance.

### Ecosystem behavior

- [ ] Declare the three-level trophic topology.
- [ ] Define predator-free and predator-present controls.
- [ ] Define the nutrient-pulse intervention.
- [ ] Define the predator-harvest press intervention.
- [ ] Run exploratory dense parameter ensembles.
- [ ] Record exploratory results separately from confirmation.
- [ ] Freeze the confirmatory scenario and thresholds.
- [ ] Run confirmatory dense comparisons.
- [ ] Replay selected scenarios through the exact path.
- [ ] Produce the behavior and institutional-evidence report.

### Handoff and completion

- [ ] Document commands sufficient to reproduce the committed results.
- [ ] Verify that no PDF or PNG artifact is tracked or introduced into history.
- [ ] Run formatting, linting, Rust, Python, type, and property-test gates.
- [ ] Record the ecosystem-to-economics concept mapping.
- [ ] Decide whether two concrete clients justify a shared extraction.

## Definition of done

This plan is complete when one frozen three-level experiment exhibits the
declared trophic responses, its selected scenarios carry inspectable exact
witnesses for invariant, nonnegative, and nondecreasing sentences, generated
tests cover the transition and trace contracts, and the economics handoff
identifies reuse candidates without introducing a speculative shared library.
