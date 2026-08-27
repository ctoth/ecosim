"""Exact ecosystem simulation with institutional conservation checks."""

from bridgman import Dimensions, canonicalize_dims

from ecosim._core import (
    BalanceReport,
    DenseFoodWeb,
    DenseFoodWebStep,
    FoodWeb,
    FoodWebParameters,
    FoodWebStep,
    World,
    energy_law_coefficients,
    simulate_food_web,
)
from ecosim.experiments import (
    BoundaryPerturbation,
    FoodWebResponse,
    simulate_food_web_response,
)

ENERGY_DIMENSIONS: Dimensions = canonicalize_dims({"M": 1, "L": 2, "T": -2})

__all__ = [
    "BalanceReport",
    "BoundaryPerturbation",
    "DenseFoodWeb",
    "DenseFoodWebStep",
    "ENERGY_DIMENSIONS",
    "FoodWeb",
    "FoodWebParameters",
    "FoodWebResponse",
    "FoodWebStep",
    "World",
    "energy_law_coefficients",
    "simulate_food_web",
    "simulate_food_web_response",
]
