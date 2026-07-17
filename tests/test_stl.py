import pytest

from banquo import Predicate, stl
from banquo import Trace as _Trace
from banquo import operators as ops

def test_parse() -> None:
    formula = stl.parse("always x <= 10.0")
    assert isinstance(formula, stl.Formula)

    formula = stl.parse("not (eventually{0.0,3.5} (not (-1.0 * x <= 2.0 and 1.0 * x <= 2.0)))")
    assert isinstance(formula, stl.Formula)

    with pytest.raises(ValueError):
        _ = stl.parse("this is not a valid formula")


Trace = _Trace[dict[str, float]]


@pytest.fixture
def trace() -> Trace:
    entries = [
        (0.0000, 0.0000),
        (0.3947, 0.5881),
        (0.7587, 1.1068),
        (1.0660, 1.4967),
        (1.2998, 1.7169),
        (1.4546, 1.7508),
        (1.5377, 1.6075),
        (1.5675, 1.3204),
        (1.5708, 0.9412),
        (1.5787, 0.5313),
        (1.6216, 0.1525),
        (1.7242, -0.1431),
        (1.9019, -0.3207),
        (2.1583, -0.368),
        (2.4844, -0.2963),
        (2.8603, -0.1383),
        (3.2583, 0.0582),
        (3.6471, 0.2386),
        (3.9968, 0.3511),
        (4.2840, 0.3561),
        (4.4947, 0.2326),
        (4.6273, -0.017),
        (4.6925, -0.3667),
        (4.7114, -0.7708),
        (4.7128, -1.1705),
        (4.7280, -1.5029),
        (4.7861, -1.7113),
        (4.9095, -1.7537),
        (5.1104, -1.6104),
        (5.3886, -1.2874),
        (5.7317, -0.816),
    ]

    return Trace({time: {"x": value} for time, value in entries})


@pytest.fixture
def p1() -> Predicate:
    return Predicate({"x": -1.0}, 2.0)


@pytest.fixture
def p2() -> Predicate:
    return Predicate({"x": 1.0}, 2.0)


def test_predicate(p1: Predicate, trace: Trace):
    f = stl.parse("-1.0 * x <= 2.0")
    assert p1.evaluate(trace) == f.evaluate(trace)


def test_negation(trace: Trace, p1: Predicate):
    formula = ops.Not(p1)
    parsed = stl.parse("not -1.0 * x <= 2.0")

    assert formula.evaluate(trace) == parsed.evaluate(trace)


def test_conjunction(trace: Trace, p1: Predicate, p2: Predicate):
    formula = ops.And(p1, p2)
    parsed = stl.parse("-1.0 * x <= 2.0 and 1.0 * x <= 2.0")

    assert formula.evaluate(trace) == parsed.evaluate(trace)


def test_disjunction(trace: Trace, p1: Predicate, p2: Predicate):
    formula = ops.Or(p1, p2)
    parsed = stl.parse("-1.0 * x <= 2.0 or 1.0 * x <= 2.0")

    assert formula.evaluate(trace) == parsed.evaluate(trace)


def test_implication(trace: Trace, p1: Predicate, p2: Predicate):
    formula = ops.Implies(p1, p2)
    parsed = stl.parse("-1.0 * x <= 2.0 implies 1.0 * x <= 2.0")

    assert formula.evaluate(trace) == parsed.evaluate(trace)


def test_eventually(trace: Trace, p1: Predicate):
    formula = ops.Eventually(p1)
    parsed = stl.parse("eventually -1.0 * x <= 2.0")

    assert formula.evaluate(trace) == parsed.evaluate(trace)

    formula = ops.Eventually.with_bounds((0.0, 3.5), p1)
    parsed = stl.parse("eventually{0.0,3.5} -1.0 * x <= 2.0")


def test_always(trace: Trace, p1: Predicate):
    formula = ops.Always(p1)
    parsed = stl.parse("always -1.0 * x <= 2.0")

    assert formula.evaluate(trace) == parsed.evaluate(trace)

    formula = ops.Always.with_bounds((0.0, 3.5), p1)
    parsed = stl.parse("always{0.0,3.5} -1.0 * x <= 2.0")


def test_composition(trace: Trace, p1: Predicate, p2: Predicate):
    formula = ops.Not(ops.Eventually.with_bounds((0.0, 3.5), ops.Not(ops.And(p2, p1))))
    parsed = stl.parse("not (eventually{0.0,3.5} (not (1.0 * x <= 2.0 and -1.0 * x <= 2.0)))")

    assert formula.evaluate(trace) == parsed.evaluate(trace)
