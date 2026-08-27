# Ecosystem-to-Economics Handoff

## Concept mapping

The completed ecosystem slice provides a concrete vocabulary for a later
economic client. This table is a design mapping, not an implementation or a
claim that ecological and economic dynamics are identical.

| Implemented ecosystem construct | Candidate economic client |
|---|---|
| `AxisId` plus `KindId` stock axis | Account, inventory, currency, good, or claim |
| Exact settled internal-flow vector | Balanced transaction postings |
| Typed boundary port and matrix direction | Issuance, retirement, import, export, loss, or leakage |
| `TransitionEquation` | Stock-flow consistency equation |
| `LinearFlowConstraint` | Double-entry or transaction-bundle constraint |
| `BoundaryCorrespondence` | Cumulative issuance, tax, or loss account |
| `CheckedNullspace` and `OpenBalance` | Incidence-derived accounting identity |
| Separate proposed and settled flow | Order versus executed or rationed trade |
| `ExactPairedModel` and comparison sentence | Audited policy counterfactual |

## Candidate reuse seam

The ecosystem experiment now depends on four distinct capabilities:

1. compile a domain topology into state transitions;
2. carry immutable initial states and forcing schedules;
3. compare named control and intervention trajectories;
4. audit selected exact traces using institutional sentences and typed
   verdicts.

An economics client can now reuse `conservation-stock-flow` and the stock-flow
adapter in `institution-conservation` directly. The implemented neutral
cross-domain test shows that one source sentence can be renamed into both
ecological and economic vocabulary without changing satisfaction. It does not
yet supply an economic transaction compiler, behavioral model, or audited
economic run.

Kinetic expressions, trophic process compilation, intervention arrays, and
cascade response selection remain ecosystem-owned. An economics client must
first demonstrate identical law construction, translation, evidence, and
error semantics before any of those are extracted.

## Extraction decision

The domain-neutral stock-flow carrier and Institution are justified and have
already been extracted: their syntax and semantics are independently defined,
and the cross-domain fixture exercises the same sentence. Do not extract an
additional experiment, kinetic, or schedule library yet. Ecosim remains the
only concrete client of `TrophicIntervention`, `TrophicTrajectory`, kinetic
sentences, allocation groups, and cascade response selection. Similar
vocabulary is insufficient evidence of identical failure semantics.

Revisit extraction only after the economics project implements one complete
audited experiment. At that point compare the two concrete APIs. Extract the
smallest structurally identical part, if any, while leaving domain topology
and law selection with their clients.
