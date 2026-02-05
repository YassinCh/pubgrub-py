//! PEP 440 version constraint parsing.

use pubgrub::Ranges;
use semver::Version;

use crate::error::ResolverError;

/// Parse a semver version string.
pub fn parse_version(version: &str) -> Result<Version, ResolverError> {
    Version::parse(version).map_err(|e| ResolverError::InvalidVersion {
        version: version.to_string(),
        reason: e.to_string(),
    })
}

/// Parse a single constraint operator and version into a Range.
fn parse_single_constraint(constraint: &str) -> Result<Ranges<Version>, ResolverError> {
    let constraint = constraint.trim();

    if constraint.is_empty() || constraint == "*" {
        return Ok(Ranges::full());
    }

    // Handle compatible release operator ~=
    if let Some(ver_str) = constraint.strip_prefix("~=") {
        let ver_str = ver_str.trim();
        let version = parse_version(ver_str)?;
        let next_minor = Version::new(version.major, version.minor + 1, 0);
        return Ok(Ranges::between(version, next_minor));
    }

    // Handle comparison operators
    if let Some(ver_str) = constraint.strip_prefix(">=") {
        let version = parse_version(ver_str.trim())?;
        return Ok(Ranges::higher_than(version));
    }

    if let Some(ver_str) = constraint.strip_prefix("<=") {
        let version = parse_version(ver_str.trim())?;
        let next = Version::new(version.major, version.minor, version.patch + 1);
        return Ok(Ranges::strictly_lower_than(next));
    }

    if let Some(ver_str) = constraint.strip_prefix("!=") {
        let version = parse_version(ver_str.trim())?;
        let next = Version::new(version.major, version.minor, version.patch + 1);
        let below = Ranges::strictly_lower_than(version.clone());
        let above = Ranges::higher_than(next);
        return Ok(below.union(&above));
    }

    if let Some(ver_str) = constraint.strip_prefix("==") {
        let version = parse_version(ver_str.trim())?;
        return Ok(Ranges::singleton(version));
    }

    if let Some(ver_str) = constraint.strip_prefix('>') {
        let version = parse_version(ver_str.trim())?;
        let next = Version::new(version.major, version.minor, version.patch + 1);
        return Ok(Ranges::higher_than(next));
    }

    if let Some(ver_str) = constraint.strip_prefix('<') {
        let version = parse_version(ver_str.trim())?;
        return Ok(Ranges::strictly_lower_than(version));
    }

    // No operator means exact match
    let version = parse_version(constraint)?;
    Ok(Ranges::singleton(version))
}

/// Parse a PEP 440 constraint string into pubgrub Ranges.
///
/// Supports: >=, <=, >, <, ==, !=, ~= (compatible release)
/// Combined constraints: ">=1.0.0,<2.0.0"
pub fn parse_constraint(constraint: &str) -> Result<Ranges<Version>, ResolverError> {
    let constraint = constraint.trim();

    if constraint.is_empty() || constraint == "*" {
        return Ok(Ranges::full());
    }

    let parts: Vec<&str> = constraint.split(',').map(|s| s.trim()).collect();

    let mut result = Ranges::full();
    for part in parts {
        if part.is_empty() {
            continue;
        }
        let range = parse_single_constraint(part)?;
        result = result.intersection(&range);
    }

    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_version() {
        assert!(parse_version("1.0.0").is_ok());
        assert!(parse_version("0.1.0").is_ok());
        assert!(parse_version("10.20.30").is_ok());
        assert!(parse_version("invalid").is_err());
    }

    #[test]
    fn test_parse_constraint_operators() {
        assert!(parse_constraint(">=1.0.0").is_ok());
        assert!(parse_constraint("<=2.0.0").is_ok());
        assert!(parse_constraint(">1.0.0").is_ok());
        assert!(parse_constraint("<2.0.0").is_ok());
        assert!(parse_constraint("==1.5.0").is_ok());
        assert!(parse_constraint("!=1.3.0").is_ok());
        assert!(parse_constraint("~=1.4.0").is_ok());
    }

    #[test]
    fn test_parse_constraint_combined() {
        let range = parse_constraint(">=1.0.0,<2.0.0").unwrap();
        assert!(range.contains(&Version::new(1, 0, 0)));
        assert!(range.contains(&Version::new(1, 5, 0)));
        assert!(!range.contains(&Version::new(2, 0, 0)));
        assert!(!range.contains(&Version::new(0, 9, 0)));
    }

    #[test]
    fn test_parse_constraint_compatible_release() {
        let range = parse_constraint("~=1.4.0").unwrap();
        assert!(range.contains(&Version::new(1, 4, 0)));
        assert!(range.contains(&Version::new(1, 4, 5)));
        assert!(!range.contains(&Version::new(1, 5, 0)));
        assert!(!range.contains(&Version::new(1, 3, 0)));
    }
}
