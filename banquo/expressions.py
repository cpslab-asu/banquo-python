from __future__ import annotations

from collections.abc import Mapping

from ._banquo_impl import Predicate as _Predicate
from .operators import OperatorMixin
from .core import EnsureOutput


class Predicate(EnsureOutput[dict[str, float], float], OperatorMixin):
    """A temporal logic expression of the form ax <= b.

    In the interest of clarity, and to reduce the overhead of needing to ensure the variables
    in each state of the trace are in the correct order, we use dictionaries to represent
    both the system state and the *a* vector. Checking for membership is done on the coefficient
    side, which means that the state may contain an arbitrary number of additional variables so
    long as it contains all of the variables for which a coefficient has been defined.

    Args:
        coefficients: The a vector provided as a dictionary of variables and their coefficients
        constant: The b value on the right side of the inequality
    """

    def __init__(self, coefficients: Mapping[str, float], constant: float):
        super().__init__(_Predicate(dict(coefficients), constant))
