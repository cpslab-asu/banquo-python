from __future__ import annotations

from collections.abc import Iterable, Iterator, Mapping
from typing import TypeVar

from typing_extensions import override

from ._banquo_impl import Trace as _Trace

T = TypeVar("T", covariant=True)
U = TypeVar("U", covariant=True)


def _iter_eq(lhs: Iterable[object], rhs: Iterable[object]) -> bool:
    return list(lhs) == list(rhs)


class MismatchedTimesStates(Exception):
    def __init__(self):
        super().__init__("Times and states iterables must contain equal number of elements")


class Trace(_Trace[T], Iterable[tuple[float, T]]):
    """A set of values and their associated times.

    Iterating over a trace yields (time, value) pairs.

    Args:
        elements: The set of times and their associated values
    """

    def __new__(cls, elements: Mapping[float, T] | _Trace[T]):
        return super().__new__(cls, elements if isinstance(elements, _Trace) else dict(elements))

    @override
    def __str__(self) -> str:
        return str({time: state for time, state in self})

    @override
    def __iter__(self) -> Iterator[tuple[float, T]]:
        return zip(super().times(), super().states())

    @override
    def __eq__(self, other: object) -> bool:
        if not isinstance(other, _Trace):
            return NotImplemented

        return _iter_eq(self.times(), other.times()) and _iter_eq(self.states(), other.states())

    @override
    def times(self) -> Iterable[float]:
        """Iterator over the times of the trace."""

        return iter(super().times())

    @override
    def states(self) -> Iterable[T]:
        """Iterator over the states of the trace."""

        return iter(super().states())

    @staticmethod
    def from_timed_states(times: Iterable[float], states: Iterable[U]) -> Trace[U]:
        """Construct a trace from a set of times and a set of states.

        Args:
            times: The times of the trace
            states: The state for each time

        Returns:
            A `Trace` instance from the times and states

        Raises:
            MismatchedTimesStates: If the number of time values does not match the number of state values.
        """

        times_ = list(times)
        states_ = list(states)

        if len(times_) != len(states_):
            raise MismatchedTimesStates()

        return Trace({time: state for time, state in zip(times_, states_)})
