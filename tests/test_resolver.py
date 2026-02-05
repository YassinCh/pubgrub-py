"""Tests for the pubgrub-py resolver."""

import pytest
from pubgrub_py import ResolutionError, Resolver, resolve


class TestResolver:
    """Tests for the resolve() convenience function."""

    def test_simple_resolution(self) -> None:
        """Test resolving a simple dependency chain."""
        result = resolve(
            requirements={"root": ">=1.0.0"},
            available={
                "root": {"1.0.0": {"dep": ">=1.0.0"}},
                "dep": {"1.0.0": {}, "1.1.0": {}},
            },
        )
        assert result["root"] == "1.0.0"
        assert result["dep"] == "1.1.0"  # Should pick highest matching version

    def test_transitive_dependencies(self) -> None:
        """Test resolving transitive dependencies."""
        result = resolve(
            requirements={"a": ">=1.0.0"},
            available={
                "a": {"1.0.0": {"b": ">=1.0.0"}},
                "b": {"1.0.0": {"c": ">=1.0.0"}, "2.0.0": {"c": ">=2.0.0"}},
                "c": {"1.0.0": {}, "2.0.0": {}},
            },
        )
        assert result["a"] == "1.0.0"
        assert result["b"] == "2.0.0"
        assert result["c"] == "2.0.0"

    def test_constraint_upper_bound(self) -> None:
        """Test that upper bounds are respected."""
        result = resolve(
            requirements={"pkg": ">=1.0.0,<2.0.0"},
            available={
                "pkg": {"1.0.0": {}, "1.5.0": {}, "2.0.0": {}, "2.1.0": {}},
            },
        )
        assert result["pkg"] == "1.5.0"

    def test_exact_version(self) -> None:
        """Test exact version matching."""
        result = resolve(
            requirements={"pkg": "==1.2.0"},
            available={
                "pkg": {"1.0.0": {}, "1.2.0": {}, "1.5.0": {}},
            },
        )
        assert result["pkg"] == "1.2.0"

    def test_compatible_release(self) -> None:
        """Test compatible release operator (~=)."""
        result = resolve(
            requirements={"pkg": "~=1.4.0"},
            available={
                "pkg": {"1.3.0": {}, "1.4.0": {}, "1.4.5": {}, "1.5.0": {}},
            },
        )
        assert result["pkg"] == "1.4.5"

    def test_no_dependencies(self) -> None:
        """Test package with no dependencies."""
        result = resolve(
            requirements={"standalone": ">=1.0.0"},
            available={
                "standalone": {"1.0.0": {}, "2.0.0": {}},
            },
        )
        assert result["standalone"] == "2.0.0"

    def test_multiple_root_requirements(self) -> None:
        """Test multiple root requirements."""
        result = resolve(
            requirements={"a": ">=1.0.0", "b": ">=2.0.0"},
            available={
                "a": {"1.0.0": {}},
                "b": {"2.0.0": {}, "2.1.0": {}},
            },
        )
        assert result["a"] == "1.0.0"
        assert result["b"] == "2.1.0"


class TestResolutionErrors:
    """Tests for resolution error handling."""

    def test_conflict_error_message(self) -> None:
        """Test that conflicts produce readable error messages."""
        with pytest.raises(ResolutionError) as exc_info:
            resolve(
                requirements={"a": ">=1.0.0", "b": ">=1.0.0"},
                available={
                    "a": {"1.0.0": {"shared": ">=2.0.0"}},
                    "b": {"1.0.0": {"shared": "<2.0.0"}},
                    "shared": {"1.0.0": {}, "2.0.0": {}},
                },
            )
        assert "shared" in str(exc_info.value)

    def test_no_matching_version(self) -> None:
        """Test error when no version matches constraint."""
        with pytest.raises(ResolutionError):
            resolve(
                requirements={"pkg": ">=5.0.0"},
                available={
                    "pkg": {"1.0.0": {}, "2.0.0": {}},
                },
            )

    def test_missing_package(self) -> None:
        """Test error when package doesn't exist."""
        with pytest.raises(ResolutionError):
            resolve(
                requirements={"nonexistent": ">=1.0.0"},
                available={},
            )

    def test_missing_transitive_dependency(self) -> None:
        """Test error when transitive dependency is missing."""
        with pytest.raises(ResolutionError):
            resolve(
                requirements={"a": ">=1.0.0"},
                available={
                    "a": {"1.0.0": {"missing": ">=1.0.0"}},
                },
            )


class TestResolverClass:
    """Tests for the Resolver class API."""

    def test_add_package_and_resolve(self) -> None:
        """Test using Resolver class directly."""
        resolver = Resolver()
        resolver.add_package("app", "1.0.0", {"lib": ">=1.0.0"})
        resolver.add_package("lib", "1.0.0", {})
        resolver.add_package("lib", "1.1.0", {})

        result = resolver.resolve({"app": ">=1.0.0"})
        assert result["app"] == "1.0.0"
        assert result["lib"] == "1.1.0"

    def test_add_package_without_dependencies(self) -> None:
        """Test adding package with no dependencies."""
        resolver = Resolver()
        resolver.add_package("standalone", "1.0.0")

        result = resolver.resolve({"standalone": ">=1.0.0"})
        assert result["standalone"] == "1.0.0"

    def test_invalid_version_format(self) -> None:
        """Test error on invalid version format."""
        resolver = Resolver()
        with pytest.raises(ValueError, match="Invalid version"):
            resolver.add_package("pkg", "not-a-version", {})

    def test_invalid_constraint_format(self) -> None:
        """Test error on invalid constraint format."""
        resolver = Resolver()
        resolver.add_package("pkg", "1.0.0", {})
        with pytest.raises(ValueError, match="Invalid"):
            resolver.resolve({"pkg": ">>invalid<<"})


class TestConstraintParsing:
    """Tests for constraint parsing and matching."""

    @pytest.mark.parametrize(
        ("constraint", "should_match", "should_not_match"),
        [
            (">=1.0.0", ["1.0.0", "1.5.0", "2.0.0"], ["0.9.0"]),
            ("<=2.0.0", ["1.0.0", "2.0.0"], ["2.0.1", "3.0.0"]),
            (">1.0.0", ["1.0.1", "2.0.0"], ["1.0.0", "0.9.0"]),
            ("<2.0.0", ["1.0.0", "1.9.9"], ["2.0.0", "3.0.0"]),
            ("==1.5.0", ["1.5.0"], ["1.4.0", "1.6.0"]),
            (">=1.0.0,<2.0.0", ["1.0.0", "1.9.0"], ["0.9.0", "2.0.0"]),
        ],
    )
    def test_constraint_matching(
        self,
        constraint: str,
        should_match: list[str],
        should_not_match: list[str],
    ) -> None:
        """Test various constraint operators match correctly."""
        # Build available versions from both lists
        available_versions = {v: {} for v in should_match + should_not_match}

        result = resolve(
            requirements={"pkg": constraint},
            available={"pkg": available_versions},
        )
        # The selected version should be one of the matching versions
        assert result.get("pkg") in should_match
