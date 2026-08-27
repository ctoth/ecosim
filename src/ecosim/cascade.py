"""Frozen confirmatory three-level trophic-cascade experiment."""

from __future__ import annotations

import json
from dataclasses import dataclass
from importlib import resources
from types import MappingProxyType
from typing import Any, Literal, cast

import numpy as np
from numpy.typing import NDArray

from ecosim.network import (
    Consumer,
    DenseTrophicNetworkPlan,
    Feeding,
    Producer,
    TrophicNetworkPlan,
    TrophicNetworkSpec,
)
from ecosim.trophic_experiments import (
    ExactTrophicComparison,
    TrophicIntervention,
    TrophicResponse,
    simulate_trophic_response,
    simulate_trophic_trajectory,
    audit_exact_trophic_run,
)

Relation = Literal["minimum", "maximum"]
Sign = Literal["positive", "negative"]


@dataclass(frozen=True)
class ResponseClaim:
    """One preregistered terminal dense-response threshold."""

    comparison: str
    stock: str
    relation: Relation
    threshold: float


@dataclass(frozen=True)
class ExactSignClaim:
    """One preregistered response sign for the short exact audit."""

    comparison: str
    stock: str
    sign: Sign


@dataclass(frozen=True)
class ResponseObservation:
    """A named observed response evaluated against its frozen threshold."""

    comparison: str
    stock: str
    observed: float
    relation: Relation
    threshold: float

    @property
    def passed(self) -> bool:
        if self.relation == "minimum":
            return self.observed >= self.threshold
        return self.observed <= self.threshold


@dataclass(frozen=True)
class ExactSignObservation:
    """A named exact response evaluated against its frozen sign."""

    comparison: str
    stock: str
    observed: float
    sign: Sign
    epsilon: float

    @property
    def passed(self) -> bool:
        if self.sign == "positive":
            return self.observed > self.epsilon
        return self.observed < -self.epsilon


@dataclass(frozen=True)
class FrozenCascadeScenario:
    """Reviewable source-data interpretation of the frozen experiment."""

    specification: TrophicNetworkSpec
    predator_free: MappingProxyType[str, float]
    predator_present: MappingProxyType[str, float]
    steps: int
    elapsed: float
    pulse_at: int
    pulse_amount: float
    harvest_consumer: str
    harvest_start: int
    harvest_stop: int
    harvest_amount_per_step: float
    claims: tuple[ResponseClaim, ...]
    exact_steps: int
    exact_elapsed: float
    exact_pulse_at: int
    exact_harvest_start: int
    exact_harvest_stop: int
    exact_harvest_amount_per_step: float
    exact_sign_claims: tuple[ExactSignClaim, ...]
    response_sign_epsilon: float
    dense_balance_absolute: float
    dense_balance_relative: float

    def initial_array(
        self,
        plan: DenseTrophicNetworkPlan,
        state: Literal["predator_free", "predator_present"],
    ) -> NDArray[np.float64]:
        values = self.predator_free if state == "predator_free" else self.predator_present
        return np.asarray([[values[name] for name in plan.stock_names]], dtype=np.float64)

    def unforced(self, consumer_names: tuple[str, ...], *, exact: bool = False) -> TrophicIntervention:
        steps = self.exact_steps if exact else self.steps
        return TrophicIntervention.unforced(1, steps, consumer_names)

    def nutrient_pulse(
        self, consumer_names: tuple[str, ...], *, exact: bool = False
    ) -> TrophicIntervention:
        return TrophicIntervention.nutrient_pulse(
            1,
            self.exact_steps if exact else self.steps,
            consumer_names,
            at=self.exact_pulse_at if exact else self.pulse_at,
            amount=self.pulse_amount,
        )

    def predator_harvest(
        self, consumer_names: tuple[str, ...], *, exact: bool = False
    ) -> TrophicIntervention:
        return TrophicIntervention.consumer_harvest_press(
            1,
            self.exact_steps if exact else self.steps,
            consumer_names,
            consumer=self.harvest_consumer,
            start=self.exact_harvest_start if exact else self.harvest_start,
            stop=self.exact_harvest_stop if exact else self.harvest_stop,
            amount_per_step=(
                self.exact_harvest_amount_per_step
                if exact
                else self.harvest_amount_per_step
            ),
        )


