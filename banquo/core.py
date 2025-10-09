from __future__ import annotations

from typing import Protocol, TypeVar

from typing_extensions import Self, override

from ._banquo_impl import Trace as _Trace
from .trace import Trace

S = TypeVar("S", contravariant=True)
M = TypeVar("M", covariant=True)


class Formula(Protocol[S, M]):
    """A Formula evaluates a `Trace` of system states into a `Trace` of metric values.

    This protocol accepts two type parameters, S and M. S represents the type of the
    state values this implementation can evaluate, and M represents the type of the
    metric this implementation produces.

    In general, there are two kinds of implementations for this interface. The first
    we refer to as *expressions*, which evaluate the trace directly into metric values.
    The second are *operators*, which generally delegate evaluation to one or more
    sub-formulas and then combine the resultant traces from each subformula into a single
    trace.
    """

    def evaluate(self, trace: Trace[S]) -> _Trace[M]:
        """Evaluate the system states into metric values.

        The trace returned by this method should contain the same times as the input trace.

        Args:
            trace: The trace of system states

        Returns:
            A trace of metric values computed from each system state
        """
        ...


class SupportsNeg(Protocol):
    def __neg__(self) -> Self: ...


class SupportsLE(Protocol):
    def __le__(self, value: Self, /) -> bool: ...


class SupportsGE(Protocol):
    def __ge__(self, value: Self, /) -> bool: ...


class SupportsNegGE(SupportsNeg, SupportsGE, Protocol): ...


class EnsureInput(Formula[S, M]):
    """Wrapper to convert the input trace to a `Trace` instance if it is not already."""

    def __init__(self, inner: Formula[S, M]):
        self.inner: Formula[S, M] = inner

    @override
    def evaluate(self, trace: _Trace[S]) -> _Trace[M]:
        return self.inner.evaluate(trace if isinstance(trace, Trace) else Trace(trace))


class EnsureOutput(Formula[S, M]):
    """Wrapper to convert traces returned from rust-implemented operators into Trace values."""

    def __init__(self, inner: Formula[S, M]):
        self.inner: Formula[S, M] = inner

    @override
    def evaluate(self, trace: Trace[S]) -> Trace[M]:
        result: _Trace[M] = self.inner.evaluate(trace)
        return result if isinstance(result, Trace) else Trace(result)


def evaluate(formula: Formula[S, M], trace: Trace[S]) -> M:
    """Evaluate a trace into a metric.

    In general, when we evaluate formulas we only care about the metric value at the earliest time
    in the trace.

    Args:
        formula: The formula to use for evaluation
        trace: The trace of system states

    Returns:
        The first value in the metric trace returned by the formula.

    Raises:
        ValueError: If the provided formula evaluates the given input trace to an empty metric trace.
    """

    result = formula.evaluate(trace)

    try:
        return next(iter(result.states()))
    except StopIteration:
        raise ValueError("Provided formula evaluated to an empty trace.")
