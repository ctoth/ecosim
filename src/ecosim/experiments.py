"""Controlled counterfactual experiments for the built-in food web."""

from dataclasses import dataclass, field

import numpy as np
from numpy.typing import NDArray

from ecosim._core import FoodWebParameters, simulate_food_web


_STOCK_INDEX = {
    "nutrient": 0,
    "producer": 1,
    "consumer": 2,
    "detritus": 3,
}


def _nonnegative_amount(name: str, value: float) -> float:
    amount = float(value)
    if not np.isfinite(amount) or amount < 0.0:
        raise ValueError(f"{name} must be finite and nonnegative")
    return amount


def _dimension(name: str, value: int, *, positive: bool = False) -> int:
    if isinstance(value, bool):
        raise ValueError(f"{name} must be an integer")
    lower = 1 if positive else 0
    if value < lower:
        qualifier = "positive" if positive else "nonnegative"
        raise ValueError(f"{name} must be {qualifier}")
    return value


def _frozen_forcing(name: str, values: NDArray[np.float64]) -> NDArray[np.float64]:
    result = np.array(values, dtype=np.float64, copy=True)
    if result.ndim != 2:
        raise ValueError(f"{name} must be a two-dimensional array")
    if not np.all(np.isfinite(result)) or np.any(result < 0.0):
        raise ValueError(f"{name} values must be finite and nonnegative")
    result.flags.writeable = False
    return result


@dataclass(frozen=True)
class BoundaryPerturbation:
    """Per-step nutrient inputs and consumer harvests for a paired experiment."""

    nutrient_inputs: NDArray[np.float64]
    harvests: NDArray[np.float64]

    def __post_init__(self) -> None:
        nutrient_inputs = _frozen_forcing("nutrient_inputs", self.nutrient_inputs)
        harvests = _frozen_forcing("harvests", self.harvests)
        if nutrient_inputs.shape != harvests.shape:
            raise ValueError("nutrient_inputs and harvests must have the same shape")
        object.__setattr__(self, "nutrient_inputs", nutrient_inputs)
        object.__setattr__(self, "harvests", harvests)

    @property
    def batch(self) -> int:
        """Number of independent initial states addressed by the perturbation."""

        return self.nutrient_inputs.shape[0]

    @property
    def steps(self) -> int:
        """Number of model transitions addressed by the perturbation."""

        return self.nutrient_inputs.shape[1]

    @classmethod
    def pulse(
        cls,
        batch: int,
        steps: int,
        *,
        at: int,
        nutrient_input: float = 0.0,
        harvest: float = 0.0,
    ) -> "BoundaryPerturbation":
        """Applies boundary amounts during exactly one model transition."""

        batch = _dimension("batch", batch)
        steps = _dimension("steps", steps, positive=True)
        at = _dimension("at", at)
        if at >= steps:
            raise ValueError("at must identify a transition before steps")
        nutrient_input = _nonnegative_amount("nutrient_input", nutrient_input)
        harvest = _nonnegative_amount("harvest", harvest)
        nutrient_inputs = np.zeros((batch, steps), dtype=np.float64)
        harvests = np.zeros((batch, steps), dtype=np.float64)
        nutrient_inputs[:, at] = nutrient_input
        harvests[:, at] = harvest
        return cls(nutrient_inputs, harvests)

    @classmethod
    def press(
        cls,
        batch: int,
        steps: int,
        *,
        start: int,
        stop: int | None = None,
        nutrient_input_per_step: float = 0.0,
        harvest_per_step: float = 0.0,
    ) -> "BoundaryPerturbation":
        """Applies fixed amounts over the half-open transition interval [start, stop)."""

        batch = _dimension("batch", batch)
        steps = _dimension("steps", steps, positive=True)
        start = _dimension("start", start)
        stop = steps if stop is None else _dimension("stop", stop)
        if start >= stop or stop > steps:
            raise ValueError("press interval must satisfy 0 <= start < stop <= steps")
        nutrient_input_per_step = _nonnegative_amount(
            "nutrient_input_per_step", nutrient_input_per_step
        )
        harvest_per_step = _nonnegative_amount("harvest_per_step", harvest_per_step)
        nutrient_inputs = np.zeros((batch, steps), dtype=np.float64)
        harvests = np.zeros((batch, steps), dtype=np.float64)
        nutrient_inputs[:, start:stop] = nutrient_input_per_step
        harvests[:, start:stop] = harvest_per_step
        return cls(nutrient_inputs, harvests)