@dataclass(frozen=True)
class CascadeExperimentResult:
    """Dense confirmation plus separately identified exact evidence audits."""

    predator_presence: TrophicResponse
    nutrient_pulse: TrophicResponse
    predator_harvest_press: TrophicResponse
    observations: tuple[ResponseObservation, ...]
    exact_predator_presence: ExactTrophicComparison
    exact_nutrient_pulse: ExactTrophicComparison
    exact_predator_harvest_press: ExactTrophicComparison
    exact_sign_observations: tuple[ExactSignObservation, ...]

    @property
    def confirmation_passed(self) -> bool:
        return all(observation.passed for observation in self.observations)

    @property
    def exact_signs_passed(self) -> bool:
        return all(observation.passed for observation in self.exact_sign_observations)

    @property
    def exact_evidence_satisfied(self) -> bool:
        audits = (
            self.exact_predator_presence.baseline,
            self.exact_predator_presence.perturbed,
            self.exact_nutrient_pulse.baseline,
            self.exact_nutrient_pulse.perturbed,
            self.exact_predator_harvest_press.baseline,
            self.exact_predator_harvest_press.perturbed,
        )
        return all(audit.evidence.satisfied for audit in audits)


def _mapping(values: dict[str, Any]) -> MappingProxyType[str, float]:
    return MappingProxyType({str(name): float(value) for name, value in values.items()})


def _claims(values: dict[str, Any]) -> tuple[ResponseClaim, ...]:
    result: list[ResponseClaim] = []
    for comparison, raw_claims in values.items():
        for key, threshold in cast(dict[str, Any], raw_claims).items():
            if key.endswith("_terminal_delta_min"):
                stock = key.removesuffix("_terminal_delta_min")
                relation: Relation = "minimum"
            elif key.endswith("_terminal_delta_max"):
                stock = key.removesuffix("_terminal_delta_max")
                relation = "maximum"
            else:
                raise ValueError(f"unknown confirmatory claim: {key}")
            result.append(ResponseClaim(comparison, stock, relation, float(threshold)))
    return tuple(result)


def _exact_sign_claims(values: dict[str, Any]) -> tuple[ExactSignClaim, ...]:
    result: list[ExactSignClaim] = []
    for comparison, raw_claims in values.items():
        for stock, raw_sign in cast(dict[str, Any], raw_claims).items():
            sign = str(raw_sign)
            if sign not in {"positive", "negative"}:
                raise ValueError(f"unknown exact response sign: {sign}")
            result.append(ExactSignClaim(comparison, stock, cast(Sign, sign)))
    return tuple(result)


def load_frozen_cascade() -> FrozenCascadeScenario:
    """Loads the committed package data for the confirmatory experiment."""

    raw = json.loads(
        resources.files("ecosim")
        .joinpath("data/three_level_cascade.json")
        .read_text(encoding="utf-8")
    )
    data = cast(dict[str, Any], raw)
    network = cast(dict[str, Any], data["network"])
    producers = tuple(
        Producer(
            str(item["name"]),
            float(item["max_growth"]),
            float(item["nutrient_half_saturation"]),
            float(item["mortality"]),
        )
        for item in cast(list[dict[str, Any]], network["producers"])
    )
    consumers = tuple(
        Consumer(str(item["name"]), float(item["mortality"]))
        for item in cast(list[dict[str, Any]], network["consumers"])
    )
    feedings = tuple(
        Feeding(
            str(item["consumer"]),
            str(item["resource"]),
            float(item["max_rate"]),
            float(item["resource_half_saturation"]),
            float(item["assimilation_efficiency"]),
        )
        for item in cast(list[dict[str, Any]], network["feedings"])
    )
    initial = cast(dict[str, dict[str, Any]], data["initial_states"])
    simulation = cast(dict[str, Any], data["simulation"])
    interventions = cast(dict[str, dict[str, Any]], data["interventions"])
    pulse = interventions["nutrient_pulse"]
    harvest = interventions["predator_harvest_press"]
    exact = cast(dict[str, Any], data["exact_audit"])
    tolerances = cast(dict[str, Any], data["numerical_tolerances"])
    return FrozenCascadeScenario(
        specification=TrophicNetworkSpec(
            producers,
            consumers,
            feedings,
            float(network["decomposition"]),
        ),
        predator_free=_mapping(initial["predator_free"]),
        predator_present=_mapping(initial["predator_present"]),
        steps=int(simulation["steps"]),
        elapsed=float(simulation["elapsed"]),
        pulse_at=int(pulse["at"]),
        pulse_amount=float(pulse["amount"]),
        harvest_consumer=str(harvest["consumer"]),
        harvest_start=int(harvest["start"]),
        harvest_stop=int(harvest["stop"]),
        harvest_amount_per_step=float(harvest["amount_per_step"]),
        claims=_claims(cast(dict[str, Any], data["confirmatory_claims"])),
        exact_steps=int(exact["steps"]),
        exact_elapsed=float(exact["elapsed"]),
        exact_pulse_at=int(exact["nutrient_pulse_at"]),
        exact_harvest_start=int(exact["predator_harvest_start"]),
        exact_harvest_stop=int(exact["predator_harvest_stop"]),
        exact_harvest_amount_per_step=float(
            exact["predator_harvest_amount_per_step"]
        ),
        exact_sign_claims=_exact_sign_claims(
            cast(dict[str, Any], exact["required_response_signs"])
        ),
        response_sign_epsilon=float(tolerances["response_sign_epsilon"]),
        dense_balance_absolute=float(tolerances["dense_balance_absolute"]),
        dense_balance_relative=float(tolerances["dense_balance_relative"]),
    )


