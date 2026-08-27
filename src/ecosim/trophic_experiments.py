"""Topology-agnostic counterfactual experiments for trophic-network plans."""

from __future__ import annotations

from collections.abc import Mapping
from dataclasses import dataclass, field
from fractions import Fraction

import numpy as np
from numpy.typing import NDArray

from ecosim.network import (
    DenseTrophicNetworkPlan,
    ExactPairedSentenceEvidence,
    ExactTransitionEvidence,
    TrophicEvidence,
    TrophicNetwork,
    TrophicNetworkPlan,
)


def _dimension(name: str, value: object, *, positive: bool = False) -> int:
    if isinstance(value, bool) or not isinstance(value, int):
        raise ValueError(f"{name} must be an integer")
    lower = 1 if positive else 0
    if value < lower:
        qualifier = "positive" if positive else "nonnegative"
        raise ValueError(f"{name} must be {qualifier}")
    return value


def _nonnegative(name: str, value: float) -> float:
    result = float(value)
    if not np.isfinite(result) or result < 0.0:
        raise ValueError(f"{name} must be finite and nonnegative")
    return result


def _names(name: str, values: tuple[str, ...]) -> tuple[str, ...]:
    result = tuple(values)
    if any(not value.strip() for value in result):
        raise ValueError(f"{name} must contain nonblank names")
    if len(set(result)) != len(result):
        raise ValueError(f"{name} must not contain duplicates")
    return result


def _frozen_array(
    name: str, values: NDArray[np.float64], *, dimensions: int
) -> NDArray[np.float64]:
    result = np.array(values, dtype=np.float64, copy=True)
    if result.ndim != dimensions:
        raise ValueError(f"{name} must be {dimensions}-dimensional")
    if not np.all(np.isfinite(result)) or np.any(result < 0.0):
        raise ValueError(f"{name} values must be finite and nonnegative")
    result.flags.writeable = False
    return result


@dataclass(frozen=True)
class TrophicIntervention:
    """Immutable boundary forcing with explicit consumer-axis metadata."""

    consumer_names: tuple[str, ...]
    nutrient_inputs: NDArray[np.float64]
    harvests: NDArray[np.float64]

    def __post_init__(self) -> None:
        consumer_names = _names("consumer_names", self.consumer_names)
        nutrient_inputs = _frozen_array(
            "nutrient_inputs", self.nutrient_inputs, dimensions=2
        )
        harvests = _frozen_array("harvests", self.harvests, dimensions=3)
        if harvests.shape != (
            nutrient_inputs.shape[0],
            nutrient_inputs.shape[1],
            len(consumer_names),
        ):
            raise ValueError(
                "harvests must have shape (batch, steps, len(consumer_names))"
            )
        object.__setattr__(self, "consumer_names", consumer_names)
        object.__setattr__(self, "nutrient_inputs", nutrient_inputs)
        object.__setattr__(self, "harvests", harvests)

    @property
    def batch(self) -> int:
        return self.nutrient_inputs.shape[0]

    @property
    def steps(self) -> int:
        return self.nutrient_inputs.shape[1]

    @classmethod
    def unforced(
        cls, batch: int, steps: int, consumer_names: tuple[str, ...]
    ) -> TrophicIntervention:
        batch = _dimension("batch", batch)
        steps = _dimension("steps", steps, positive=True)
        consumer_names = _names("consumer_names", consumer_names)
        return cls(
            consumer_names,
            np.zeros((batch, steps), dtype=np.float64),
            np.zeros((batch, steps, len(consumer_names)), dtype=np.float64),
        )

    @classmethod
    def nutrient_pulse(
        cls,
        batch: int,
        steps: int,
        consumer_names: tuple[str, ...],
        *,
        at: int,
        amount: float,
    ) -> TrophicIntervention:
        result = cls.unforced(batch, steps, consumer_names)
        at = _dimension("at", at)
        if at >= steps:
            raise ValueError("at must identify a transition before steps")
        amount = _nonnegative("amount", amount)
        nutrient_inputs = np.array(result.nutrient_inputs, copy=True)
        nutrient_inputs[:, at] = amount
        return cls(result.consumer_names, nutrient_inputs, result.harvests)

    @classmethod
    def consumer_harvest_press(
        cls,
        batch: int,
        steps: int,
        consumer_names: tuple[str, ...],
        *,
        consumer: str,
        start: int,
        stop: int,
        amount_per_step: float,
    ) -> TrophicIntervention:
        result = cls.unforced(batch, steps, consumer_names)
        start = _dimension("start", start)
        stop = _dimension("stop", stop)
        if start >= stop or stop > steps:
            raise ValueError("press interval must satisfy 0 <= start < stop <= steps")
        amount_per_step = _nonnegative("amount_per_step", amount_per_step)
        try:
            consumer_index = result.consumer_names.index(consumer)
        except ValueError as error:
            raise ValueError(f"unknown intervention consumer: {consumer}") from error
        harvests = np.array(result.harvests, copy=True)
        harvests[:, start:stop, consumer_index] = amount_per_step
        return cls(result.consumer_names, result.nutrient_inputs, harvests)


