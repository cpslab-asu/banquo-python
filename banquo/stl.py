from __future__ import annotations

from .core import EnsureOutput
from ._banquo_impl import stl as _stl


class Formula(EnsureOutput[dict[str, float], float]):
    def __init__(self, formula: _stl.Formula):
        super().__init__(formula)


def parse(phi: str) -> Formula:
    return Formula(_stl.parse(phi))
