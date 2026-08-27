"""Reproduce the declared exploratory grid for the frozen cascade scenario.

This script reports every parameter tuple that has all three qualitative sign
patterns.  It does not rank candidates or modify confirmatory source data.
"""

from __future__ import annotations

import itertools
import json

import numpy as np

from ecosim import (
    Consumer,
    DenseTrophicNetworkPlan,
    Feeding,
    Producer,
    TrophicIntervention,
    TrophicNetworkSpec,
    simulate_trophic_response,
    simulate_trophic_trajectory,
)

PREDATOR_RATES = (0.01, 0.02, 0.03)
PREDATOR_HALF_SATURATIONS = (1.0, 3.0)
PREDATOR_INITIALS = (1.0, 2.0)
HERBIVORE_RATES = (0.3, 0.4)
HARVEST_AMOUNTS = (0.005, 0.01)


def observe(
    predator_rate: float,
    predator_half_saturation: float,
    predator_initial: float,
    herbivore_rate: float,
    harvest_amount: float,
) -> dict[str, object]:
    spec = TrophicNetworkSpec(
        producers=(Producer("producer", 0.5, 10.0, 0.01),),
        consumers=(Consumer("herbivore", 0.02), Consumer("predator", 0.01)),
        feedings=(
            Feeding("herbivore", "producer", herbivore_rate, 5.0, 0.7),
            Feeding(
                "predator",
                "herbivore",
                predator_rate,
                predator_half_saturation,
                0.7,
            ),
        ),
        decomposition=0.1,
    )
    plan = DenseTrophicNetworkPlan(spec)
    predator_free = np.asarray([[100.0, 10.0, 5.0, 0.0, 0.0]])
    predator_present = np.asarray(
        [[100.0, 10.0, 5.0, predator_initial, 0.0]]
    )
    control = TrophicIntervention.unforced(1, 200, plan.consumer_names)
    free = simulate_trophic_trajectory(plan, predator_free, 0.05, control)
    present = simulate_trophic_trajectory(plan, predator_present, 0.05, control)
    cascade_delta = present.values[:, -1] - free.values[:, -1]
    pulse = simulate_trophic_response(
        plan,
        predator_present,
        0.05,
        TrophicIntervention.nutrient_pulse(
            1, 200, plan.consumer_names, at=20, amount=10.0
        ),
    )
    harvest = simulate_trophic_response(
        plan,
        predator_present,
        0.05,
        TrophicIntervention.consumer_harvest_press(
            1,
            200,
            plan.consumer_names,
            consumer="predator",
            start=50,
            stop=150,
            amount_per_step=harvest_amount,
        ),
    )
    indices = {name: index for index, name in enumerate(plan.stock_names)}
    response = {
        "predator_presence": {
            "producer": float(cascade_delta[0, indices["producer"]]),
            "herbivore": float(cascade_delta[0, indices["herbivore"]]),
        },
        "nutrient_pulse": {
            name: float(pulse.terminal_delta(name)[0])
            for name in ("producer", "herbivore", "predator")
        },
        "predator_harvest_press": {
            name: float(harvest.terminal_delta(name)[0])
            for name in ("producer", "herbivore", "predator")
        },
    }
    signs_match = (
        response["predator_presence"]["producer"] > 0.0
        and response["predator_presence"]["herbivore"] < 0.0
        and all(value > 0.0 for value in response["nutrient_pulse"].values())
        and response["predator_harvest_press"]["producer"] < 0.0
        and response["predator_harvest_press"]["herbivore"] > 0.0
        and response["predator_harvest_press"]["predator"] < 0.0
    )
    return {
        "parameters": {
            "predator_rate": predator_rate,
            "predator_half_saturation": predator_half_saturation,
            "predator_initial": predator_initial,
            "herbivore_rate": herbivore_rate,
            "harvest_amount_per_step": harvest_amount,
        },
        "responses": response,
        "qualitative_signs_match": signs_match,
    }


def main() -> None:
    observations = (
        observe(*parameters)
        for parameters in itertools.product(
            PREDATOR_RATES,
            PREDATOR_HALF_SATURATIONS,
            PREDATOR_INITIALS,
            HERBIVORE_RATES,
            HARVEST_AMOUNTS,
        )
    )
    candidates = [
        observation
        for observation in observations
        if observation["qualitative_signs_match"]
    ]
    print(
        json.dumps(
            {
                "status": "exploratory",
                "grid_size": 48,
                "matching_candidates": len(candidates),
                "candidates": candidates,
            },
            indent=2,
            sort_keys=True,
        )
    )


if __name__ == "__main__":
    main()