@dataclass(frozen=True)
class TrophicTrajectory:
    """Immutable trajectories with explicit stock and consumer axes."""

    values: NDArray[np.float64]
    stock_names: tuple[str, ...]
    consumer_names: tuple[str, ...]

    def __post_init__(self) -> None:
        values = _frozen_array("values", self.values, dimensions=3)
        stock_names = _names("stock_names", self.stock_names)
        consumer_names = _names("consumer_names", self.consumer_names)
        if values.shape[2] != len(stock_names):
            raise ValueError("values stock axis must match stock_names")
        if values.shape[1] == 0:
            raise ValueError("values must contain an initial time point")
        object.__setattr__(self, "values", values)
        object.__setattr__(self, "stock_names", stock_names)
        object.__setattr__(self, "consumer_names", consumer_names)

    def stock(self, name: str) -> NDArray[np.float64]:
        try:
            index = self.stock_names.index(name)
        except ValueError as error:
            raise ValueError(f"unknown trophic stock: {name}") from error
        return self.values[:, :, index]


@dataclass(frozen=True)
class TrophicResponse:
    """Paired control/intervention trajectories and their signed response."""

    baseline: TrophicTrajectory
    perturbed: TrophicTrajectory
    delta: NDArray[np.float64] = field(init=False)

    def __post_init__(self) -> None:
        if self.baseline.stock_names != self.perturbed.stock_names:
            raise ValueError("baseline and perturbed stock axes must match")
        if self.baseline.consumer_names != self.perturbed.consumer_names:
            raise ValueError("baseline and perturbed consumer axes must match")
        if self.baseline.values.shape != self.perturbed.values.shape:
            raise ValueError("baseline and perturbed trajectories must have the same shape")
        delta = self.perturbed.values - self.baseline.values
        delta.flags.writeable = False
        object.__setattr__(self, "delta", delta)

    def stock_delta(self, stock: str) -> NDArray[np.float64]:
        try:
            index = self.baseline.stock_names.index(stock)
        except ValueError as error:
            raise ValueError(f"unknown trophic stock: {stock}") from error
        return self.delta[:, :, index]

    def terminal_delta(self, stock: str) -> NDArray[np.float64]:
        return self.stock_delta(stock)[:, -1]

    def peak_delta(self, stock: str) -> NDArray[np.float64]:
        return np.max(self.stock_delta(stock), axis=1)


@dataclass(frozen=True)
class ExactTrophicAudit:
    """One exact replay with explicit institutional evidence."""

    trajectory: TrophicTrajectory
    evidence: TrophicEvidence
    cumulative_input: float
    cumulative_output: float
    exact_terminal: Mapping[str, Fraction]
    transitions: tuple[ExactTransitionEvidence, ...]
    _run: TrophicNetwork = field(repr=False, compare=False)

    def paired_terminal_sentence(
        self,
        perturbed: ExactTrophicAudit,
        name: str,
        axis: str,
        relation: str,
        threshold: Fraction,
    ) -> ExactPairedSentenceEvidence:
        """Evaluates one paired sentence with this audit as the baseline arm."""

        return self._run.paired_terminal_sentence(
            perturbed._run,
            "baseline",
            "perturbed",
            name,
            axis,
            relation,
            threshold,
        )


