use banquo::Trace;
use pyo3::exceptions::PyKeyError;
use pyo3::prelude::*;
use pyo3::types::PyDict;

use crate::metric::PyMetric;

#[pyclass(name = "Trace", subclass, generic)]
pub struct PyTrace(Trace<Py<PyAny>>);

impl From<Trace<Py<PyAny>>> for PyTrace {
    fn from(value: Trace<Py<PyAny>>) -> Self {
        Self(value)
    }
}

impl<'py> FromPyObject<'py> for PyTrace {
    fn extract_bound(obj: &Bound<'py, PyAny>) -> PyResult<Self> {
        Self::new(obj)
    }
}

impl AsRef<Trace<Py<PyAny>>> for PyTrace {
    fn as_ref(&self) -> &Trace<Py<PyAny>> {
        &self.0
    }
}

#[pymethods]
impl PyTrace {
    #[new]
    fn new(elements: &Bound<'_, PyAny>) -> PyResult<Self> {
        // If we construct a pytrace from a pytrace, we can copy without converting to python objects
        if let Ok(pytrace) = elements.cast::<PyTrace>() {
            let py = elements.py();
            let copied = pytrace
                .borrow()
                .0
                .iter()
                .map_states(|obj| obj.clone_ref(py))
                .collect();
            return Ok(PyTrace(copied));
        }

        elements
            .cast::<PyDict>()?
            .iter()
            .map(|(key, value)| key.extract::<f64>().map(|time| (time, value.unbind())))
            .collect::<PyResult<Trace<_>>>()
            .map(|trace| Self(trace))
    }

    fn __getitem__(&self, py: Python<'_>, time: f64) -> PyResult<Py<PyAny>> {
        self.at_time(py, time)
            .ok_or_else(|| PyKeyError::new_err(format!("Time {} is not present in trace", time)))
    }

    fn times(&self) -> Vec<f64> {
        self.0.times().collect()
    }

    fn states(&self, py: Python<'_>) -> Vec<Py<PyAny>> {
        self.0.states().map(|state| state.clone_ref(py)).collect()
    }

    fn at_time(&self, py: Python<'_>, time: f64) -> Option<Py<PyAny>> {
        self.0.at_time(time).map(|state| state.clone_ref(py))
    }
}

pub struct PyMetricTrace(Trace<PyMetric>);

impl PyMetricTrace {
    pub fn into_inner(self) -> Trace<PyMetric> {
        self.0
    }

    /// After evaluation, a PyMetricTrace is a trace of Result values that may or may not
    /// contain an error. This function iterates over the states, collapsing the result values
    /// into a single outer result value and resetting the inner values to successes.
    pub fn invert(self) -> PyResult<Self> {
        let inverted: Trace<PyMetric> = self
            .0
            .into_iter()
            .map(|(time, metric)| {
                metric
                    .into_inner()
                    .map(|value| (time, PyMetric::from(value)))
            })
            .collect::<PyResult<_>>()?;

        Ok(Self(inverted))
    }
}

impl From<PyTrace> for PyMetricTrace {
    fn from(value: PyTrace) -> Self {
        Self(value.0.into_iter().map_states(PyMetric::from).collect())
    }
}

impl From<Trace<PyMetric>> for PyMetricTrace {
    fn from(value: Trace<PyMetric>) -> Self {
        Self(value)
    }
}

impl<'py> IntoPyObject<'py> for PyMetricTrace {
    type Target = PyTrace;
    type Output = Bound<'py, Self::Target>;
    type Error = PyErr;

    fn into_pyobject(self, py: Python<'py>) -> Result<Self::Output, Self::Error> {
        let trace = self
            .0
            .into_iter()
            .map(|(time, state)| state.into_inner().map(|value| (time, value)))
            .collect::<PyResult<Trace<Py<PyAny>>>>()?;

        Bound::new(py, PyTrace(trace))
    }
}
