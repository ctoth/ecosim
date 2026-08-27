# ecosim

Exact, institution-aware ecosystem simulation with a Rust core and a Python
interface built by Maturin.

The foundation slice is an energy ledger with two internal compartments
and one cumulative external-flow coordinate. Its transition matrix derives the
exact law

```text
left + right - net_external = constant
```

from a rational left nullspace. Every accepted event appends a trace state,
and the trace is checked through the conservation institution. Consequently,
the ordinary ledger presentation is also exact:

```text
initial_stock + inputs - outputs = final_stock + residual
```

The residual is zero for a satisfying run. Internal transfers change neither
side of that equation.

The first ecological slice is a well-mixed, discrete-time food web with four
stocks:

```text
boundary -> nutrient -> producer -> consumer -> boundary
                ^          |           |
                |          v           v
                +------- detritus <-----+
```

Nutrient-limited growth moves material into producers; producer-limited grazing
moves it into consumers; mortality moves both living stocks into detritus; and
decomposition recycles detritus into nutrient. Nutrient input and consumer
harvest cross the model boundary. All mechanisms propose flows from the same
pre-step state and the shared `conservation-dynamics` kernel settles competing
withdrawals proportionally, exactly, and without proposal-order effects.

`FoodWeb` compiles this topology once and advances an exact rational state.
`DenseFoodWeb` uses the same compiled topology with finite `f64` state for
numerical workloads. Both expose stock values, process-level settled flows,
time, cumulative inputs and outputs, and a balance verdict. The exact facade is
the evidence-oriented reference; the dense facade is the explicitly numerical
execution path.

`simulate_food_web` runs a batch of dense trajectories from a NumPy array with
shape `(batch, 4)` and returns a NumPy array with shape `(batch, steps + 1, 4)`.
Optional nutrient-input and harvest arrays have shape `(batch, steps)`. Inputs
are validated and copied before native execution releases Python's GIL; the
inner loop uses the no-report settlement path.

Python's immutable `FoodWebParameters` validates the seven process parameters
once and can be supplied to `FoodWeb`, `DenseFoodWeb`, or `simulate_food_web`.
The three paths therefore use the same growth, grazing, mortality, and
decomposition semantics while supporting parameter sweeps and controlled
perturbation experiments.

`BoundaryPerturbation.pulse` and `.press` construct immutable nutrient-input
and consumer-harvest interventions. `simulate_food_web_response` runs the
intervention beside its unforced counterfactual and returns signed stock
responses with peak, terminal, and recovery observations. This makes trophic
propagation explicit: enrichment can propagate from nutrient to producers and
consumers, while sustained consumer harvest can release producers from
grazing.

## Configurable trophic networks

`TrophicNetworkSpec` declares any number of nutrient-limited producers,
consumers, and directed feeding relationships. Compilation creates nutrient
and detritus pools; producer growth; assimilated and unassimilated feeding;
biotic mortality; detrital recycling; nutrient input; and per-consumer harvest
flows. All process requests are evaluated from the same pre-step state and
settled simultaneously, so competing producers and consumers share depleted
resources without declaration-order priority.

`TrophicNetwork` executes the compiled structure with exact rational
arithmetic. `DenseTrophicNetwork` uses the same topology and process laws with
finite `f64` state. Feeding efficiency is represented by paired resource flows
to the consumer and detritus, so proportional source limitation preserves the
declared assimilation/waste partition as well as total material. The model
still uses the deliberately abstract material-equivalent currency described in
the verification scope below.

`TrophicNetworkPlan` and `DenseTrophicNetworkPlan` compile that immutable
structure separately from mutable run state, so one plan can start independent
runs without rebuilding its topology and process graph. The dense plan also
runs dynamic NumPy ensembles: `initial_states` has shape `(batch, stocks)`,
optional nutrient inputs have shape `(batch, steps)`, optional harvests have
shape `(batch, steps, consumers)`, and the returned trajectory has shape
`(batch, steps + 1, stocks)`. `stock_names` and `consumer_names` define the
array-axis order. Inputs are validated and copied before native execution
releases Python's GIL; the ensemble loop uses the no-report settlement path.

## Paper benchmarks

`ecosim_core::paper_models::almonacid_2020` independently implements the NPZD
production matrix, two-stage MPRK update, exact Gaussian nutrient pulse, exact
sinking subflow, and discrete ledger from Mata Almonacid and Medel (2020). Its
conservation, positivity, forcing, and ledger obligations are executable tests.
The reported Figure 10 primary-production value is retained as an ignored,
currently failing acceptance target because of an unresolved source/model
reproduction discrepancy. Time-step refinement and left-, midpoint-, and
right-sampling at the discontinuous day/night boundary converge near 33.7,
rather than the reported 24.38, so those numerical conventions do not explain
the difference. The implementation has not been tuned to the reported number.

The selected Heinle and Flemming benchmark is not implemented yet. Legitimate
full-text retrieval reached publisher metadata and purchase/institutional-access
pages only, so its equations and parameters have not been inferred from an
abstract or fabricated from adjacent literature.

## Verification scope

The exact ledger, stock-flow kernel, and property tests verify the stated accounting formalism:
classified internal transfers cancel globally, boundary flows change the total,
and rejected events are atomic. Graded institutional sentences separately
state that the derived total-energy form is invariant and that each internal
stock remains nonnegative; a balanced total therefore cannot conceal a
negative compartment. They do not establish that an ecosystem is at
equilibrium, that the selected compartments and mechanisms are empirically
adequate, or that the model is validated for a real-world use. Such a claim
requires a separately stated purpose, performance criteria, operating context,
and empirical evidence. The four stocks currently share one deliberately
model-specific material-equivalent currency; this is not a claim that real
nutrient mass, biomass, and energy are generally interchangeable.

## Development

```powershell
uv sync
uv run maturin develop
uv run pytest
uv run pyright
cargo test --workspace --all-targets
```

Cross-repository Rust dependencies are pinned to immutable reviewed Git
revisions of the public `conservation` and `institution` repositories.
