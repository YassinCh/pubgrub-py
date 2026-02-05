//! Python bindings for the PubGrub version resolution algorithm.
//!
//! This module provides a Python-accessible resolver that uses the pubgrub-rs
//! library for dependency resolution with semver version constraints.

use pyo3::prelude::*;

mod constraint;
mod error;
mod package;
mod resolver;

pub use constraint::{parse_constraint, parse_version};
pub use error::ResolverError;
pub use package::Package;
pub use resolver::Resolver;

// Python exception for resolution errors.
pyo3::create_exception!(pubgrub_py, ResolutionError, pyo3::exceptions::PyException);

/// Python module definition.
#[pymodule]
fn _core(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<Resolver>()?;
    m.add("ResolutionError", m.py().get_type::<ResolutionError>())?;
    Ok(())
}
