"""High-level resolve function for PubGrub."""

from pubgrub_py._core import Resolver


def resolve(
    requirements: dict[str, str],
    available: dict[str, dict[str, dict[str, str]]],
) -> dict[str, str]:
    """Resolve dependencies using PubGrub algorithm.

    Args:
        requirements: Root requirements {package: constraint}
        available: Available packages {package: {version: {dep: constraint}}}

    Returns:
        Resolved versions {package: version}

    Raises:
        ResolutionError: With human-readable explanation if unresolvable

    Example:
        >>> resolve(
        ...     requirements={"app": ">=1.0.0"},
        ...     available={
        ...         "app": {"1.0.0": {"lib": ">=2.0.0,<3.0.0"}},
        ...         "lib": {"2.0.0": {}, "2.1.0": {}},
        ...     }
        ... )
        {"app": "1.0.0", "lib": "2.1.0"}
    """
    resolver = Resolver()
    for pkg, versions in available.items():
        for ver, deps in versions.items():
            resolver.add_package(pkg, ver, deps)
    return resolver.resolve(requirements)