def _response_for(result: CascadeExperimentResult, comparison: str) -> TrophicResponse:
    if comparison == "predator_presence":
        return result.predator_presence
    if comparison == "nutrient_pulse":
        return result.nutrient_pulse
    if comparison == "predator_harvest_press":
        return result.predator_harvest_press
    raise ValueError(f"unknown cascade comparison: {comparison}")


def _exact_response_for(
    comparisons: dict[str, ExactTrophicComparison], comparison: str
) -> TrophicResponse:
    try:
        return comparisons[comparison].response
    except KeyError as error:
        raise ValueError(f"unknown exact cascade comparison: {comparison}") from error


def run_frozen_cascade(
    scenario: FrozenCascadeScenario | None = None,
) -> CascadeExperimentResult:
    """Runs dense confirmation and the separately frozen short exact audits."""

    scenario = load_frozen_cascade() if scenario is None else scenario
    dense_plan = DenseTrophicNetworkPlan(scenario.specification)
    unforced = scenario.unforced(dense_plan.consumer_names)
    free_initial = scenario.initial_array(dense_plan, "predator_free")
    present_initial = scenario.initial_array(dense_plan, "predator_present")
    free = simulate_trophic_trajectory(
        dense_plan, free_initial, scenario.elapsed, unforced
    )
    present = simulate_trophic_trajectory(
        dense_plan, present_initial, scenario.elapsed, unforced
    )
    predator_presence = TrophicResponse(free, present)
    nutrient_pulse = simulate_trophic_response(
        dense_plan,
        present_initial,
        scenario.elapsed,
        scenario.nutrient_pulse(dense_plan.consumer_names),
    )
    predator_harvest = simulate_trophic_response(
        dense_plan,
        present_initial,
        scenario.elapsed,
        scenario.predator_harvest(dense_plan.consumer_names),
    )

    exact_plan = TrophicNetworkPlan(scenario.specification)
    exact_unforced = scenario.unforced(exact_plan.consumer_names, exact=True)
    free_audit = audit_exact_trophic_run(
        exact_plan,
        scenario.predator_free,
        scenario.exact_elapsed,
        exact_unforced,
    )
    present_audit = audit_exact_trophic_run(
        exact_plan,
        scenario.predator_present,
        scenario.exact_elapsed,
        exact_unforced,
    )
    pulse_audit = audit_exact_trophic_run(
        exact_plan,
        scenario.predator_present,
        scenario.exact_elapsed,
        scenario.nutrient_pulse(exact_plan.consumer_names, exact=True),
    )
    harvest_audit = audit_exact_trophic_run(
        exact_plan,
        scenario.predator_present,
        scenario.exact_elapsed,
        scenario.predator_harvest(exact_plan.consumer_names, exact=True),
    )
    exact_comparisons = {
        "predator_presence": ExactTrophicComparison(free_audit, present_audit),
        "nutrient_pulse": ExactTrophicComparison(present_audit, pulse_audit),
        "predator_harvest_press": ExactTrophicComparison(present_audit, harvest_audit),
    }

    provisional = CascadeExperimentResult(
        predator_presence=predator_presence,
        nutrient_pulse=nutrient_pulse,
        predator_harvest_press=predator_harvest,
        observations=(),
        exact_predator_presence=exact_comparisons["predator_presence"],
        exact_nutrient_pulse=exact_comparisons["nutrient_pulse"],
        exact_predator_harvest_press=exact_comparisons["predator_harvest_press"],
        exact_sign_observations=(),
    )
    observations = tuple(
        ResponseObservation(
            claim.comparison,
            claim.stock,
            float(_response_for(provisional, claim.comparison).terminal_delta(claim.stock)[0]),
            claim.relation,
            claim.threshold,
        )
        for claim in scenario.claims
    )
    exact_sign_observations = tuple(
        ExactSignObservation(
            claim.comparison,
            claim.stock,
            float(
                _exact_response_for(exact_comparisons, claim.comparison)
                .terminal_delta(claim.stock)[0]
            ),
            claim.sign,
            scenario.response_sign_epsilon,
        )
        for claim in scenario.exact_sign_claims
    )
    return CascadeExperimentResult(
        predator_presence=predator_presence,
        nutrient_pulse=nutrient_pulse,
        predator_harvest_press=predator_harvest,
        observations=observations,
        exact_predator_presence=exact_comparisons["predator_presence"],
        exact_nutrient_pulse=exact_comparisons["nutrient_pulse"],
        exact_predator_harvest_press=exact_comparisons["predator_harvest_press"],
        exact_sign_observations=exact_sign_observations,
    )
