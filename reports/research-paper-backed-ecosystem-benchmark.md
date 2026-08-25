# Research: paper-backed ecosystem benchmark

## Summary

The four supplied criteria select a two-paper benchmark suite, but do not authorize an ordering between its complementary members. Mata Almonacid and Medel's four-compartment, nitrogen-currency NPZD model supplies published pulse/bloom trajectories and boundary-aware mass-balance figures. Heinle and Slawig's NPZD variants supply published equilibria and stability regimes. Both publish equations and parameters and map directly to nutrient–producer–consumer–detritus. Choosing which to implement first requires an additional user-approved criterion; complexity and alignment with the current runtime are reported below but were not used to rank them.

## Selection Criteria Used

Only the criteria supplied for this research were used to compare and recommend candidates:

1. Published equations and parameters.
2. Defensible mapping to nutrient–producer–consumer–detritus.
3. Stated equilibria or qualitative regimes.
4. A trajectory or figure we can reproduce.

Complexity and implementation availability are reported below because the research skill requires them; they were not additional selection criteria.

| Candidate | Equations and parameters | N–P–Z–D mapping | Equilibria or regimes | Reproducible trajectory/figure | Result |
| --- | --- | --- | --- | --- | --- |
| Mata Almonacid & Medel (2022) | Full ODE system, admissible ranges, calibrated parameter tables, initial conditions and forcing definitions | Exact: nutrient, phytoplankton, zooplankton, detritus, all in mmol N m⁻³ | Positive trajectories; closed internal mass balance; pulse-driven blooms; detrital-sinking loss regime | Figure 4 biomass balance under nutrient pulses/sinking; Figure 10 three-pulse NPZD response | **Selected for trajectory and balance benchmarks** |
| Heinle & Slawig (2013) | Three explicit NPZD closure variants with parameter/initial-value conditions | Exact four compartments in a common nitrogen currency | Three equilibria per model with stability analysis and characteristic behaviors | Model-response examples can be regenerated from the stated parameter/initial-value constellations | **Selected for equilibrium and stability benchmarks** |
| Powell et al. (2006) | Four biological equations and complete Table 1 parameter set | Exact nitrogen-based NPZD | Initial non-equilibrium oscillation and bloom decay by about 120 days | Figures 2–5 show transients, but inside a coupled three-dimensional physical model | Credible, but the published figure target includes spatial physics |
| Fasham, Ducklow & McKelvie (1990) | Published compartment equations, parameterization and Station S forcing | Requires aggregation: seven pools include nitrate, ammonium, DON and bacteria in addition to P, Z and D | Seasonal spring bloom, summer nutrient depletion, regenerated-production regime | Published annual Station S cycles | Credible, but not a four-stock benchmark without model-changing aggregation |

## Approaches Found

### Structure-preserving, forced NPZD box model

