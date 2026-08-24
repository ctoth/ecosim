from hypothesis import given
from hypothesis import strategies as st

from ecosim import ENERGY_DIMENSIONS, World, energy_law_coefficients


def test_energy_dimension_comes_from_bridgman() -> None:
    assert ENERGY_DIMENSIONS == {"M": 1, "L": 2, "T": -2}


def test_derived_law_and_open_ledger_agree() -> None:
    world = World(left=10, right=5)
    world.input("left", 7)
    world.transfer("left", "right", 4)
    world.output("right", 3)

    report = world.report()
    assert energy_law_coefficients() == {
        "left": 1,
        "net_external": -1,
        "right": 1,
    }
    assert report.initial_stock + report.inputs - report.outputs == (
        report.final_stock + report.residual
    )
    assert report.residual == 0
    assert report.balanced
    assert world.conserves_total_energy()

def test_invalid_events_are_rejected_without_mutating_the_world() -> None:
    world = World(left=2, right=3)

    for operation in (
        lambda: world.transfer("left", "left", 1),
        lambda: world.transfer("left", "right", 3),
        lambda: world.output("right", 4),
        lambda: world.input("left", -1),
        lambda: world.output("bog", 1),
    ):
        try:
            operation()
        except ValueError:
            pass
        else:
            raise AssertionError("invalid event was accepted")

    assert (world.left, world.right, world.net_external) == (2, 3, 0)
    assert world.history_length == 1


@given(st.lists(st.tuples(st.integers(0, 5), st.integers(0, 10_000)), min_size=1, max_size=64))
def test_arbitrary_valid_event_sequences_preserve_total_energy(
    events: list[tuple[int, int]],
) -> None:
    world = World(left=11, right=13)

    for event, amount in events:
        if event == 0:
            world.input("left", amount)
        elif event == 1:
            world.input("right", amount)
        elif event == 2:
            world.input("left", amount)
            world.transfer("left", "right", amount)
        elif event == 3:
            world.input("right", amount)
            world.transfer("right", "left", amount)
        elif event == 4:
            world.input("left", amount)
            world.output("left", amount)
        else:
            world.input("right", amount)
            world.output("right", amount)

    report = world.report()
    assert report.initial_stock + report.inputs - report.outputs == report.final_stock
    assert report.residual == 0
    assert world.conserves_total_energy()
