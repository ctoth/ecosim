import math
import sys
import threading
import time

import numpy as np
import pytest
from hypothesis import given
from hypothesis import settings
from hypothesis import strategies as st

from ecosim import DenseFoodWeb, FoodWeb, FoodWebParameters, simulate_food_web


FINITE_STOCK = st.integers(min_value=0, max_value=10_000).map(float)
FINITE_DURATION = st.integers(min_value=0, max_value=40).map(lambda value: value / 4)
FINITE_RATE = st.integers(min_value=0, max_value=40).map(lambda value: value / 20)
POSITIVE_HALF_SATURATION = st.integers(min_value=1, max_value=400).map(
    lambda value: value / 4
)


def test_trophic_processes_and_recycling_are_observable() -> None:
    web = FoodWeb(nutrient=100.0, producer=20.0, consumer=5.0, detritus=0.0)
    first = web.step(1.0)

    assert first.applied("producer-growth") > 0.0
    assert first.applied("grazing") > 0.0
    assert first.applied("producer-mortality") > 0.0
    assert first.applied("consumer-mortality") > 0.0
    assert (web.nutrient, web.producer, web.consumer, web.detritus) != (
        100.0,
        20.0,
        5.0,
        0.0,
    )

    second = web.step(1.0)
    assert second.applied("decomposition") > 0.0
    assert web.balanced
    assert web.balance_residual == 0.0


def test_custom_parameters_control_named_ecological_processes() -> None:
    parameters = FoodWebParameters(max_growth=0.0, max_grazing=0.0)
    exact = FoodWeb(100.0, 20.0, 5.0, 0.0, parameters=parameters)
    dense = DenseFoodWeb(100.0, 20.0, 5.0, 0.0, parameters=parameters)

    exact_step = exact.step(1.0)
    dense_step = dense.step(1.0)

    for step in (exact_step, dense_step):
        assert step.applied("producer-growth") == 0.0
        assert step.applied("grazing") == 0.0
        assert step.applied("producer-mortality") > 0.0
        assert step.applied("consumer-mortality") > 0.0
    assert exact.balanced
    assert dense.balanced


def test_custom_parameters_are_shared_by_object_and_batch_paths() -> None:
    parameters = FoodWebParameters(
        max_growth=0.25,
        nutrient_half_saturation=4.0,
        max_grazing=0.1,
        producer_half_saturation=3.0,
        producer_mortality=0.02,
        consumer_mortality=0.03,
        decomposition=0.2,
    )
    initial = np.array([[100.0, 20.0, 5.0, 2.0]], dtype=np.float64)
    batched = simulate_food_web(initial, 4, 0.25, parameters=parameters)
    web = DenseFoodWeb(*initial[0], parameters=parameters)
    expected = [initial[0].copy()]
    for _ in range(4):
        web.step(0.25)
        expected.append(np.array([web.nutrient, web.producer, web.consumer, web.detritus]))

    np.testing.assert_array_equal(batched[0], np.asarray(expected))


@pytest.mark.parametrize(
    "kwargs",
    [
        {"max_growth": -1.0},
        {"nutrient_half_saturation": 0.0},
        {"producer_half_saturation": math.inf},
    ],
)
def test_invalid_custom_parameters_are_rejected(kwargs: dict[str, float]) -> None:
    with pytest.raises(ValueError):
        FoodWebParameters(**kwargs)