def _frozen_trajectory(name: str, values: NDArray[np.float64]) -> NDArray[np.float64]:
    result = np.array(values, dtype=np.float64, copy=True)
    if result.ndim != 3 or result.shape[1] == 0 or result.shape[2] != 4:
        raise ValueError(f"{name} must have shape (batch, time, 4)")
    if not np.all(np.isfinite(result)) or np.any(result < 0.0):
        raise ValueError(f"{name} values must be finite and nonnegative")
    result.flags.writeable = False
    return result


@dataclass(frozen=True)
class FoodWebResponse:
    """Paired unforced and perturbed trajectories plus their signed difference."""

    baseline: NDArray[np.float64]
    perturbed: NDArray[np.float64]
    delta: NDArray[np.float64] = field(init=False)

    def __post_init__(self) -> None:
        baseline = _frozen_trajectory("baseline", self.baseline)
        perturbed = _frozen_trajectory("perturbed", self.perturbed)
        if baseline.shape != perturbed.shape:
            raise ValueError("baseline and perturbed trajectories must have the same shape")
        delta = perturbed - baseline
        delta.flags.writeable = False
        object.__setattr__(self, "baseline", baseline)
        object.__setattr__(self, "perturbed", perturbed)
        object.__setattr__(self, "delta", delta)

    def stock_delta(self, stock: str) -> NDArray[np.float64]:
        """Signed response trajectory for one named stock."""

        try:
            index = _STOCK_INDEX[stock]
        except KeyError as error:
            raise ValueError(f"unknown food-web stock: {stock}") from error
        return self.delta[:, :, index]

    def peak_absolute_delta(self, stock: str) -> NDArray[np.float64]:
        """Largest absolute deviation from baseline for each batch member."""

        return np.max(np.abs(self.stock_delta(stock)), axis=1)

    def terminal_delta(self, stock: str) -> NDArray[np.float64]:
        """Signed deviation from baseline at the final time point."""

        return self.stock_delta(stock)[:, -1]

    def recovery_step(self, stock: str, *, tolerance: float) -> tuple[int | None, ...]:
        """First post-peak point within tolerance, or ``None`` if not recovered."""

        tolerance = _nonnegative_amount("tolerance", tolerance)
        recovery: list[int | None] = []
        for trajectory in np.abs(self.stock_delta(stock)):
            peak = int(np.argmax(trajectory))
            if trajectory[peak] <= tolerance:
                recovery.append(0)
                continue
            candidates = np.flatnonzero(trajectory[peak + 1 :] <= tolerance)
            recovery.append(None if candidates.size == 0 else peak + 1 + int(candidates[0]))
        return tuple(recovery)


def simulate_food_web_response(
    initial_states: NDArray[np.float64],
    elapsed: float,
    perturbation: BoundaryPerturbation,
    parameters: FoodWebParameters | None = None,
) -> FoodWebResponse:
    """Compares a boundary perturbation with an unforced counterfactual."""

    baseline = simulate_food_web(
        initial_states,
        perturbation.steps,
        elapsed,
        parameters=parameters,
    )
    perturbed = simulate_food_web(
        initial_states,
        perturbation.steps,
        elapsed,
        perturbation.nutrient_inputs,
        perturbation.harvests,
        parameters=parameters,
    )
    return FoodWebResponse(baseline=baseline, perturbed=perturbed)
