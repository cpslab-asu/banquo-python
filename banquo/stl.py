from __future__ import annotations

from ._banquo_impl import stl as _stl
from .core import EnsureOutput


class Formula(EnsureOutput[dict[str, float], float]):
    def __init__(self, formula: _stl.Formula):
        super().__init__(formula)


def parse(phi: str) -> Formula:
    return Formula(_stl.parse(phi))
