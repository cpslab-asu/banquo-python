from __future__ import annotations

from .core import EnsureOutput
from . import _banquo_impl as _impl


class Formula(EnsureOutput[dict[str, float], float]):
    def __init__(self, formula: _impl.stl.Formula):
        super().__init__(formula)


def parse(phi: str) -> Formula:
    return Formula(_impl.stl.parse(phi))
