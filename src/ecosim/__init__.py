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
from ecosim.network import (
    Consumer,
    DenseTrophicNetwork,
    DenseTrophicNetworkPlan,
    DenseTrophicNetworkStep,
    Feeding,
    Producer,
    TrophicNetwork,
    TrophicNetworkPlan,
    TrophicNetworkSpec,
    TrophicNetworkStep,
)

ENERGY_DIMENSIONS: Dimensions = canonicalize_dims({"M": 1, "L": 2, "T": -2})

__all__ = [
    "BalanceReport",
    "BoundaryPerturbation",
    "Consumer",
    "DenseFoodWeb",
    "DenseFoodWebStep",
    "DenseTrophicNetwork",
    "DenseTrophicNetworkPlan",
    "DenseTrophicNetworkStep",
    "ENERGY_DIMENSIONS",
    "FoodWeb",
    "FoodWebParameters",
    "FoodWebResponse",
    "Feeding",
    "Producer",
    "TrophicNetwork",
    "TrophicNetworkPlan",
    "TrophicNetworkSpec",
    "TrophicNetworkStep",
    "FoodWebStep",
    "World",
    "energy_law_coefficients",
    "simulate_food_web",
    "simulate_food_web_response",
]
