use std::cmp::Ordering;
use std::ops::Neg;

use pyo3::PyErr;
use pyo3::basic::CompareOp;
use pyo3::types::{PyAny, PyAnyMethods, PyBool, PyModule, PyNotImplemented};
use pyo3::{Bound, IntoPyObject, IntoPyObjectExt, Py, PyResult, Python, pyclass, pymethods};

use banquo::{Bottom, Join, Meet, Top};

// This class is a special value that represents the maximum of ALL python values.
// Thus, this the greater-than implementation for this class will always return true.
#[pyclass(name = "Top")]
pub struct PyTop;

#[pymethods]
impl PyTop {
    #[new]
    fn new() -> Self {
        Self
    }

    fn __richcmp__<'py>(&self, other: &Bound<'py, PyAny>, op: CompareOp) -> Bound<'py, PyAny> {
        let py = other.py();
        let result = match op {
            CompareOp::Lt | CompareOp::Le => Some(false),
            CompareOp::Eq | CompareOp::Ne => other
                .cast::<Self>()
                .map(|_| if let CompareOp::Eq = op { true } else { false })
                .ok(),
            CompareOp::Ge | CompareOp::Gt => Some(true),
        };

        result
            .map(|v| PyBool::new(py, v).to_owned().into_any())
            .unwrap_or_else(|| PyNotImplemented::get(py).to_owned().into_any())
    }
}

// This class is a special value that represents the minimum of ALL python values.
// Thus, this the less-than implementation for this class will always return true.
#[pyclass(name = "Bottom")]
pub struct PyBottom;

#[pymethods]
impl PyBottom {
    #[new]
    fn new() -> Self {
        Self
    }

    fn __richcmp__<'py>(&self, other: &Bound<'py, PyAny>, op: CompareOp) -> Bound<'py, PyAny> {
        let py = other.py();
        let result = match op {
            CompareOp::Lt | CompareOp::Le => Some(true),
            CompareOp::Eq | CompareOp::Ne => other
                .cast::<Self>()
                .map(|_| if let CompareOp::Eq = op { true } else { false }) // Op can only be Eq or Ne in this branch
                .ok(),
            CompareOp::Ge | CompareOp::Gt => Some(false),
        };

        result
            .map(|v| PyBool::new(py, v).to_owned().into_any())
            .unwrap_or_else(|| PyNotImplemented::get(py).to_owned().into_any())
    }
}

type PyMetricInner = PyResult<Py<PyAny>>;

pub struct PyMetric(PyMetricInner);

impl PyMetric {
    pub fn from_f64(value: f64) -> Self {
        Self(Python::attach(|py| value.into_py_any(py)))
    }

    pub fn into_inner(self) -> PyMetricInner {
        self.0
    }
}

impl From<Py<PyAny>> for PyMetric {
    fn from(value: Py<PyAny>) -> Self {
        Self(Ok(value))
    }
}

impl<'py> IntoPyObject<'py> for PyMetric {
    type Target = PyAny;
    type Output = Bound<'py, Self::Target>;
    type Error = PyErr;

    fn into_pyobject(self, py: Python<'py>) -> Result<Self::Output, Self::Error> {
        self.0.map(|value| value.into_bound(py))
    }
}

fn transpose_results<A, B, E>(lhs: Result<A, E>, rhs: Result<B, E>) -> Result<(A, B), E> {
    Ok((lhs?, rhs?))
}

impl PartialEq for PyMetric {
    fn eq(&self, other: &Self) -> bool {
        // Two metrics are equal if their inner values are equal and not errors
        Python::attach(|py| -> bool {
            transpose_results(self.0.as_ref(), other.0.as_ref())
                .map_err(|e| e.clone_ref(py))
                .and_then(|(lhs, rhs)| lhs.bind(py).eq(rhs))
                .unwrap_or(false)
        })
    }
}

impl PartialOrd for PyMetric {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        // Create an ordering for the two metrics only if both are not errors
        // If the compare method creates an error, transform it to a None value
        Python::attach(|py| -> Option<Ordering> {
            transpose_results(self.0.as_ref(), other.0.as_ref())
                .map_err(|e| e.clone_ref(py))
                .and_then(|(lhs, rhs)| lhs.bind(py).compare(rhs))
                .ok()
        })
    }
}

impl Neg for PyMetric {
    type Output = Self;

    fn neg(self) -> Self::Output {
        let negated = Python::attach(|py| {
            self.0
                .and_then(|value| value.bind(py).neg())
                .map(|value| value.unbind())
        });

        Self(negated)
    }
}

fn builtin(py: Python<'_>, lhs: &PyMetricInner, rhs: &PyMetricInner, name: &str) -> PyMetricInner {
    let (lval, rval) =
        transpose_results(lhs.as_ref(), rhs.as_ref()).map_err(|e| e.clone_ref(py))?;

    PyModule::import(py, "builtins")?
        .getattr(name)?
        .call((lval, rval), None)
        .map(|value| value.unbind())
}

impl Meet for PyMetric {
    fn min(&self, other: &Self) -> Self {
        Self(Python::attach(|py| builtin(py, &self.0, &other.0, "min")))
    }
}

impl Join for PyMetric {
    fn max(&self, other: &Self) -> Self {
        Self(Python::attach(|py| builtin(py, &self.0, &other.0, "max")))
    }
}

impl Top for PyMetric {
    fn top() -> Self {
        Python::attach(|py| Self(PyTop.into_py_any(py)))
    }
}

impl Bottom for PyMetric {
    fn bottom() -> Self {
        Python::attach(|py| Self(PyBottom.into_py_any(py)))
    }
}
