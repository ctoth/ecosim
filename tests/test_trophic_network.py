import math
import sys
import threading
import time
from fractions import Fraction

import numpy as np
import pytest
from hypothesis import given, settings
from hypothesis import strategies as st

from ecosim import (
    Consumer,
    DenseTrophicNetwork,
    DenseTrophicNetworkPlan,
    Feeding,
    Producer,
    TrophicNetwork,
    TrophicNetworkPlan,
    TrophicNetworkSpec,
)


def network_spec(grazing: float = 0.8) -> TrophicNetworkSpec:
    return TrophicNetworkSpec(
        producers=(
            Producer("grazed", 0.5, 10.0, 0.01),
            Producer("refuge", 0.5, 10.0, 0.01),
        ),
        consumers=(Consumer("grazer", 0.02),),
        feedings=(Feeding("grazer", "grazed", grazing, 5.0, 0.75),),
        decomposition=0.1,
    )


def initial_stocks() -> dict[str, float]:
    return {
        "nutrient": 100.0,
        "grazed": 10.0,
        "refuge": 10.0,
        "grazer": 5.0,
        "detritus": 0.0,
    }


def snapshot(network: TrophicNetwork | DenseTrophicNetwork) -> tuple[float, ...]:
    return (network.time, *(network.stock(name) for name in network.stock_names))


def ordered_initial(plan: DenseTrophicNetworkPlan) -> np.ndarray:
    initial = initial_stocks()
    return np.asarray([[initial[name] for name in plan.stock_names]], dtype=np.float64)


def test_exact_and_dense_python_networks_share_named_process_semantics() -> None:
    exact = TrophicNetwork(network_spec(), initial_stocks())
    dense = DenseTrophicNetwork(network_spec(), initial_stocks())

    exact_step = exact.step(0.25, nutrient_input=2.0, harvests={"grazer": 0.5})
    dense_step = dense.step(0.25, nutrient_input=2.0, harvests={"grazer": 0.5})

    assert exact.stock_names == (
        "nutrient",
        "grazed",
        "refuge",
        "grazer",
        "detritus",
    )
    for stock in exact.stock_names:
        assert dense.stock(stock) == pytest.approx(exact.stock(stock), abs=1e-12)
    for producer in ("grazed", "refuge"):
        assert dense_step.growth(producer) == pytest.approx(
            exact_step.growth(producer), abs=1e-12
        )
    assert dense_step.feeding("grazer", "grazed") == pytest.approx(
        exact_step.feeding("grazer", "grazed"), abs=1e-12
    )
    assert exact.balanced
    assert dense.balanced


def test_compiled_plans_start_independent_runs() -> None:
    exact_plan = TrophicNetworkPlan(network_spec())
    dense_plan = DenseTrophicNetworkPlan(network_spec())

    exact_changed = exact_plan.start(initial_stocks())
    exact_untouched = exact_plan.start(initial_stocks())
    dense_changed = dense_plan.start(initial_stocks())
    dense_untouched = dense_plan.start(initial_stocks())

    exact_changed.step(0.25, nutrient_input=2.0)
    dense_changed.step(0.25, nutrient_input=2.0)

    assert exact_plan.stock_names == dense_plan.stock_names
    assert exact_plan.consumer_names == dense_plan.consumer_names == ("grazer",)
    assert snapshot(exact_untouched) == snapshot(TrophicNetwork(network_spec(), initial_stocks()))
    assert snapshot(dense_untouched) == snapshot(DenseTrophicNetwork(network_spec(), initial_stocks()))
    assert snapshot(exact_changed) != snapshot(exact_untouched)
    assert snapshot(dense_changed) != snapshot(dense_untouched)


