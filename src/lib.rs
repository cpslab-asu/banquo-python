mod metric;
mod operators;
mod stl;
mod traces;

#[pyo3::pymodule]
mod _banquo_impl {
    #[pymodule_export]
    use crate::metric::{PyBottom, PyTop};

    #[pymodule_export]
    use crate::traces::PyTrace;

    #[pymodule_export]
    use crate::operators::{
        PyAlways, PyAnd, PyEventually, PyImplies, PyNext, PyNot, PyOr, PyPredicate,
    };

    #[pyo3::pymodule]
    mod stl {
        #[pymodule_export]
        use crate::stl::PyFormula;

        #[pymodule_export]
        use crate::stl::parse;
    }
}
