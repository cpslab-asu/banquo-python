use std::collections::HashMap;

use banquo::operators::{Always, And, Eventually, Implies, Next, Not, Or};
use banquo::operators::{BinaryOperatorError, ForwardEvaluationError, ForwardOperatorError};
use banquo::{Formula, Predicate, Trace};
use pyo3::exceptions::{PyRuntimeError, PyValueError};
use pyo3::prelude::*;

use crate::metric::PyMetric;
use crate::stl;
use crate::traces::{PyMetricTrace, PyTrace};

#[pyclass(name = "Predicate", subclass)]
pub struct PyPredicate(Predicate);

impl PyPredicate {
    fn evaluate_inner(
        &self,
        py: Python<'_>,
        trace: &Trace<Py<PyAny>>,
    ) -> PyResult<Trace<PyMetric>> {
        let converted = trace
            .iter()
            .map(|(time, state)| state.extract::<HashMap<String, f64>>(py).map(|s| (time, s)))
            .collect::<PyResult<Trace<_>>>()
            .map_err(|_| {
                PyValueError::new_err("Predicate only supports dict values as trace states.")
            })?;

        let evaluated = self
            .0
            .evaluate(&converted)
            .map_err(|err| PyValueError::new_err(err.to_string()))?
            .into_iter()
            .map(|(time, rho)| (time, PyMetric::from_f64(rho)))
            .collect();

        Ok(evaluated)
    }
}

#[pymethods]
impl PyPredicate {
    #[new]
    pub fn new(coefficients: HashMap<String, f64>, constant: f64) -> Self {
        let mut p = Predicate::from_iter(coefficients);
        p += constant;

        Self(p)
    }

    pub fn __eq__(&self, other: &Bound<'_, Self>) -> bool {
        self.0 == other.borrow().0
    }

    pub fn evaluate(&self, trace: &Bound<'_, PyTrace>) -> PyResult<PyMetricTrace> {
        self.evaluate_inner(trace.py(), trace.borrow().as_ref())
            .map(PyMetricTrace::from)
    }
}

struct PyFormula(Py<PyAny>);

impl<'py> FromPyObject<'py> for PyFormula {
    fn extract_bound(obj: &Bound<'py, PyAny>) -> PyResult<Self> {
        Ok(Self(obj.clone().unbind()))
    }
}

fn evaluate(obj: &Bound<'_, PyAny>, trace: &Trace<Py<PyAny>>) -> PyResult<Trace<PyMetric>> {
    if let Ok(pred) = obj.cast::<PyPredicate>() {
        return pred.borrow().evaluate_inner(obj.py(), trace);
    }

    if let Ok(not) = obj.cast::<PyNot>() {
        return not.borrow().evaluate_inner(trace);
    }

    if let Ok(and) = obj.cast::<PyAnd>() {
        return and.borrow().evaluate_inner(trace);
    }

    if let Ok(or) = obj.cast::<PyOr>() {
        return or.borrow().evaluate_inner(trace);
    }

    if let Ok(implies) = obj.cast::<PyImplies>() {
        return implies.borrow().evaluate_inner(trace);
    }

    if let Ok(always) = obj.cast::<PyAlways>() {
        return always.borrow().evaluate_inner(trace);
    }

    if let Ok(eventually) = obj.cast::<PyEventually>() {
        return eventually.borrow().evaluate_inner(trace);
    }

    if let Ok(formula) = obj.cast::<stl::PyFormula>() {
        return formula.borrow().evaluate_inner(obj.py(), trace);
    }

    let py = obj.py();
    let new_trace: Trace<Py<PyAny>> = trace.iter().map_states(|obj| obj.clone_ref(py)).collect();
    let pytrace = PyTrace::from(new_trace);

    obj.call_method1("evaluate", (pytrace,))
        .and_then(|result| result.extract::<PyTrace>())
        .map(|result| PyMetricTrace::from(result).into_inner())
}

impl Formula<Py<PyAny>> for PyFormula {
    type Metric = PyMetric;
    type Error = PyErr;

    fn evaluate(&self, trace: &Trace<Py<PyAny>>) -> Result<Trace<Self::Metric>, Self::Error> {
        // Apply the internal operator implementation
        let evaluated = Python::attach(|py| evaluate(self.0.bind(py), trace))?;

        // Invert results to stop evaluation if a metric value contains an error
        let inverted = PyMetricTrace::from(evaluated).invert()?;

        Ok(inverted.into_inner())
    }
}

#[pyclass(name = "Not")]
pub struct PyNot(Not<PyFormula>);

impl PyNot {
    fn evaluate_inner(&self, trace: &Trace<Py<PyAny>>) -> PyResult<Trace<PyMetric>> {
        self.0.evaluate(trace)
    }
}

#[pymethods]
impl PyNot {
    #[new]
    fn new(subformula: PyFormula) -> Self {
        Self(Not::new(subformula))
    }

    fn evaluate(&self, trace: &Bound<'_, PyTrace>) -> PyResult<PyMetricTrace> {
        self.evaluate_inner(trace.borrow().as_ref())
            .map(PyMetricTrace::from)
    }
}

#[pyclass(name = "And")]
pub struct PyAnd(And<PyFormula, PyFormula>);

impl PyAnd {
    fn evaluate_inner(&self, trace: &Trace<Py<PyAny>>) -> PyResult<Trace<PyMetric>> {
        self.0.evaluate(trace).map_err(|err| match err {
            BinaryOperatorError::LeftError(left) => left,
            BinaryOperatorError::RightError(right) => right,
            BinaryOperatorError::EvaluationError(err) => PyRuntimeError::new_err(err.to_string()),
        })
    }
}

