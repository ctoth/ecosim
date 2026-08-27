# Ecosystem-to-Economics Handoff

## Concept mapping

The completed ecosystem slice provides a concrete vocabulary for a later
economic client. This table is a design mapping, not an implementation or a
claim that ecological and economic dynamics are identical.

| Ecosystem concept | Candidate economic analogue |
|---|---|
| Physical stock | Account balance or inventory balance |
| Internal feeding or transfer | Transaction between accounts |
| Boundary input | Issuance, production, or import |
| Boundary output | Export, consumption, tax, waste, or leakage |
| Invariant total | Closed-currency or inventory identity |
| Nonnegative stock sentence | No-overdraft or solvency rule where applicable |
| Nondecreasing boundary ledger | Cumulative tax, loss, issuance, or leakage account |
| Immutable intervention schedule | Policy, demand, supply, or market shock |
| Named counterfactual response | Difference from a policy-free control |
| Exact evidence replay | Audited accounting scenario |

## Candidate reuse seam

The ecosystem experiment now depends on four distinct capabilities:

1. compile a domain topology into state transitions;
2. carry immutable initial states and forcing schedules;
3. compare named control and intervention trajectories;
4. audit selected exact traces using institutional sentences and typed
   verdicts.

An economics client can reuse the already-generic `institution` and
`conservation` libraries immediately. The topology declarations, schedule
axes, response metrics, and error semantics remain domain-owned until an
economic implementation shows that they are actually identical.

## Extraction decision

Do not extract another shared library yet. Ecosim is currently the only
concrete client of `TrophicIntervention`, `TrophicTrajectory`, and
`TrophicResponse`; the economic client is still a mapping rather than running
code. Similar vocabulary is insufficient evidence that both clients need the
same data types, law construction, verdict handling, and failure semantics.

Revisit extraction only after the economics project implements one complete
audited experiment. At that point compare the two concrete APIs. Extract the
smallest structurally identical part, if any, while leaving domain topology
and law selection with their clients.
