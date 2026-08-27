# Synthetic Multi-Kind Balance Boundaries

The fixture is mathematical and synthetic, not empirically calibrated.

| Kind | Internal stocks | Input boundary | Output boundary |
|---|---|---|---|
| Carbon | available C, biomass C | C input | C export |
| Nitrogen | available N, biomass N | N input | N export |
| Phosphorus | available P, biomass P | P input | P export |
| Stored energy | stored energy | energy input | heat, work, energy export |

Every scalar stock and flow has exactly one kind. C, N, and P use same-kind
available-to-biomass transfers. Stored energy has no invented conversion into
matter. Boundary magnitudes are nonnegative and boundary-matrix signs encode
direction. Input and output ledgers are separate because an unsigned cumulative
ledger cannot mix opposite boundary roles.

Four checked nullspace certificates derive separate open balances. The energy
sentence is

```text
delta(E_stored) = E_input - E_heat - E_work - E_export
```

and is not an invariant of biomass. A negative fixture removes the settled
heat amount while retaining the observed state change; both energy open balance
and energy-output correspondence then fail visibly. A cross-kind carbon-flow
effect on a nitrogen stock fails topology construction. Independently, the
carrier-checked sentence constructor rejects a carbon law that names the valid
nitrogen-incorporation flow, proving kind safety at law construction rather
than only at topology construction or later evaluation.
