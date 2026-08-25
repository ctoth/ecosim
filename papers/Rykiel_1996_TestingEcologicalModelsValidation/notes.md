---
title: "Testing Ecological Models: The Meaning of Validation"
authors: "Edward J. Rykiel, Jr."
year: 1996
venue: "Ecological Modelling"
doi_url: "https://doi.org/10.1016/0304-3800(95)00152-2"
---

# Reading notes — Rykiel (1996)

## Complete paper (PDF pp. 1–16; journal pp. 229–244)

- Validation is pragmatic acceptance for an intended use against specified performance requirements, not certification that a model or scientific theory is true. Purpose, criteria, and context must be stated before a validation claim has content. [PDF pp. 1–2; journal pp. 229–230]
- The paper distinguishes verification of the modeling formalism and implementation from validation of empirical performance. A verified program may still be a poor representation of the real system. [PDF pp. 4–6; journal pp. 232–234]
- Calibration adjusts parameters to improve agreement with a data set; it is neither verification nor independent validation. [PDF pp. 4–5; journal pp. 232–233]
- Operational, conceptual, and data validity are distinct. Good output correspondence does not guarantee correct mechanisms, and justifiable mechanisms do not guarantee accurate predictions. [PDF pp. 6–7; journal pp. 234–235]
- Useful verification/validation procedures include internal consistency, event validity, extreme-condition tests, traces, sensitivity analysis, multistage testing, prediction against independent data, and statistical comparison. The applicable set depends on purpose and available knowledge/data. [PDF pp. 7–9; journal pp. 235–237]
- Formal validity is not material soundness: correct conclusions from encoded premises do not establish that the premises describe reality. A simulation is also an analogy that shares selected properties with its target and necessarily fails outside some scope. [PDF pp. 9–10; journal pp. 237–238]
- A balanced budget failing against current observations can indicate bad data, omitted flows, a wrong system boundary, or a wrong model; it does not uniquely falsify ecosystem closure or conservation. [PDF p. 11; journal p. 239]
- Qualification seeks the domain where a model is applicable; invalidation identifies failure against an essential criterion. A failed test can lead to recalibration, structural revision, narrower scope, or rejection. [PDF pp. 11–12; journal pp. 239–240]
- Scale changes acceptable mechanisms, error tolerances, and context. Passing tests at one scale or context cannot be generalized silently. [PDF p. 12; journal p. 240]
- Modelers should state the technical sense of verification/validation, context, acceptability, and performance indices. Validation is only one part of evaluation and is unnecessary for some exploratory or theory-building models. [PDF pp. 13–14; journal pp. 241–242]
- The final pages summarize the purpose–criteria–context requirement and list references. [PDF pp. 14–16; journal pp. 242–244]

## Implementation obligations

1. Call property-based and example tests verification of the formalized contracts, not validation of an ecosystem.
2. Exercise internal consistency, event semantics, traces, and extreme inputs, including arbitrary sequences and rejected operations.
3. Require failed operations to be atomic so an error cannot leave an unverified partial state.
4. State the modeled quantity, system boundary, assumptions, and applicability when presenting an ecosystem model built on the kernel.
5. Reserve validation claims for a separately specified purpose, performance criterion, context, and preferably independent empirical data.

## Collection Cross-References

### Already in Collection

- (none found)

### New Leads (Not Yet in Collection)

- Oreskes, Shrader-Frechette, and Belitz (1994), "Verification, Validation, and Confirmation of Numerical Models in the Earth Sciences" - a stricter account already retrieved but not yet processed.
- Sargent (1984), "A Tutorial on Verification and Validation of Simulation Models" - source for the operational, conceptual, and data validation cycle.

### Supersedes or Recontextualizes

- (none)

### Conceptual Links (not citation-based)

- [Ecopath with Ecosim: Methods, Capabilities and Limitations](../Christensen_2004_EcopathEcosimMethodsCapabilities/notes.md) - concrete uncertainty and mechanism failures to which Rykiel's distinctions apply.
- [A structure-preserving numerical approach for simulating algae blooms in marine water bodies of western Patagonia](../Almonacid_2020_Structure-preservingNumericalApproachSimulating/notes.md) - illustrates Rykiel's distinction directly: conservation and positivity tests verify implementation contracts, while fitting the same Puyuhuapi observations used for calibration does not independently validate the ecological model.

### Cited By (in Collection)

- (none found)
