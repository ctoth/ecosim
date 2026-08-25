"""Exact ecosystem simulation with institutional conservation checks."""

from bridgman import Dimensions, canonicalize_dims

from ecosim._core import (
    BalanceReport,
    DenseFoodWeb,
    DenseFoodWebStep,
    FoodWeb,
    FoodWebStep,
    World,
    energy_law_coefficients,
    simulate_food_web,
)

ENERGY_DIMENSIONS: Dimensions = canonicalize_dims({"M": 1, "L": 2, "T": -2})

__all__ = [
    "BalanceReport",
    "DenseFoodWeb",
    "DenseFoodWebStep",
    "ENERGY_DIMENSIONS",
    "FoodWeb",
    "FoodWebStep",
    "World",
    "energy_law_coefficients",
    "simulate_food_web",
]
