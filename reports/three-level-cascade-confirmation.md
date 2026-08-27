# Frozen Three-Level Cascade Confirmation

## Scope

This report concerns the abstract material-equivalent model declared in
`src/ecosim/data/three_level_cascade.json`. It demonstrates behavior of that
model; it is not empirical validation of a real ecosystem. Dense trajectories
are numerical results. Only the separately identified short exact runs carry
institutional satisfaction evidence.

The model contains nutrient, producer, herbivore, predator, and detritus
stocks. The herbivore consumes the producer and the predator consumes the
herbivore. Nutrient input and consumer harvest are the only boundary flows.

## Reproduction

Build the native module and execute the frozen scenario:

```powershell
uv run maturin develop
uv run python scripts/run_three_level_cascade.py
uv run pytest tests/test_trophic_experiments.py -q
```

The runner reads the committed JSON, runs all dense comparisons and exact
audits, and emits the observations, thresholds, law names, grades, axes, and
satisfaction results as JSON.

## Dense confirmation

The frozen dense schedule uses 200 steps of 0.05. The nutrient pulse adds 10 at
step 20. Predator harvest removes 0.01 per step from steps 50 through 149. All
eight preregistered terminal thresholds passed.

| Comparison | Stock | Required | Observed |
|---|---|---:|---:|
| Predator presence | Producer | at least 0.5 | 0.731855095818311 |
| Predator presence | Herbivore | at most -0.5 | -0.6615009179914821 |
| Nutrient pulse | Producer | at least 5.0 | 9.15968936929682 |
| Nutrient pulse | Herbivore | at least 0.1 | 0.4474254632422756 |
| Nutrient pulse | Predator | at least 0.000001 | 0.00001269160957462212 |
| Predator harvest | Producer | at most -0.1 | -0.15751910057282714 |
| Predator harvest | Herbivore | at least 0.1 | 0.19231253454643138 |
| Predator harvest | Predator | at most -0.5 | -1.0169852146498948 |

The dense boundary-account tests use absolute and relative tolerances of
`256 * f64::EPSILON`, each `5.684341886080802e-14`, scaled by the conserved
stock magnitude.

## Exact replay and institutional evidence

Selected audits use five exact rational steps of 0.05. The short horizon keeps
exact nonlinear rational arithmetic tractable while testing each claimed
response direction and every institutional sentence. It is not described as a
bit-identical replay of the 200-step dense trajectory.

The dense and exact paths agree on all eight claimed response signs:

| Comparison | Stock | Required sign | Exact delta |
|---|---|---:|---:|
| Predator presence | Producer | positive | 0.00011622840918867894 |
| Predator presence | Herbivore | negative | -0.0042498086632152265 |
| Nutrient pulse | Producer | positive | 0.008194550941054146 |
| Nutrient pulse | Herbivore | positive | 0.0000178366686727216 |
| Nutrient pulse | Predator | positive | 0.00000000021768387092890862 |
| Predator harvest | Producer | negative | -0.000001150288689188983 |
| Predator harvest | Herbivore | positive | 0.0000843888901780332 |
| Predator harvest | Predator | negative | -0.0500085692695319 |

Every exact audit has an initial state plus five committed states and satisfies
all eight compiled sentences:

- invariant total material;
- nonnegative nutrient, producer, herbivore, predator, and detritus;
- nondecreasing cumulative input;
- nondecreasing cumulative output.

The pulsed exact run records cumulative input 10.0. The harvested exact run
records cumulative output 0.05. The evidence API retains each named law, grade,
axis, satisfaction result, and the typed Rust `LawVerdict`; the Python report
surface intentionally exposes a compact summary without implying evidence for
the dense runs.
