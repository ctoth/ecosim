"""Typed Python construction API for configurable trophic networks."""

from __future__ import annotations

from collections.abc import Mapping
from dataclasses import dataclass
from fractions import Fraction

import numpy as np
from numpy.typing import NDArray

from ecosim._core import (
    DenseTrophicNetworkStep,
    NativeDenseTrophicNetwork,
    NativeDenseTrophicNetworkPlan,
    NativeTrophicNetwork,
    NativeTrophicNetworkPlan,
    TrophicNetworkStep,
)


@dataclass(frozen=True)
class Producer:
    """One nutrient-limited producer declaration."""

    name: str
    max_growth: float
    nutrient_half_saturation: float
    mortality: float


@dataclass(frozen=True)
class Consumer:
    """One consumer declaration."""

    name: str
    mortality: float


@dataclass(frozen=True)
class Feeding:
    """One directed resource-to-consumer feeding relationship."""

    consumer: str
    resource: str
    max_rate: float
    resource_half_saturation: float
    assimilation_efficiency: float


@dataclass(frozen=True)
class TrophicNetworkSpec:
    """An immutable trophic structure and its process parameters."""

    producers: tuple[Producer, ...]
    consumers: tuple[Consumer, ...]
    feedings: tuple[Feeding, ...]
    decomposition: float


@dataclass(frozen=True)
class TrophicLaw:
    """One named graded sentence compiled for an exact trophic plan."""

    name: str
    axis_name: str | None
    grade: str


@dataclass(frozen=True)
class TrophicLawEvidence:
    """Python summary of one typed exact verdict retained by the Rust core."""

    law: TrophicLaw
    satisfied: bool


@dataclass(frozen=True)
class TrophicEvidence:
    """Institutional verdict summaries for one exact run."""

    laws: tuple[TrophicLawEvidence, ...]

    @property
    def satisfied(self) -> bool:
        return all(evidence.satisfied for evidence in self.laws)


@dataclass(frozen=True)
class ExactFlowEvidence:
    """One semantic proposal and its kernel-settled exact amount."""

    name: str
    proposed: Fraction
    settled: Fraction


@dataclass(frozen=True)
class ExactTransitionEvidence:
    """Exact process evidence for one committed trophic transition."""

    index: int
    time_before: Fraction
    time_after: Fraction
    flows: tuple[ExactFlowEvidence, ...]


@dataclass(frozen=True)
class ExactPairedSentenceEvidence:
    """One exact comparative verdict produced by the Rust paired-run theory."""

    satisfied: bool
    delta: Fraction


def _producer_tuples(spec: TrophicNetworkSpec) -> list[tuple[str, float, float, float]]:
    return [
        (
            producer.name,
            producer.max_growth,
            producer.nutrient_half_saturation,
            producer.mortality,
        )
        for producer in spec.producers
    ]


def _consumer_tuples(spec: TrophicNetworkSpec) -> list[tuple[str, float]]:
    return [(consumer.name, consumer.mortality) for consumer in spec.consumers]


def _feeding_tuples(
    spec: TrophicNetworkSpec,
) -> list[tuple[str, str, float, float, float]]:
    return [
        (
            feeding.consumer,
            feeding.resource,
            feeding.max_rate,
            feeding.resource_half_saturation,
            feeding.assimilation_efficiency,
        )
        for feeding in spec.feedings
    ]


class TrophicNetworkPlan:
    """An exact trophic structure compiled once for independent runs."""

    def __init__(self, spec: TrophicNetworkSpec) -> None:
        self._inner = NativeTrophicNetworkPlan(
            _producer_tuples(spec),
            _consumer_tuples(spec),
            _feeding_tuples(spec),
            spec.decomposition,
        )

    @property
    def stock_names(self) -> tuple[str, ...]:
        return tuple(self._inner.stock_names)

    @property
    def consumer_names(self) -> tuple[str, ...]:
        return tuple(self._inner.consumer_names)

    @property
    def evidence_axis_names(self) -> tuple[str, ...]:
        return tuple(self._inner.evidence_axis_names)

    @property
    def evidence_laws(self) -> tuple[TrophicLaw, ...]:
        return tuple(TrophicLaw(*law) for law in self._inner.evidence_laws)

    @property
    def law_suite_laws(self) -> tuple[TrophicLaw, ...]:
        return tuple(TrophicLaw(*law) for law in self._inner.law_suite_laws)

    def start(self, initial: Mapping[str, float]) -> TrophicNetwork:
        return TrophicNetwork.from_compiled(self._inner.start(dict(initial)))


class DenseTrophicNetworkPlan:
    """A dense trophic structure compiled once for runs and NumPy ensembles."""

    def __init__(self, spec: TrophicNetworkSpec) -> None:
        self._inner = NativeDenseTrophicNetworkPlan(
            _producer_tuples(spec),
            _consumer_tuples(spec),
            _feeding_tuples(spec),
            spec.decomposition,
        )

    @property
    def stock_names(self) -> tuple[str, ...]:
        return tuple(self._inner.stock_names)

    @property
    def consumer_names(self) -> tuple[str, ...]:
        return tuple(self._inner.consumer_names)

    def start(self, initial: Mapping[str, float]) -> DenseTrophicNetwork:
        return DenseTrophicNetwork.from_compiled(self._inner.start(dict(initial)))

    def simulate(
        self,
        initial_states: NDArray[np.float64],
        steps: int,
        elapsed: float,
        nutrient_inputs: NDArray[np.float64] | None = None,
        harvests: NDArray[np.float64] | None = None,
    ) -> NDArray[np.float64]:
        return self._inner.simulate(
            initial_states,
            steps,
            elapsed,
            nutrient_inputs,
            harvests,
        )


