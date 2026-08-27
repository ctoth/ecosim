"""Typed Python construction API for configurable trophic networks."""

from __future__ import annotations

from collections.abc import Mapping
from dataclasses import dataclass

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