def test_exact_python_plan_exposes_compiled_evidence_and_run_verdicts() -> None:
    plan = TrophicNetworkPlan(network_spec())
    run = plan.start(initial_stocks())

    assert plan.evidence_axis_names == (
        "nutrient",
        "grazed",
        "refuge",
        "grazer",
        "detritus",
        "cumulative_input",
        "cumulative_output",
    )
    assert plan.evidence_laws[0].name == "material_invariant"
    assert plan.evidence_laws[0].grade == "invariant"
    assert run.trace_length == 1
    with pytest.raises(ValueError, match="at least two"):
        run.evidence()

    run.step(0.25, nutrient_input=2.0, harvests={"grazer": 0.5})
    evidence = run.evidence()
    law_evidence = run.law_evidence()

    assert run.trace_length == 2
    assert evidence.satisfied
    assert len(evidence.laws) == 8
    assert all(law.satisfied for law in evidence.laws)
    assert law_evidence.satisfied
    assert len(law_evidence.laws) == 13
    assert plan.law_suite_laws[0].name == "transition_equation"
    assert plan.law_suite_laws[-1].name == "open_material_balance"
    assert run.transition_count == 1
    assert run.transitions[0].time_before == Fraction(0)
    assert run.transitions[0].time_after == Fraction(1, 4)
    assert all(flow.settled <= flow.proposed for flow in run.transitions[0].flows)
    assert run.exact_stocks["nutrient"].denominator > 0


def test_dense_plan_batch_matches_individual_runs_with_forcing() -> None:
    plan = DenseTrophicNetworkPlan(network_spec())
    initial = np.concatenate((ordered_initial(plan), ordered_initial(plan) * 0.5), axis=0)
    nutrient_inputs = np.asarray([[1.0, 0.0, 2.0], [0.0, 3.0, 0.5]])
    harvests = np.asarray([[[0.0], [0.5], [0.0]], [[0.25], [0.0], [1.0]]])

    batched = plan.simulate(initial, 3, 0.25, nutrient_inputs, harvests)

    expected = np.empty_like(batched)
    for batch_index, values in enumerate(initial):
        run = plan.start(dict(zip(plan.stock_names, values, strict=True)))
        expected[batch_index, 0] = values
        for step_index in range(3):
            run.step(
                0.25,
                float(nutrient_inputs[batch_index, step_index]),
                {"grazer": float(harvests[batch_index, step_index, 0])},
            )
            expected[batch_index, step_index + 1] = [
                run.stock(name) for name in plan.stock_names
            ]

    assert batched.shape == (2, 4, 5)
    np.testing.assert_array_equal(batched, expected)


def test_dense_plan_batch_accepts_strided_arrays_and_rejects_bad_inputs() -> None:
    plan = DenseTrophicNetworkPlan(network_spec())
    initial = np.repeat(ordered_initial(plan), 2, axis=0)
    initial_strided = np.repeat(initial, 2, axis=1)[:, ::2]
    nutrients = np.asarray([[1.0, 2.0, 3.0], [4.0, 5.0, 6.0]])
    nutrient_strided = np.repeat(nutrients, 2, axis=1)[:, ::2]
    harvests = np.asarray([[[0.0], [0.5], [0.0]], [[0.25], [0.0], [1.0]]])
    harvest_strided = np.repeat(harvests, 2, axis=1)[:, ::2, :]

    contiguous = plan.simulate(initial, 3, 0.25, nutrients, harvests)
    strided = plan.simulate(
        initial_strided, 3, 0.25, nutrient_strided, harvest_strided
    )

    np.testing.assert_array_equal(strided, contiguous)
    with pytest.raises(ValueError, match="initial_states must have shape"):
        plan.simulate(np.zeros((2, 4)), 3, 0.25)
    with pytest.raises(ValueError, match="nutrient_inputs must have shape"):
        plan.simulate(initial, 3, 0.25, np.zeros((2, 2)))
    with pytest.raises(ValueError, match="harvests must have shape"):
        plan.simulate(initial, 3, 0.25, harvests=np.zeros((2, 3, 2)))
    with pytest.raises(ValueError, match="finite and nonnegative"):
        invalid = initial.copy()
        invalid[0, 0] = math.nan
        plan.simulate(invalid, 3, 0.25)


def test_dense_plan_batch_handles_empty_and_zero_step_ensembles() -> None:
    plan = DenseTrophicNetworkPlan(network_spec())
    empty = plan.simulate(np.zeros((0, 5), dtype=np.float64), 0, 0.0)
    initial = ordered_initial(plan)
    zero_step = plan.simulate(initial, 0, 0.0)

    assert empty.shape == (0, 1, 5)
    np.testing.assert_array_equal(zero_step, initial[:, np.newaxis, :])
    with pytest.raises(ValueError):
        plan.simulate(initial, sys.maxsize, 1.0)


