# Kinetic and Allocation Semantics

Kinetic sentences remain owned by ecosim. They recompute proposals from the
pre-transition exact state and declared exact elapsed time:

- nutrient-limited producer growth;
- Holling-type total feeding before assimilation/waste partition;
- biomass-proportional producer and consumer mortality;
- detritus-proportional decomposition.

Parameters submitted through the binary64 Python API are interpreted as their
exact binary rational values. Negative rates and nonpositive half-saturation
parameters are structural errors. A mismatch reports the first transition,
semantic flow, recorded proposal, and recomputed expectation. Zero actor,
resource, maximum, or elapsed time produces a zero saturating proposal.

Allocation sentences name the complete set of withdrawals sharing a source.
They check `0 <= settled <= proposed`, zero-source behavior, and equality of
the exact `settled/proposed` scale across positive competing proposals. This
describes the existing conservation kernel's simultaneous proportional
settlement; it does not introduce another allocator.
