import numpy as np
import pytest

from ecosim import (
    CascadeExperimentResult,
    Consumer,
    DenseTrophicNetworkPlan,
    Feeding,
    Producer,
    TrophicIntervention,
    TrophicNetworkSpec,
    FrozenCascadeScenario,
    load_frozen_cascade,
    run_frozen_cascade,
    simulate_trophic_response,
)


@pytest.fixture(scope="module")
def frozen_result() -> tuple[FrozenCascadeScenario, CascadeExperimentResult]:
    scenario = load_frozen_cascade()
    return scenario, run_frozen_cascade(scenario)


def small_plan() -> DenseTrophicNetworkPlan:
    return DenseTrophicNetworkPlan(
        TrophicNetworkSpec(
            producers=(Producer("producer", 0.5, 10.0, 0.01),),
            consumers=(Consumer("herbivore", 0.02),),
            feedings=(Feeding("herbivore", "producer", 0.4, 5.0, 0.7),),
            decomposition=0.1,
        )
    )


def test_trophic_interventions_are_immutable_and_retain_consumer_axis() -> None:
    consumers = ("herbivore", "predator")
    pulse = TrophicIntervention.nutrient_pulse(
        2, 5, consumers, at=2, amount=3.0
    )
    harvest = TrophicIntervention.consumer_harvest_press(
        1,
        6,
        consumers,
        consumer="predator",
        start=1,
        stop=4,
        amount_per_step=0.25,
    )

    assert pulse.consumer_names == consumers
    np.testing.assert_array_equal(
        pulse.nutrient_inputs,
        np.asarray([[0.0, 0.0, 3.0, 0.0, 0.0]] * 2),
    )
    np.testing.assert_array_equal(pulse.harvests, np.zeros((2, 5, 2)))
    np.testing.assert_array_equal(harvest.harvests[0, :, 1], [0, 0.25, 0.25, 0.25, 0, 0])
    assert not pulse.nutrient_inputs.flags.writeable
    assert not harvest.harvests.flags.writeable


def test_generic_response_uses_one_plan_and_retains_named_axes() -> None:
    plan = small_plan()
    initial = np.asarray([[20.0, 5.0, 2.0, 0.0]])
    pulse = TrophicIntervention.nutrient_pulse(
        1, 20, plan.consumer_names, at=0, amount=5.0
    )

    response = simulate_trophic_response(plan, initial, 0.1, pulse)

    assert response.baseline.stock_names == plan.stock_names
    assert response.baseline.consumer_names == plan.consumer_names
    assert response.perturbed.stock_names == plan.stock_names
    assert response.terminal_delta("producer")[0] > 0.0
    assert response.peak_delta("producer")[0] > 0.0
    assert not response.delta.flags.writeable


def test_intervention_axis_mismatch_and_invalid_shapes_are_rejected() -> None:
    plan = small_plan()
    initial = np.asarray([[20.0, 5.0, 2.0, 0.0]])
    mismatch = TrophicIntervention.unforced(1, 2, ("other",))

    with pytest.raises(ValueError, match="consumer_names"):
        simulate_trophic_response(plan, initial, 0.1, mismatch)
    with pytest.raises(ValueError, match="harvests must have shape"):
        TrophicIntervention(("herbivore",), np.zeros((1, 2)), np.zeros((1, 2, 2)))


def test_frozen_cascade_confirms_behavior_exact_signs_and_institutional_evidence(
    frozen_result: tuple[FrozenCascadeScenario, CascadeExperimentResult],
) -> None:
    _, result = frozen_result

    assert result.confirmation_passed
    assert result.exact_signs_passed
    assert result.exact_evidence_satisfied
    assert all(observation.passed for observation in result.observations)
    assert all(observation.passed for observation in result.exact_sign_observations)
    for comparison in (
        result.exact_predator_presence,
        result.exact_nutrient_pulse,
        result.exact_predator_harvest_press,
    ):
        assert comparison.baseline.evidence.satisfied
        assert comparison.perturbed.evidence.satisfied
        assert len(comparison.baseline.evidence.laws) > 8
        assert len(comparison.perturbed.evidence.laws) > 8
        assert comparison.baseline.transitions
        assert comparison.perturbed.transitions
        assert comparison.baseline.exact_terminal
        assert comparison.perturbed.exact_terminal


def test_frozen_dense_trajectories_obey_preregistered_boundary_accounts(
    frozen_result: tuple[FrozenCascadeScenario, CascadeExperimentResult],
) -> None:
    scenario, result = frozen_result
    absolute = scenario.dense_balance_absolute
    relative = scenario.dense_balance_relative

    for trajectory in (
        result.predator_presence.baseline,
        result.predator_presence.perturbed,
        result.nutrient_pulse.baseline,
        result.predator_harvest_press.baseline,
    ):
        totals = trajectory.values.sum(axis=2)
        initial_totals = np.broadcast_to(totals[:, :1], totals.shape)
        scale = np.maximum(np.abs(totals), np.abs(initial_totals))
        assert np.all(np.abs(totals - initial_totals) <= absolute + relative * scale)

    pulse_total_delta = result.nutrient_pulse.delta.sum(axis=2)[0]
    expected_pulse = np.zeros(scenario.steps + 1)
    expected_pulse[scenario.pulse_at + 1 :] = scenario.pulse_amount
    pulse_scale = np.maximum(
        np.abs(result.nutrient_pulse.baseline.values.sum(axis=2)[0]),
        np.abs(result.nutrient_pulse.perturbed.values.sum(axis=2)[0]),
    )
    assert np.all(
        np.abs(pulse_total_delta - expected_pulse)
        <= absolute + relative * pulse_scale
    )

    harvest_total_delta = result.predator_harvest_press.delta.sum(axis=2)[0]
    expected_harvest = np.zeros(scenario.steps + 1)
    forcing = np.zeros(scenario.steps)
    forcing[scenario.harvest_start : scenario.harvest_stop] = (
        scenario.harvest_amount_per_step
    )
    expected_harvest[1:] = -np.cumsum(forcing)
    harvest_scale = np.maximum(
        np.abs(result.predator_harvest_press.baseline.values.sum(axis=2)[0]),
        np.abs(result.predator_harvest_press.perturbed.values.sum(axis=2)[0]),
    )
    assert np.all(
        np.abs(harvest_total_delta - expected_harvest)
        <= absolute + relative * harvest_scale
    )


def test_frozen_tolerances_match_the_declared_kernel_contract() -> None:
    scenario = load_frozen_cascade()

    assert scenario.dense_balance_absolute == 256.0 * np.finfo(np.float64).eps
    assert scenario.dense_balance_relative == 256.0 * np.finfo(np.float64).eps
