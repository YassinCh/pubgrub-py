//! Package identifier type for pubgrub.

use std::borrow::Borrow;
use std::fmt::Display;
use std::sync::Arc;

/// Package identifier wrapper for pubgrub.
#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct Package(pub Arc<str>);

impl Display for Package {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl Borrow<str> for Package {
    fn borrow(&self) -> &str {
        &self.0
    }
}

impl From<&str> for Package {
    fn from(s: &str) -> Self {
        Package(Arc::from(s))
    }
}

impl From<String> for Package {
    fn from(s: String) -> Self {
        Package(Arc::from(s.as_str()))
    }
}