@given(
    initial=st.tuples(
        st.integers(min_value=0, max_value=100).map(float),
        st.integers(min_value=0, max_value=100).map(float),
        st.integers(min_value=0, max_value=100).map(float),
        st.integers(min_value=0, max_value=100).map(float),
    ),
    elapsed=FINITE_DURATION,
    nutrient_input=st.integers(min_value=0, max_value=20).map(float),
    harvest=st.integers(min_value=0, max_value=20).map(float),
    max_growth=FINITE_RATE,
    nutrient_half_saturation=POSITIVE_HALF_SATURATION,
    max_grazing=FINITE_RATE,
    producer_half_saturation=POSITIVE_HALF_SATURATION,
    producer_mortality=FINITE_RATE,
    consumer_mortality=FINITE_RATE,
    decomposition=FINITE_RATE,
)
@settings(max_examples=40, deadline=None)
def test_custom_parameters_preserve_exact_dense_process_semantics(
    initial: tuple[float, float, float, float],
    elapsed: float,
    nutrient_input: float,
    harvest: float,
    max_growth: float,
    nutrient_half_saturation: float,
    max_grazing: float,
    producer_half_saturation: float,
    producer_mortality: float,
    consumer_mortality: float,
    decomposition: float,
) -> None:
    parameters = FoodWebParameters(
        max_growth=max_growth,
        nutrient_half_saturation=nutrient_half_saturation,
        max_grazing=max_grazing,
        producer_half_saturation=producer_half_saturation,
        producer_mortality=producer_mortality,
        consumer_mortality=consumer_mortality,
        decomposition=decomposition,
    )
    exact = FoodWeb(*initial, parameters=parameters)
    dense = DenseFoodWeb(*initial, parameters=parameters)

    exact_step = exact.step(elapsed, nutrient_input, harvest)
    dense_step = dense.step(elapsed, nutrient_input, harvest)

    for process in (
        "nutrient-input",
        "producer-growth",
        "grazing",
        "producer-mortality",
        "consumer-mortality",
        "decomposition",
        "harvest",
    ):
        assert dense_step.applied(process) == pytest.approx(
            exact_step.applied(process), rel=1e-12, abs=1e-12
        )
    assert (dense.nutrient, dense.producer, dense.consumer, dense.detritus) == pytest.approx(
        (exact.nutrient, exact.producer, exact.consumer, exact.detritus),
        rel=1e-12,
        abs=1e-12,
    )
    assert exact.balanced
    assert dense.balanced


@given(
    initial=st.tuples(FINITE_STOCK, FINITE_STOCK, FINITE_STOCK, FINITE_STOCK),
    steps=st.lists(
        st.tuples(FINITE_DURATION, FINITE_STOCK, FINITE_STOCK),
        min_size=1,
        max_size=8,
    ),
)
@settings(max_examples=30, deadline=None)
def test_arbitrary_forcing_preserves_nonnegative_stocks_and_open_balance(
    initial: tuple[float, float, float, float],
    steps: list[tuple[float, float, float]],
) -> None:
    web = FoodWeb(*initial)
    initial_total = sum(initial)

    for elapsed, nutrient_input, harvest in steps:
        web.step(elapsed, nutrient_input=nutrient_input, harvest=harvest)
        assert min(web.nutrient, web.producer, web.consumer, web.detritus) >= 0.0
        assert web.balanced
        assert web.balance_residual == 0.0
        assert math.isclose(
            initial_total + web.inputs - web.outputs,
            web.nutrient + web.producer + web.consumer + web.detritus,
            rel_tol=1e-12,
            abs_tol=1e-8,
        )


def test_invalid_float_inputs_are_rejected_without_advancing_time() -> None:
    web = FoodWeb(10.0, 1.0, 0.0, 0.0)
    for invalid in (-1.0, math.nan, math.inf):
        try:
            web.step(invalid)
        except ValueError:
            pass
        else:
            raise AssertionError("invalid duration was accepted")
    assert web.time == 0.0


def test_dense_food_web_supports_long_trajectories_and_atomic_rejection() -> None:
    web = DenseFoodWeb(100.0, 20.0, 5.0, 0.0)
    for _ in range(2_000):
        web.step(0.01, nutrient_input=0.002, harvest=0.001)
    assert web.time == pytest.approx(20.0)
    assert min(web.nutrient, web.producer, web.consumer, web.detritus) >= 0.0
    assert web.balanced

    before = (web.time, web.nutrient, web.producer, web.consumer, web.detritus)
    with pytest.raises(ValueError):
        web.step(math.nan)
    assert (web.time, web.nutrient, web.producer, web.consumer, web.detritus) == before


def test_batched_trajectory_has_documented_shape_and_independent_forcing() -> None:
    initial = np.array(
        [[100.0, 20.0, 5.0, 0.0], [20.0, 3.0, 1.0, 7.0]],
        dtype=np.float64,
    )
    nutrient_inputs = np.array([[1.0, 0.0, 2.0], [0.0, 3.0, 0.0]], dtype=np.float64)
    harvests = np.zeros((2, 3), dtype=np.float64)
    initial_before = initial.copy()
    inputs_before = nutrient_inputs.copy()

    result = simulate_food_web(initial, 3, 0.25, nutrient_inputs, harvests)

    assert result.shape == (2, 4, 4)
    np.testing.assert_array_equal(result[:, 0, :], initial)
    assert np.all(np.isfinite(result))
    assert np.all(result >= 0.0)
    np.testing.assert_allclose(
        result[:, -1, :].sum(axis=1),
        initial.sum(axis=1) + nutrient_inputs.sum(axis=1),
        rtol=1e-13,
        atol=1e-12,
    )
    np.testing.assert_array_equal(initial, initial_before)
    np.testing.assert_array_equal(nutrient_inputs, inputs_before)


