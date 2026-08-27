# Exploratory Three-Level Cascade Search

## Status and selection rule

This is exploratory output. It was used only to find a declared parameter
regime in which the three qualitative hypotheses in `PLAN.md` all have the
required signs:

- predator presence increases producer and decreases herbivore;
- nutrient input increases producer, herbivore, and predator;
- predator harvest decreases producer, increases herbivore, and decreases
  predator.

No additional ranking criterion was applied. In particular, the grid does not
claim biological realism or choose an optimum. All 48 tuples in the declared
grid met those sign conditions. The subsequently frozen tuple is simply one
qualifying regime, and its confirmation thresholds were committed to source
data before the confirmatory execution.

## Declared grid

The fixed features were one producer, one herbivore, one predator, 0.7 feeding
assimilation efficiency, producer growth 0.5, producer mortality 0.01,
herbivore mortality 0.02, predator mortality 0.01, decomposition 0.1, 200
steps, and an elapsed interval of 0.05 per step.

| Parameter | Explored values |
|---|---|
| Herbivore feeding rate | 0.3, 0.4 |
| Predator feeding rate | 0.01, 0.02, 0.03 |
| Predator half-saturation | 1.0, 3.0 |
| Initial predator stock | 1.0, 2.0 |
| Harvest per step | 0.005, 0.01 |

The Cartesian product contains 48 tuples. Run it with:

```powershell
uv run python scripts/explore_three_level_cascade.py
```

The script reports every matching tuple as JSON and neither reads nor writes
the frozen confirmatory file.

## Frozen tuple observed during exploration

The tuple later frozen in `src/ecosim/data/three_level_cascade.json` has
herbivore feeding rate 0.4, predator feeding rate 0.02, predator
half-saturation 1.0, initial predator stock 1.0, and harvest 0.01 per step.

| Comparison | Stock | Exploratory terminal delta |
|---|---:|---:|
| Predator presence | Producer | 0.731855095818311 |
| Predator presence | Herbivore | -0.6615009179914821 |
| Nutrient pulse | Producer | 9.15968936929682 |
| Nutrient pulse | Herbivore | 0.4474254632422756 |
| Nutrient pulse | Predator | 0.00001269160957462212 |
| Predator harvest press | Producer | -0.15751910057282714 |
| Predator harvest press | Herbivore | 0.19231253454643138 |
| Predator harvest press | Predator | -1.0169852146498948 |

These values are not confirmatory results. The separate frozen execution and
its institutional evidence are recorded in
`reports/three-level-cascade-confirmation.md`.