@dataclass(frozen=True)
class ExactTrophicComparison:
    """Paired exact replays whose trajectories retain separate evidence."""

    baseline: ExactTrophicAudit
    perturbed: ExactTrophicAudit
    response: TrophicResponse = field(init=False)

    def __post_init__(self) -> None:
        object.__setattr__(
            self,
            "response",
            TrophicResponse(self.baseline.trajectory, self.perturbed.trajectory),
        )

    def evaluate_terminal_sentence(
        self,
        name: str,
        axis: str,
        relation: str,
        threshold: Fraction,
    ) -> ExactPairedSentenceEvidence:
        """Evaluates a core paired-model sentence over both sealed run audits."""

        return self.baseline.paired_terminal_sentence(
            self.perturbed,
            name,
            axis,
            relation,
            threshold,
        )


def simulate_trophic_trajectory(
    plan: DenseTrophicNetworkPlan,
    initial_states: NDArray[np.float64],
    elapsed: float,
    intervention: TrophicIntervention,
) -> TrophicTrajectory:
    """Runs one immutable initial-state/forcing scenario through a dense plan."""

    if intervention.consumer_names != plan.consumer_names:
        raise ValueError("intervention consumer_names must match the compiled plan")
    values = plan.simulate(
        initial_states,
        intervention.steps,
        elapsed,
        intervention.nutrient_inputs,
        intervention.harvests,
    )
    return TrophicTrajectory(values, plan.stock_names, plan.consumer_names)


def simulate_trophic_response(
    plan: DenseTrophicNetworkPlan,
    initial_states: NDArray[np.float64],
    elapsed: float,
    intervention: TrophicIntervention,
) -> TrophicResponse:
    """Runs an intervention and an unforced control through one compiled plan."""

    if intervention.consumer_names != plan.consumer_names:
        raise ValueError("intervention consumer_names must match the compiled plan")
    baseline = simulate_trophic_trajectory(
        plan,
        initial_states,
        elapsed,
        TrophicIntervention.unforced(
            intervention.batch, intervention.steps, intervention.consumer_names
        ),
    )
    perturbed = simulate_trophic_trajectory(
        plan, initial_states, elapsed, intervention
    )
    return TrophicResponse(baseline, perturbed)


def audit_exact_trophic_run(
    plan: TrophicNetworkPlan,
    initial: Mapping[str, float],
    elapsed: float,
    intervention: TrophicIntervention,
    *,
    batch_index: int = 0,
) -> ExactTrophicAudit:
    """Replays one forcing row exactly and returns its institutional evidence."""

    if intervention.consumer_names != plan.consumer_names:
        raise ValueError("intervention consumer_names must match the compiled plan")
    batch_index = _dimension("batch_index", batch_index)
    if batch_index >= intervention.batch:
        raise ValueError("batch_index must identify an intervention row")
    run = plan.start(initial)
    trajectory = np.empty(
        (1, intervention.steps + 1, len(plan.stock_names)), dtype=np.float64
    )
    trajectory[0, 0] = [run.stock(name) for name in plan.stock_names]
    for step in range(intervention.steps):
        harvests = {
            name: float(intervention.harvests[batch_index, step, index])
            for index, name in enumerate(plan.consumer_names)
            if intervention.harvests[batch_index, step, index] != 0.0
        }
        run.step(
            elapsed,
            float(intervention.nutrient_inputs[batch_index, step]),
            harvests,
        )
        trajectory[0, step + 1] = [run.stock(name) for name in plan.stock_names]
    return ExactTrophicAudit(
        trajectory=TrophicTrajectory(
            trajectory,
            plan.stock_names,
            plan.consumer_names,
        ),
        evidence=run.law_evidence(),
        cumulative_input=run.inputs,
        cumulative_output=run.outputs,
        exact_terminal=run.exact_stocks,
        transitions=run.transitions,
        _run=run,
    )
