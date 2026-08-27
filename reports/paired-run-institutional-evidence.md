# Paired-Run Institutional Evidence

`ExactRunRecord::from_trophic_network` is the only public run-record
constructor. It recomputes the complete law suite, rejects zero-transition or
unsatisfied runs, and seals the accepted transitions, their requested and
settled values, the compiled exact plan identity, the forcing schedule, and
terminal stocks together. Callers cannot supply topology or parameter
fingerprints or substitute a terminal map.

`ExactPairedModel` accepts two sealed records only when their compiled topology
and exact parameters, horizon, observation-time grid, and terminal axes agree.
The exact forcing schedule is retained separately for each arm: requiring its
values to agree would erase the intervention being compared. Run identities
must remain distinct.

Named `PairedComparisonSentence` values compare the exact
perturbed-minus-baseline terminal delta with a rational threshold. A typed
`PairedResponseMetric::TerminalStock` makes the observation rule explicit in
the sentence, witness, and violation. Witnesses retain both run identities,
metric, delta, relation, and threshold. Suites stop
at and retain the first failed named comparison. A total bijective axis
renaming applied to both models and sentences preserves comparative truth.

The frozen cascade's short exact audits retain their native sealed runs. Each
preregistered sign observation now crosses the Python/Rust boundary into a
`PairedComparisonSentence`; the returned exact rational delta and typed
satisfaction verdict populate the public observation. The configured binary64
epsilon is interpreted as its exact binary rational value. The longer dense
comparisons remain numerical confirmation only. Every cascade claim remains
conditional on the frozen topology, parameters, initial states, two audited
forcing schedules, and terminal observation rule; it is not asserted as a
universal ecological law.