**Source:** [Mata Almonacid & Medel (2022), published article](https://doi.org/10.1016/j.ecolmodel.2021.109871); [open author manuscript](https://arxiv.org/abs/2007.11815)

**Description:** A well-mixed euphotic-layer model whose autonomous core transfers one nitrogen currency among nutrient, phytoplankton, zooplankton and detritus. External nutrient input is a time-dependent Gaussian pulse and detrital sinking is the boundary loss. The paper splits those three pieces and uses a modified Patankar–Runge–Kutta step for the internal production–destruction system.

**Pros:** The compartment mapping is exact; internal transfer conservation, nonnegative trajectories, boundary inputs and boundary losses are all explicit; published numerical studies expose both accounting behavior and an ecological bloom trajectory.

**Cons:** The calibrated field case adds daylight forcing, pulse construction, depth averaging and a non-unique genetic-algorithm fit. It is not a chemostat equilibrium study.

**Complexity:** Medium for Figure 4; high for the full Figure 10 field-calibrated case.

### Equilibrium and closure-regime NPZD models

**Source:** [Heinle & Slawig (2013), published article](https://doi.org/10.1016/j.ecolmodel.2013.01.012)

**Description:** Three four-stock NPZD ODE models based on the Oschlies–Garçon framework, differing in linear or quadratic phytoplankton and zooplankton loss closures. The paper derives three equilibria for every variant and analyzes stability, parameter restrictions and initial-value-dependent behavior.

**Pros:** It supplies strong analytical oracles for extinction, producer-only and coexistence states, and directly tests whether changing closure laws changes qualitative dynamics.

**Cons:** It is a family of three models rather than one canonical empirical trajectory. Its main contribution is internal dynamics, not reproduction of an observed time series.

**Complexity:** Medium.

### Four-component NPZD coupled to ocean circulation

**Source:** [Powell et al. (2006), full published article](https://doi.org/10.1029/2004JC002506)

**Description:** A nitrogen-based N–P–Z–D reaction model embedded in a three-dimensional California Current circulation model. Published biological terms include light- and nutrient-limited uptake, Ivlev grazing, mortality, remineralization and detrital sinking; Table 1 supplies the parameter values.

**Pros:** Exact four-compartment mapping, a complete parameter table, and clear transient bloom behavior from stated initial conditions.

**Cons:** The paper's figures combine biology with advection, vertical mixing, irradiance by depth and detrital sinking. Reproducing them would test a spatial ocean model, not just the current ecosystem core.

**Complexity:** High.

### Seven-compartment seasonal mixed-layer model

**Source:** [Fasham, Ducklow & McKelvie (1990), published article](https://doi.org/10.1357/002224090784984678)

**Description:** A nitrogen-based seasonal mixed-layer model for Station S near Bermuda, with phytoplankton, zooplankton, bacteria, nitrate, ammonium, dissolved organic nitrogen and detritus.

**Pros:** Seminal real-ecosystem model with a pronounced spring bloom and published seasonal cycles.

**Cons:** Collapsing nitrate, ammonium, DON and bacteria into a single nutrient/detrital structure changes the model. Its mixed-layer and irradiance forcings also make a faithful reproduction substantially larger than the current four-stock slice.

**Complexity:** High.

## Key Papers

- [Mata Almonacid & Medel (2022)](https://doi.org/10.1016/j.ecolmodel.2021.109871) — Defines and calibrates the selected forced NPZD trajectory model and its structure-preserving integration.
- [Mata Almonacid & Medel (2020 author manuscript)](https://arxiv.org/abs/2007.11815) — Open full manuscript containing equations, parameter tables, forcing definitions and Figures 4 and 10.
- [Heinle & Slawig (2013)](https://doi.org/10.1016/j.ecolmodel.2013.01.012) — Derives equilibria and stability conditions for three common NPZD closure variants.
- [Powell et al. (2006)](https://doi.org/10.1029/2004JC002506) — Gives a complete four-component NPZD parameterization and coupled spatial trajectories.
- [Fasham, Ducklow & McKelvie (1990)](https://doi.org/10.1357/002224090784984678) — Establishes the classic nitrogen-based mixed-layer seasonal plankton model.

## Existing Implementations

- **[OceanBioME.jl NPZD](https://oceanbiome.github.io/OceanBioME.jl/stable/model_components/biogeochemical/NPZ/):** Maintained Julia implementation of a low-complexity NPZD model. Its documentation publishes the four equations, parameter names/defaults and the nitrogen-conservation identity. It implements the Kuhn et al. formulation, not either selected paper verbatim, so it is useful as an independent interface and conservation reference rather than a trajectory reference.
- **Published Almonacid–Medel algorithm:** The paper specifies a three-stage splitting/composition integrator using modified Patankar–Runge–Kutta for autonomous NPZD, an analytic nutrient-pulse stage and an analytic sinking stage. No official source repository was identified in the primary sources reviewed; implement from the paper and compare against its figures rather than importing an unverified transcription.

## Complexity vs Quality Tradeoffs

The smallest reported trajectory target is Almonacid–Medel Figure 4. It needs the four internal NPZD processes, Gaussian nutrient input, detrital loss and the paper's stated parameter set, but not the field-data interpolation or genetic algorithm. In return it tests nonnegativity and total nitrogen equal to initial nitrogen plus integrated input minus integrated sinking. This complexity observation does not rank it ahead of the independently useful equilibrium target.

The higher-quality ecological reproduction is Figure 10. It adds three explicit nutrient pulses, light availability and the calibrated parameter vector, and should reproduce the ordering and shape of nutrient, phytoplankton, zooplankton and detritus responses. The full empirical calibration is a separate, larger step because reproducing the optimizer is not required to reproduce the published trajectory with the published fit.

Heinle–Slawig provides a different kind of quality: analytical equilibrium and stability checks. Together, the two selected papers can check both finite-time behavior and asymptotic regimes; the supplied criteria do not determine their implementation order. Powell and Fasham add physical and biological realism, but their figures entangle the four-stock reaction system with spatial or seasonal machinery.

## Recommendations

Treat the two qualified papers as a complementary benchmark suite and keep both as named paper specifications rather than embedding either paper's equations in the generic stock-flow engine. The supplied criteria do not determine implementation order.

- **Forced-trajectory member — Mata Almonacid–Medel:** Encode the autonomous NPZD production/destruction terms in the common nitrogen currency; represent Gaussian nutrient pulses as boundary inputs and detrital sinking as boundary outputs; reproduce Figure 4's pulse and sinking scenarios; and reproduce Figure 10's three-pulse compartment trajectories.
- **Equilibrium member — Heinle–Slawig:** Implement one stated NPZD closure variant and verify its three equilibrium classes and corresponding stability conditions.

These are parallel benchmark obligations. No implementation sequence is selected here.

The dense Rust backend is the intended trajectory runner, subject to performance measurement on these benchmarks. The exact rational backend remains a short-horizon arithmetic reference where the same process amounts can be represented exactly; because both compiled backends share settlement code, an independent implementation must remain in the semantic conformance suite. Exact rationals should not be expected to reproduce transcendental light or Gaussian forcing without an explicitly defined approximation.

## Estimated Implementation Effort

- **Minimal approach:** Implement one member only: either Figure 4's forced trajectory and balance scenarios, or one Heinle–Slawig closure's stated equilibria and stability conditions. These give different evidence and the supplied criteria do not choose between them.
- **Full approach:** Implement both members, extend the trajectory member through Figure 10's daylight and three-pulse forcing, and add figure-data comparison tooling. Re-running the genetic-algorithm calibration and recreating field-data preprocessing remain outside the trajectory reproduction itself.

## Open Questions

- [ ] Should Figure 10 acceptance use author data if it can be obtained, or values digitized from the published figure with an explicit tolerance?
- [ ] Which of the paper's calibrated parameter rows should be canonical when the optimization admits multiple local optima?
- [ ] Should the paper's modified Patankar–Runge–Kutta method be reproduced as a separate integrator benchmark, or should only its invariants and trajectory be compared with the stock-flow limiter?
- [ ] How should Gaussian and daylight forcing be approximated in exact-backend differential tests so the approximation, rather than floating-point roundoff, is the declared source of error?

## References

- Mata Almonacid, P., & Medel, C. (2022). *A structure-preserving model for the dynamics of estuarine ecosystems and its application in western Patagonia fjords*. Ecological Modelling, 466, 109871. https://doi.org/10.1016/j.ecolmodel.2021.109871
- Mata Almonacid, P., & Medel, C. (2020). *A structure-preserving numerical approach for simulating algae blooms in marine water bodies of western Patagonia*. https://arxiv.org/abs/2007.11815
- Heinle, A., & Slawig, T. (2013). *Internal dynamics of NPZD type ecosystem models*. Ecological Modelling, 254, 33–42. https://doi.org/10.1016/j.ecolmodel.2013.01.012
- Powell, T. M., Lewis, C. V. W., Curchitser, E. N., Haidvogel, D. B., Hermann, A. J., & Dobbins, E. L. (2006). *Results from a three-dimensional, nested biological-physical model of the California Current System and comparisons with statistics from satellite imagery*. Journal of Geophysical Research: Oceans, 111, C07018. https://doi.org/10.1029/2004JC002506
- Fasham, M. J. R., Ducklow, H. W., & McKelvie, S. M. (1990). *A nitrogen-based model of plankton dynamics in the oceanic mixed layer*. Journal of Marine Research, 48(3), 591–639. https://doi.org/10.1357/002224090784984678
- OceanBioME contributors. *Nutrient Phytoplankton Zooplankton Detritus (NPZD) model*. https://oceanbiome.github.io/OceanBioME.jl/stable/model_components/biogeochemical/NPZ/
