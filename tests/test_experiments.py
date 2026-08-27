from collections.abc import Callable

import numpy as np
import pytest
from hypothesis import given, settings
from hypothesis import strategies as st

from ecosim import (
    BoundaryPerturbation,
    FoodWebParameters,
    simulate_food_web_response,
)


def test_pulse_is_applied_to_exactly_one_transition() -> None:
    pulse = BoundaryPerturbation.pulse(
        batch=2,
        steps=5,
        at=2,
        nutrient_input=3.0,
        harvest=1.0,
    )

    np.testing.assert_array_equal(
        pulse.nutrient_inputs,
        np.array([[0.0, 0.0, 3.0, 0.0, 0.0]] * 2),
    )
    np.testing.assert_array_equal(
        pulse.harvests,
        np.array([[0.0, 0.0, 1.0, 0.0, 0.0]] * 2),
    )
    assert not pulse.nutrient_inputs.flags.writeable
    assert not pulse.harvests.flags.writeable


def test_press_is_applied_over_a_half_open_step_interval() -> None:
    press = BoundaryPerturbation.press(
        batch=1,
        steps=6,
        start=1,
        stop=4,
        nutrient_input_per_step=2.0,
    )

    np.testing.assert_array_equal(
        press.nutrient_inputs,
        np.array([[0.0, 2.0, 2.0, 2.0, 0.0, 0.0]]),
    )
    np.testing.assert_array_equal(press.harvests, np.zeros((1, 6)))


def test_paired_response_exposes_peak_terminal_and_recovery_observations() -> None:
    initial = np.array([[10.0, 0.0, 0.0, 0.0]])
    no_internal_processes = FoodWebParameters(
        max_growth=0.0,
        max_grazing=0.0,
        producer_mortality=0.0,
        consumer_mortality=0.0,
        decomposition=0.0,
    )
    pulse = BoundaryPerturbation.pulse(1, 4, at=1, nutrient_input=5.0)

    response = simulate_food_web_response(
        initial,
        elapsed=1.0,
        perturbation=pulse,
        parameters=no_internal_processes,
    )

    np.testing.assert_array_equal(response.delta[0, :, 0], [0.0, 0.0, 5.0, 5.0, 5.0])
    np.testing.assert_array_equal(response.stock_delta("producer"), np.zeros((1, 5)))
    np.testing.assert_array_equal(response.peak_absolute_delta("nutrient"), [5.0])
    np.testing.assert_array_equal(response.terminal_delta("nutrient"), [5.0])
    assert response.recovery_step("nutrient", tolerance=0.1) == (None,)
    assert response.recovery_step("producer", tolerance=0.0) == (0,)


def test_perturbations_propagate_through_the_trophic_web() -> None:
    initial = np.array([[20.0, 5.0, 2.0, 0.0]])
    nutrient_pulse = BoundaryPerturbation.pulse(
        1, 100, at=0, nutrient_input=5.0
    )
    harvest_press = BoundaryPerturbation.press(
        1, 100, start=0, stop=50, harvest_per_step=0.01
    )

    enrichment = simulate_food_web_response(initial, 0.1, nutrient_pulse)
    harvesting = simulate_food_web_response(initial, 0.1, harvest_press)

    assert enrichment.peak_absolute_delta("producer")[0] > 0.0
    assert enrichment.terminal_delta("consumer")[0] > 0.0
    assert harvesting.terminal_delta("consumer")[0] < 0.0
    assert harvesting.terminal_delta("producer")[0] > 0.0


@given(
    batch=st.integers(min_value=1, max_value=4),
    steps=st.integers(min_value=1, max_value=10),
    amount=st.integers(min_value=0, max_value=100).map(float),
    data=st.data(),
)
@settings(max_examples=30, deadline=None)
def test_nutrient_pulse_response_obeys_the_boundary_ledger(
    batch: int,
    steps: int,
    amount: float,
    data: st.DataObject,
) -> None:
    at = data.draw(st.integers(min_value=0, max_value=steps - 1))
    initial = np.full((batch, 4), 10.0)
    pulse = BoundaryPerturbation.pulse(batch, steps, at=at, nutrient_input=amount)

    response = simulate_food_web_response(initial, elapsed=0.1, perturbation=pulse)
    total_delta = response.delta.sum(axis=2)

    np.testing.assert_allclose(total_delta[:, : at + 1], 0.0, atol=1e-12)
    np.testing.assert_allclose(total_delta[:, at + 1 :], amount, atol=1e-11)


@pytest.mark.parametrize(
    "operation",
    [
        lambda: BoundaryPerturbation.pulse(1, 0, at=0, nutrient_input=1.0),
        lambda: BoundaryPerturbation.pulse(1, 2, at=2, nutrient_input=1.0),
        lambda: BoundaryPerturbation.press(1, 3, start=2, stop=2),
        lambda: BoundaryPerturbation.press(
            1, 3, start=0, stop=2, nutrient_input_per_step=-1.0
        ),
    ],
)
def test_invalid_perturbations_are_rejected(operation: Callable[[], object]) -> None:
    with pytest.raises(ValueError):
        operation()