class TrophicNetwork:
    """Exact-arithmetic execution of a compiled trophic network."""

    def __init__(self, spec: TrophicNetworkSpec, initial: Mapping[str, float]) -> None:
        self._inner = NativeTrophicNetwork(
            _producer_tuples(spec),
            _consumer_tuples(spec),
            _feeding_tuples(spec),
            spec.decomposition,
            dict(initial),
        )

    @classmethod
    def from_compiled(cls, inner: NativeTrophicNetwork) -> TrophicNetwork:
        """Wraps a run started by a compiled plan."""

        instance = cls.__new__(cls)
        instance._inner = inner
        return instance

    @property
    def time(self) -> float:
        return self._inner.time

    @property
    def stock_names(self) -> tuple[str, ...]:
        return tuple(self._inner.stock_names)

    def stock(self, name: str) -> float:
        return self._inner.stock(name)

    @property
    def exact_stocks(self) -> Mapping[str, Fraction]:
        """Exact current stock values, without binary64 projection."""

        return {
            name: Fraction(numerator, denominator)
            for name, numerator, denominator in self._inner.exact_stocks()
        }

    @property
    def inputs(self) -> float:
        return self._inner.inputs

    @property
    def outputs(self) -> float:
        return self._inner.outputs

    @property
    def balance_residual(self) -> float:
        return self._inner.balance_residual

    @property
    def balanced(self) -> bool:
        return self._inner.balanced

    @property
    def trace_length(self) -> int:
        return self._inner.trace_length

    def evidence(self) -> TrophicEvidence:
        return TrophicEvidence(
            tuple(
                TrophicLawEvidence(TrophicLaw(name, axis_name, grade), satisfied)
                for name, axis_name, grade, satisfied in self._inner.evidence()
            )
        )

    def law_evidence(self) -> TrophicEvidence:
        """Complete transition, process, boundary, graded, and balance evidence."""

        return TrophicEvidence(
            tuple(
                TrophicLawEvidence(TrophicLaw(name, axis_name, grade), satisfied)
                for name, axis_name, grade, satisfied in self._inner.law_evidence()
            )
        )

    def paired_terminal_sentence(
        self,
        perturbed: TrophicNetwork,
        baseline_id: str,
        perturbed_id: str,
        sentence_name: str,
        axis: str,
        relation: str,
        threshold: Fraction,
    ) -> ExactPairedSentenceEvidence:
        """Checks one terminal comparison over two sealed exact run audits."""

        satisfied, numerator, denominator = self._inner.paired_terminal_sentence(
            perturbed._inner,
            baseline_id,
            perturbed_id,
            sentence_name,
            axis,
            relation,
            threshold.numerator,
            threshold.denominator,
        )
        return ExactPairedSentenceEvidence(
            satisfied,
            Fraction(numerator, denominator),
        )

    @property
    def transition_count(self) -> int:
        return self._inner.transition_count

    @property
    def transitions(self) -> tuple[ExactTransitionEvidence, ...]:
        """Committed exact proposed/settled flow records."""

        return tuple(
            ExactTransitionEvidence(
                index,
                Fraction(*time_before),
                Fraction(*time_after),
                tuple(
                    ExactFlowEvidence(
                        name,
                        Fraction(proposed_numerator, proposed_denominator),
                        Fraction(settled_numerator, settled_denominator),
                    )
                    for (
                        name,
                        proposed_numerator,
                        proposed_denominator,
                        settled_numerator,
                        settled_denominator,
                    ) in flows
                ),
            )
            for index, time_before, time_after, flows in self._inner.transitions()
        )

    def step(
        self,
        elapsed: float,
        nutrient_input: float = 0.0,
        harvests: Mapping[str, float] | None = None,
    ) -> TrophicNetworkStep:
        return self._inner.step(
            elapsed,
            nutrient_input,
            None if harvests is None else dict(harvests),
        )


class DenseTrophicNetwork:
    """Binary64 execution of the same compiled trophic-network semantics."""

    def __init__(self, spec: TrophicNetworkSpec, initial: Mapping[str, float]) -> None:
        self._inner = NativeDenseTrophicNetwork(
            _producer_tuples(spec),
            _consumer_tuples(spec),
            _feeding_tuples(spec),
            spec.decomposition,
            dict(initial),
        )

    @classmethod
    def from_compiled(cls, inner: NativeDenseTrophicNetwork) -> DenseTrophicNetwork:
        """Wraps a run started by a compiled plan."""

        instance = cls.__new__(cls)
        instance._inner = inner
        return instance

    @property
    def time(self) -> float:
        return self._inner.time

    @property
    def stock_names(self) -> tuple[str, ...]:
        return tuple(self._inner.stock_names)

    def stock(self, name: str) -> float:
        return self._inner.stock(name)

    @property
    def inputs(self) -> float:
        return self._inner.inputs

    @property
    def outputs(self) -> float:
        return self._inner.outputs

    @property
    def balance_residual(self) -> float:
        return self._inner.balance_residual

    @property
    def balanced(self) -> bool:
        return self._inner.balanced

    def step(
        self,
        elapsed: float,
        nutrient_input: float = 0.0,
        harvests: Mapping[str, float] | None = None,
    ) -> DenseTrophicNetworkStep:
        return self._inner.step(
            elapsed,
            nutrient_input,
            None if harvests is None else dict(harvests),
        )
