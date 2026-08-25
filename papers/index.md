# Paper Index

## [Institutions: Abstract Model Theory for Specification and Programming](Goguen_1992_InstitutionsAbstractModelTheory/notes.md)  (institutions, category-theory, semantics)
Goguen and Burstall define institutions using a signature category, sentence and model functors, and satisfaction invariant under change of notation. The paper develops theory structuring, cross-institution mappings, and detailed logical examples. It is the authority for the category, functor, and satisfaction laws required by this project's Institution boundary.

## [Conservation Laws in Biochemical Reaction Networks](Mahdi_2017_ConservationLawsBiochemicalReaction/notes.md)  (conservation, stoichiometry, dynamical-systems)
Mahdi and colleagues study linear and nonlinear conservation laws in mass-action reaction networks. Structural linear laws arise from the stoichiometric matrix's left nullspace, while additional laws can depend on kinetics or be nonlinear. The paper supplies the exact algebraic contract and scope limit for this project's linear conservation layer.

## [The Trophic-Dynamic Aspect of Ecology](Lindeman_1942_TrophicDynamicAspectEcology/notes.md)  (ecosystems, energy-flow, trophic-dynamics)
Lindeman treats ecosystems as energy-bearing trophic compartments connected by production, consumption, decomposition, and dissipation. The account distinguishes stocks, transfers, losses, rates, and efficiencies. It motivates explicit boundary I/O, internal-transfer cancellation, and classified compartment flows in the simulation core.

## [Ecopath with Ecosim: Methods, Capabilities and Limitations](Christensen_2004_EcopathEcosimMethodsCapabilities/notes.md)  (ecosystem-modeling, mass-balance, energy-balance)
Christensen and Walters describe static Ecopath balance, dynamic Ecosim, and spatial Ecospace. The paper covers accounting equations, uncertainty, forcing, fitting, structure, and limitations. It supports exact currency-specific accounting while separating consistency from equilibrium, calibration, mechanism choice, and predictive validity.

## [Testing Ecological Models: The Meaning of Validation](Rykiel_1996_TestingEcologicalModelsValidation/notes.md)  (model-validation, verification, ecological-modeling)
Rykiel separates verification, calibration, validation, credibility, and qualification for ecological simulations. Validation is relative to an intended purpose, explicit criteria, and a stated context. The paper supports strong implementation verification while preventing passing conservation tests from being presented as empirical ecosystem validation.

## [A structure-preserving numerical approach for simulating algae blooms in marine water bodies of western Patagonia](Almonacid_2020_Structure-preservingNumericalApproachSimulating/notes.md)  (ecosystem-modeling, mass-balance, dynamical-systems, conservation)
Almonacid and Medel present a two-layer, nitrogen-based NPZD model for short wind-driven algal blooms in western Patagonia. Their split MPRK/exact-flow integrator preserves positive states and computes an exact biomass ledger while a genetic algorithm calibrates four parameterizations against a 2015 winter bloom. The equations, forcing cases, and Figure 4 mass-balance evidence provide an independent conservation and nonnegative-state benchmark for this project's ecosystem simulation core.