#[pymethods]
impl PyAnd {
    #[new]
    fn new(lhs: PyFormula, rhs: PyFormula) -> Self {
        Self(And::new(lhs, rhs))
    }

    fn evaluate(&self, trace: &Bound<'_, PyTrace>) -> PyResult<PyMetricTrace> {
        self.evaluate_inner(trace.borrow().as_ref())
            .map(PyMetricTrace::from)
    }
}

#[pyclass(name = "Or")]
pub struct PyOr(Or<PyFormula, PyFormula>);

impl PyOr {
    fn evaluate_inner(&self, trace: &Trace<Py<PyAny>>) -> PyResult<Trace<PyMetric>> {
        self.0.evaluate(trace).map_err(|err| match err {
            BinaryOperatorError::LeftError(left) => left,
            BinaryOperatorError::RightError(right) => right,
            BinaryOperatorError::EvaluationError(err) => PyRuntimeError::new_err(err.to_string()),
        })
    }
}

#[pymethods]
impl PyOr {
    #[new]
    fn new(lhs: PyFormula, rhs: PyFormula) -> Self {
        Self(Or::new(lhs, rhs))
    }

    fn evaluate(&self, trace: &Bound<'_, PyTrace>) -> PyResult<PyMetricTrace> {
        self.evaluate_inner(trace.borrow().as_ref())
            .map(PyMetricTrace::from)
    }
}

#[pyclass(name = "Implies")]
pub struct PyImplies(Implies<PyFormula, PyFormula>);

impl PyImplies {
    fn evaluate_inner(&self, trace: &Trace<Py<PyAny>>) -> PyResult<Trace<PyMetric>> {
        self.0.evaluate(trace).map_err(|err| match err {
            BinaryOperatorError::LeftError(left) => left,
            BinaryOperatorError::RightError(right) => right,
            BinaryOperatorError::EvaluationError(err) => PyRuntimeError::new_err(err.to_string()),
        })
    }
}

#[pymethods]
impl PyImplies {
    #[new]
    fn new(lhs: PyFormula, rhs: PyFormula) -> Self {
        Self(Implies::new(lhs, rhs))
    }

    fn evaluate(&self, trace: &Bound<'_, PyTrace>) -> PyResult<PyMetricTrace> {
        self.evaluate_inner(trace.borrow().as_ref())
            .map(PyMetricTrace::from)
    }
}

#[pyclass(name = "Next")]
pub struct PyNext(Next<PyFormula>);

impl PyNext {
    fn evaluate_inner(&self, trace: &Trace<Py<PyAny>>) -> PyResult<Trace<PyMetric>> {
        self.0.evaluate(trace)
    }
}

#[pymethods]
impl PyNext {
    #[new]
    fn new(subformula: PyFormula) -> Self {
        Self(Next::new(subformula))
    }

    fn evaluate(&self, trace: &Bound<'_, PyTrace>) -> PyResult<PyMetricTrace> {
        self.evaluate_inner(trace.borrow().as_ref())
            .map(PyMetricTrace::from)
    }
}

#[pyclass(name = "Always")]
pub struct PyAlways(Always<PyFormula>);

impl PyAlways {
    fn evaluate_inner(&self, trace: &Trace<Py<PyAny>>) -> PyResult<Trace<PyMetric>> {
        self.0.evaluate(trace).map_err(|err| match err {
            ForwardOperatorError::EvaluationError(eval_err) => match eval_err {
                ForwardEvaluationError::EmptyInterval => {
                    PyValueError::new_err("Bounds interval must not be empty.")
                }
                ForwardEvaluationError::EmptySubtraceEvaluation(t) => {
                    PyRuntimeError::new_err(format!("Subtrace at time {} is empty.", t))
                }
            },
            ForwardOperatorError::FormulaError(err) => err,
        })
    }
}

#[pymethods]
impl PyAlways {
    #[new]
    fn new(bounds: Option<(f64, f64)>, subformula: PyFormula) -> Self {
        let inner = if let Some((lo, hi)) = bounds {
            Always::bounded(lo..=hi, subformula)
        } else {
            Always::unbounded(subformula)
        };

        Self(inner)
    }

    fn evaluate(&self, trace: &Bound<'_, PyTrace>) -> PyResult<PyMetricTrace> {
        self.evaluate_inner(trace.borrow().as_ref())
            .map(PyMetricTrace::from)
    }
}

#[pyclass(name = "Eventually")]
pub struct PyEventually(Eventually<PyFormula>);

impl PyEventually {
    fn evaluate_inner(&self, trace: &Trace<Py<PyAny>>) -> PyResult<Trace<PyMetric>> {
        self.0.evaluate(trace).map_err(|err| match err {
            ForwardOperatorError::EvaluationError(eval_err) => match eval_err {
                ForwardEvaluationError::EmptyInterval => {
                    PyValueError::new_err("Bounds interval must not be empty.")
                }
                ForwardEvaluationError::EmptySubtraceEvaluation(t) => {
                    PyRuntimeError::new_err(format!("Subtrace at time {} is empty.", t))
                }
            },
            ForwardOperatorError::FormulaError(err) => err,
        })
    }
}

#[pymethods]
impl PyEventually {
    #[new]
    fn new(bounds: Option<(f64, f64)>, subformula: PyFormula) -> Self {
        let inner = if let Some((lo, hi)) = bounds {
            Eventually::bounded(lo..=hi, subformula)
        } else {
            Eventually::unbounded(subformula)
        };

        Self(inner)
    }

    fn evaluate(&self, trace: &Bound<'_, PyTrace>) -> PyResult<PyMetricTrace> {
        self.evaluate_inner(trace.borrow().as_ref())
            .map(PyMetricTrace::from)
    }
}
