//! The main Resolver class exposed to Python.

use pubgrub::{DefaultStringReporter, OfflineDependencyProvider, PubGrubError, Ranges, Reporter};
use pyo3::exceptions::PyValueError;
use pyo3::prelude::*;
use semver::Version;
use std::collections::HashMap;

use crate::constraint::{parse_constraint, parse_version};
use crate::package::Package;
use crate::ResolutionError;

#[pyclass]
pub struct Resolver {
    provider: OfflineDependencyProvider<Package, Ranges<Version>>,
}

#[pymethods]
impl Resolver {
    #[new]
    pub fn new() -> Self {
        Resolver {
            provider: OfflineDependencyProvider::new(),
        }
    }

    /// Add a package with its version and dependencies.
    ///
    /// Args:
    ///     name: Package identifier
    ///     version: Version string (semver format: X.Y.Z)
    ///     dependencies: Dict of {package: constraint} e.g. {"foo": ">=1.0.0,<2.0.0"}
    ///
    /// Raises:
    ///     ValueError: If version or constraint format is invalid
    #[pyo3(signature = (name, version, dependencies=None))]
    pub fn add_package(
        &mut self,
        name: String,
        version: String,
        dependencies: Option<HashMap<String, String>>,
    ) -> PyResult<()> {
        let pkg = Package::from(name);
        let ver = parse_version(&version).map_err(|e| PyValueError::new_err(e.to_string()))?;

        let deps: Vec<(Package, Ranges<Version>)> = match dependencies {
            Some(deps_map) => {
                let mut result = Vec::with_capacity(deps_map.len());
                for (dep_name, constraint) in deps_map {
                    let range = parse_constraint(&constraint)
                        .map_err(|e| PyValueError::new_err(e.to_string()))?;
                    result.push((Package::from(dep_name), range));
                }
                result
            }
            None => Vec::new(),
        };

        self.provider.add_dependencies(pkg, ver, deps);
        Ok(())
    }

    /// Resolve dependencies starting from root packages.
    ///
    /// Args:
    ///     requirements: Dict of {package: constraint} for root dependencies
    ///
    /// Returns:
    ///     Dict of {package: version} for all resolved packages
    ///
    /// Raises:
    ///     ResolutionError: If no valid solution exists (with explanation)
    pub fn resolve(
        &self,
        requirements: HashMap<String, String>,
    ) -> PyResult<HashMap<String, String>> {
        let root = Package::from("__root__");
        let root_version = Version::new(0, 0, 0);

        // Build the root dependencies
        let mut root_deps: Vec<(Package, Ranges<Version>)> = Vec::with_capacity(requirements.len());
        for (pkg_name, constraint) in &requirements {
            let range =
                parse_constraint(constraint).map_err(|e| PyValueError::new_err(e.to_string()))?;
            root_deps.push((Package::from(pkg_name.as_str()), range));
        }

        // Create a new provider with the root package
        let mut provider = self.provider.clone();
        provider.add_dependencies(root.clone(), root_version.clone(), root_deps);

        // Resolve
        match pubgrub::resolve(&provider, root.clone(), root_version) {
            Ok(solution) => {
                let mut result = HashMap::new();
                for (pkg, ver) in solution {
                    let pkg_name = pkg.0.to_string();
                    if pkg_name != "__root__" {
                        result.insert(pkg_name, ver.to_string());
                    }
                }
                Ok(result)
            }
            Err(PubGrubError::NoSolution(derivation)) => {
                let explanation = DefaultStringReporter::report(&derivation);
                Err(ResolutionError::new_err(format!(
                    "No solution found.\n\n{}",
                    explanation
                )))
            }
            Err(e) => Err(PyValueError::new_err(format!("Resolution error: {}", e))),
        }
    }
}

impl Default for Resolver {
    fn default() -> Self {
        Self::new()
    }
}
