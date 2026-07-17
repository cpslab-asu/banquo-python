use std::collections::HashMap;

use banquo::Formula;
use banquo::trace::Trace;
use pyo3::PyResult;
use pyo3::exceptions::PyValueError;
use pyo3::prelude::*;

use crate::metric::PyMetric;
use crate::traces::{PyMetricTrace, PyTrace};

#[pyclass(name = "Formula")]
pub struct PyFormula(banquo::stl::Formula);

impl PyFormula {
    pub fn evaluate_inner(
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
impl PyFormula {
    fn evaluate(&self, trace: &Bound<'_, PyTrace>) -> PyResult<PyMetricTrace> {
        self.evaluate_inner(trace.py(), trace.borrow().as_ref())
            .map(PyMetricTrace::from)
    }
}

#[pyfunction]
pub fn parse(phi: &str) -> PyResult<PyFormula> {
    banquo::stl::parse(phi)
        .map(PyFormula)
        .map_err(|e| PyValueError::new_err(format!("Error parsing formula: {:?}", e)))
}