def test_dense_plan_batch_releases_the_gil_during_owned_computation() -> None:
    plan = DenseTrophicNetworkPlan(network_spec())
    rendezvous = threading.Barrier(2)
    progressed = threading.Event()

    def worker() -> None:
        rendezvous.wait()
        time.sleep(0.01)
        progressed.set()

    thread = threading.Thread(target=worker)
    thread.start()
    rendezvous.wait()
    initial = np.repeat(ordered_initial(plan), 8, axis=0)
    plan.simulate(initial, 10_000, 0.001)
    assert progressed.is_set()
    thread.join()


def test_selective_grazing_causes_competitive_release() -> None:
    control = DenseTrophicNetwork(network_spec(grazing=0.0), initial_stocks())
    selective = DenseTrophicNetwork(network_spec(grazing=0.8), initial_stocks())

    for _ in range(100):
        control.step(0.1)
        selective.step(0.1)

    assert selective.stock("grazed") < control.stock("grazed")
    assert selective.stock("refuge") > control.stock("refuge")
    assert selective.stock("grazer") > control.stock("grazer")
    assert selective.balanced


def test_invalid_structure_and_forcing_fail_atomically() -> None:
    malformed = TrophicNetworkSpec(
        producers=(Producer("plant", 1.0, 1.0, 0.0),),
        consumers=(Consumer("grazer", 0.0),),
        feedings=(Feeding("grazer", "missing", 1.0, 1.0, 0.5),),
        decomposition=0.0,
    )
    with pytest.raises(ValueError, match="unknown biotic resource"):
        DenseTrophicNetwork(
            malformed,
            {"nutrient": 1.0, "plant": 1.0, "grazer": 1.0, "detritus": 0.0},
        )

    network = DenseTrophicNetwork(network_spec(), initial_stocks())
    before = snapshot(network)
    with pytest.raises(ValueError, match="unknown harvest stock"):
        network.step(1.0, harvests={"grazed": 1.0})
    assert snapshot(network) == before


@given(
    elapsed=st.integers(min_value=0, max_value=8).map(lambda value: value / 4),
    nutrient_input=st.integers(min_value=0, max_value=100).map(float),
    harvest=st.integers(min_value=0, max_value=100).map(float),
)
@settings(max_examples=30, deadline=None)
def test_generated_boundary_forcing_preserves_dense_network_contracts(
    elapsed: float,
    nutrient_input: float,
    harvest: float,
) -> None:
    network = DenseTrophicNetwork(network_spec(), initial_stocks())
    network.step(elapsed, nutrient_input, {"grazer": harvest})

    assert all(math.isfinite(network.stock(name)) for name in network.stock_names)
    assert all(network.stock(name) >= 0.0 for name in network.stock_names)
    assert network.balanced


@given(
    initial_values=st.lists(
        st.integers(min_value=0, max_value=100), min_size=10, max_size=10
    ),
    input_values=st.lists(
        st.integers(min_value=0, max_value=20), min_size=8, max_size=8
    ),
    elapsed=st.integers(min_value=0, max_value=10).map(lambda value: value / 10),
)
@settings(max_examples=40, deadline=None)
def test_batched_closed_network_total_changes_only_by_nutrient_input(
    initial_values: list[int],
    input_values: list[int],
    elapsed: float,
) -> None:
    plan = DenseTrophicNetworkPlan(network_spec())
    initial = np.asarray(initial_values, dtype=np.float64).reshape(2, 5)
    nutrient_inputs = np.asarray(input_values, dtype=np.float64).reshape(2, 4)

    trajectory = plan.simulate(initial, 4, elapsed, nutrient_inputs)

    expected_totals = np.concatenate(
        (
            initial.sum(axis=1, keepdims=True),
            initial.sum(axis=1, keepdims=True) + np.cumsum(nutrient_inputs, axis=1),
        ),
        axis=1,
    )
    np.testing.assert_allclose(
        trajectory.sum(axis=2), expected_totals, rtol=1e-12, atol=1e-10
    )
