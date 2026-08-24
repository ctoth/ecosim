"""Exact ecosystem simulation with institutional conservation checks."""

from bridgman import Dimensions, canonicalize_dims

from ecosim._core import BalanceReport, World, energy_law_coefficients

ENERGY_DIMENSIONS: Dimensions = canonicalize_dims({"M": 1, "L": 2, "T": -2})

__all__ = [
    "BalanceReport",
    "ENERGY_DIMENSIONS",
    "World",
    "energy_law_coefficients",
]
