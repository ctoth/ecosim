# Ecosystem Process and Boundary Laws

Each compiled exact trophic plan now has stable semantic symbols for nutrient
input, producer growth, assimilated feeding, feeding waste, mortality,
decomposition, and per-consumer harvest. Every accepted step retains pre/post
stocks, requested proposals, kernel-settled amounts, elapsed time, and
cumulative ledgers. State snapshots remain a compatibility projection.

The compiled law suite contains, in canonical family/symbol order:

- the exact state-transition equation;
- one feeding-partition constraint per feeding edge;
- nutrient-input ledger correspondence;
- one harvest-ledger correspondence per consumer;
- existing invariant, nonnegative-stock, and nondecreasing-ledger laws;
- a sealed-certificate-derived open material balance.

Typed verdicts retain sentence family, relevant symbols, and the underlying
exact witness or first violation. Tests cover valid generated runs, balanced
misrouting, a wrong feeding partition, a dishonest input ledger, two harvest
amounts attributed to the wrong per-consumer ledgers, source-limited harvest,
trace continuity, rejected-step atomicity, and declaration-order stability.
Dense runs expose no exact institutional witnesses.

Python exposes both the compatibility graded evidence and the complete suite.
Exact transition and terminal values cross the ABI as integer
numerator/denominator pairs and become `fractions.Fraction`; no binary64
round-trip is used for exact evidence.