def test_batched_trajectory_preserves_strided_input_semantics() -> None:
    initial_base = np.array(
        [
            [100.0, 20.0, 5.0, 0.0],
            [999.0, 999.0, 999.0, 999.0],
            [20.0, 3.0, 1.0, 7.0],
            [999.0, 999.0, 999.0, 999.0],
        ],
        dtype=np.float64,
    )
    forcing_base = np.array(
        [[1.0, 99.0, 0.0, 99.0, 2.0, 99.0], [0.0, 99.0, 3.0, 99.0, 0.0, 99.0]],
        dtype=np.float64,
    )
    initial = initial_base[::2]
    forcing = forcing_base[:, ::2]
    assert not initial.flags.c_contiguous
    assert not forcing.flags.c_contiguous

    strided = simulate_food_web(initial, 3, 0.25, forcing)
    contiguous = simulate_food_web(
        np.ascontiguousarray(initial),
        3,
        0.25,
        np.ascontiguousarray(forcing),
    )
    np.testing.assert_array_equal(strided, contiguous)


def test_batched_trajectory_releases_the_gil_during_owned_computation() -> None:
    rendezvous = threading.Barrier(2)
    progressed = threading.Event()

    def worker() -> None:
        rendezvous.wait()
        time.sleep(0.01)
        progressed.set()

    thread = threading.Thread(target=worker)
    thread.start()
    rendezvous.wait()
    initial = np.tile(np.array([[100.0, 20.0, 5.0, 0.0]]), (8, 1))
    simulate_food_web(initial, 20_000, 0.001)
    assert progressed.is_set()
    thread.join()


@pytest.mark.parametrize("steps", [sys.maxsize, (1 << (8 * np.dtype(np.uintp).itemsize)) - 1])
def test_batched_trajectory_rejects_output_shape_overflow_as_value_error(steps: int) -> None:
    for batch in (0, 1):
        with pytest.raises(ValueError):
            simulate_food_web(np.zeros((batch, 4), dtype=np.float64), steps, 1.0)


@given(
    initial=st.lists(
        st.lists(st.floats(min_value=0.0, max_value=1_000.0, allow_nan=False), min_size=4, max_size=4),
        min_size=1,
        max_size=5,
    ),
    steps=st.integers(min_value=0, max_value=12),
    elapsed=st.floats(min_value=0.0, max_value=1.0, allow_nan=False),
)
@settings(max_examples=30, deadline=None)
def test_batched_trajectory_preserves_nonnegativity_and_closed_balance(
    initial: list[list[float]], steps: int, elapsed: float
) -> None:
    initial_array = np.asarray(initial, dtype=np.float64)
    result = simulate_food_web(initial_array, steps, elapsed)
    assert result.shape == (len(initial), steps + 1, 4)
    assert np.all(np.isfinite(result))
    assert np.all(result >= 0.0)
    np.testing.assert_allclose(
        result[:, -1, :].sum(axis=1),
        initial_array.sum(axis=1),
        rtol=1e-12,
        atol=1e-10,
    )


@pytest.mark.parametrize(
    ("initial", "steps", "elapsed", "nutrient_inputs", "harvests"),
    [
        (np.zeros((2, 3)), 2, 1.0, None, None),
        (np.zeros((2, 4)), 2, 1.0, np.zeros((2, 1)), None),
        (np.zeros((2, 4)), 2, 1.0, None, np.zeros((1, 2))),
        (np.array([[0.0, -1.0, 0.0, 0.0]]), 1, 1.0, None, None),
        (np.array([[0.0, math.nan, 0.0, 0.0]]), 1, 1.0, None, None),
        (np.zeros((1, 4)), 1, math.inf, None, None),
        (np.zeros((1, 4)), 1, 1.0, np.array([[math.nan]]), None),
        (np.zeros((1, 4)), 1, 1.0, None, np.array([[-1.0]])),
    ],
)
def test_batched_trajectory_rejects_invalid_inputs(
    initial: np.ndarray,
    steps: int,
    elapsed: float,
    nutrient_inputs: np.ndarray | None,
    harvests: np.ndarray | None,
) -> None:
    with pytest.raises(ValueError):
        simulate_food_web(initial, steps, elapsed, nutrient_inputs, harvests)
