from collections.abc import Callable

from hypothesis import given
from hypothesis import settings
from hypothesis import strategies as st
from hypothesis.stateful import (
    RuleBasedStateMachine,
    initialize,
    invariant,
    rule,
    run_state_machine_as_test,  # pyright: ignore[reportUnknownVariableType]
)

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


LARGE_NONNEGATIVE = st.integers(min_value=0, max_value=10**60)
COMPARTMENTS = st.sampled_from(("left", "right"))


def snapshot(world: World) -> tuple[int, int, int, int, int, int, int, int, bool]:
    report = world.report()
    return (
        world.left,
        world.right,
        world.net_external,
        world.history_length,
        report.initial_stock,
        report.inputs,
        report.outputs,
        report.final_stock,
        report.balanced,
    )


def expect_value_error(operation: Callable[[], object]) -> None:
    try:
        operation()
    except ValueError:
        return
    raise AssertionError("invalid event was accepted")


class EnergyLedgerStateMachine(RuleBasedStateMachine):
    @initialize(left=LARGE_NONNEGATIVE, right=LARGE_NONNEGATIVE)
    def start(self, left: int, right: int) -> None:
        self.world = World(left=left, right=right)
        self.left = left
        self.right = right
        self.initial = left + right
        self.inputs = 0
        self.outputs = 0
        self.accepted_events = 0

    @rule(compartment=COMPARTMENTS, amount=LARGE_NONNEGATIVE)
    def input(self, compartment: str, amount: int) -> None:
        self.world.input(compartment, amount)
        if compartment == "left":
            self.left += amount
        else:
            self.right += amount
        self.inputs += amount
        self.accepted_events += 1

    @rule(data=st.data(), source=COMPARTMENTS)
    def valid_output(self, data: st.DataObject, source: str) -> None:
        available = self.left if source == "left" else self.right
        amount = data.draw(st.integers(min_value=0, max_value=available))
        self.world.output(source, amount)
        if source == "left":
            self.left -= amount
        else:
            self.right -= amount
        self.outputs += amount
        self.accepted_events += 1

    @rule(data=st.data(), source=COMPARTMENTS)
    def valid_transfer(self, data: st.DataObject, source: str) -> None:
        target = "right" if source == "left" else "left"
        available = self.left if source == "left" else self.right
        amount = data.draw(st.integers(min_value=0, max_value=available))
        self.world.transfer(source, target, amount)
        if source == "left":
            self.left -= amount
            self.right += amount
        else:
            self.right -= amount
            self.left += amount
        self.accepted_events += 1

    @rule(source=COMPARTMENTS, excess=st.integers(min_value=1, max_value=10**30))
    def rejected_overdraw_is_atomic(self, source: str, excess: int) -> None:
        available = self.left if source == "left" else self.right
        before = snapshot(self.world)
        expect_value_error(lambda: self.world.output(source, available + excess))
        assert snapshot(self.world) == before

    @rule(compartment=COMPARTMENTS, magnitude=st.integers(min_value=1, max_value=10**30))
    def rejected_negative_input_is_atomic(self, compartment: str, magnitude: int) -> None:
        before = snapshot(self.world)
        expect_value_error(lambda: self.world.input(compartment, -magnitude))
        assert snapshot(self.world) == before

    @rule(source=COMPARTMENTS, amount=LARGE_NONNEGATIVE)
    def rejected_self_transfer_is_atomic(self, source: str, amount: int) -> None:
        before = snapshot(self.world)
        expect_value_error(lambda: self.world.transfer(source, source, amount))
        assert snapshot(self.world) == before

    @invariant()
    def exact_oracle_and_derived_law_agree(self) -> None:
        report = self.world.report()
        assert (self.world.left, self.world.right) == (self.left, self.right)
        assert self.world.net_external == self.inputs - self.outputs
        assert self.world.history_length == self.accepted_events + 1
        assert report.initial_stock == self.initial
        assert report.inputs == self.inputs
        assert report.outputs == self.outputs
        assert report.final_stock == self.left + self.right
        assert report.residual == 0
        assert report.balanced
        if self.accepted_events == 0:
            expect_value_error(self.world.conserves_total_energy)
        else:
            assert self.world.conserves_total_energy()


def test_energy_ledger_state_machine() -> None:
    run_state_machine_as_test(
        EnergyLedgerStateMachine,
        settings=settings(max_examples=50, stateful_step_count=40),
    )


@given(
    initial=st.integers(min_value=0, max_value=10**80),
    first=LARGE_NONNEGATIVE,
    second=LARGE_NONNEGATIVE,
)
def test_splitting_and_merging_boundary_inputs_is_observationally_equivalent(
    initial: int,
    first: int,
    second: int,
) -> None:
    merged = World(left=initial, right=0)
    merged.input("left", first + second)
    split = World(left=initial, right=0)
    split.input("left", first)
    split.input("left", second)

    assert (merged.left, merged.right, merged.net_external) == (
        split.left,
        split.right,
        split.net_external,
    )
    assert snapshot(merged)[4:9] == snapshot(split)[4:9]
    assert merged.conserves_total_energy()
    assert split.conserves_total_energy()


@given(first=LARGE_NONNEGATIVE, second=LARGE_NONNEGATIVE)
def test_splitting_and_merging_internal_transfers_is_observationally_equivalent(
    first: int,
    second: int,
) -> None:
    total = first + second
    merged = World(left=total, right=0)
    merged.transfer("left", "right", total)
    split = World(left=total, right=0)
    split.transfer("left", "right", first)
    split.transfer("left", "right", second)

    assert (merged.left, merged.right, merged.net_external) == (
        split.left,
        split.right,
        split.net_external,
    )
    assert snapshot(merged)[4:9] == snapshot(split)[4:9]
    assert merged.conserves_total_energy()
    assert split.conserves_total_energy()
