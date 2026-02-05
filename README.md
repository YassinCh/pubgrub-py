# pubgrub-py

Python bindings for the [PubGrub](https://github.com/pubgrub-rs/pubgrub) version resolution algorithm.

PubGrub is the dependency resolution algorithm used by Dart's pub package manager, and this library provides Python access to the Rust implementation for high-performance dependency resolution.

## Installation

```bash
pip install pubgrub-py
```

Or with maturin for development:

```bash
cd packages/pubgrub-py
maturin develop
```

## Usage

### Simple API

```python
from pubgrub_py import resolve

# Define available packages with their versions and dependencies
available = {
    "app": {
        "1.0.0": {"lib-a": ">=1.0.0", "lib-b": ">=2.0.0"},
    },
    "lib-a": {
        "1.0.0": {},
        "1.1.0": {"shared": ">=1.0.0"},
    },
    "lib-b": {
        "2.0.0": {"shared": ">=1.0.0"},
        "2.1.0": {"shared": ">=1.5.0"},
    },
    "shared": {
        "1.0.0": {},
        "1.5.0": {},
        "2.0.0": {},
    },
}

# Resolve dependencies
result = resolve(
    requirements={"app": ">=1.0.0"},
    available=available,
)
# Returns: {"app": "1.0.0", "lib-a": "1.1.0", "lib-b": "2.1.0", "shared": "2.0.0"}
```

### Resolver Class API

For more control, use the `Resolver` class directly:

```python
from pubgrub_py import Resolver, ResolutionError

resolver = Resolver()

# Add packages one by one
resolver.add_package("app", "1.0.0", {"lib": ">=1.0.0"})
resolver.add_package("lib", "1.0.0", {})
resolver.add_package("lib", "2.0.0", {})

# Resolve
try:
    result = resolver.resolve({"app": ">=1.0.0"})
    print(result)  # {"app": "1.0.0", "lib": "2.0.0"}
except ResolutionError as e:
    print(f"Resolution failed: {e}")
```

## Version Constraint Syntax

This library supports PEP 440 version specifiers:

| Operator | Example | Meaning |
|----------|---------|---------|
| `>=` | `>=1.0.0` | Greater than or equal |
| `<=` | `<=2.0.0` | Less than or equal |
| `>` | `>1.0.0` | Greater than |
| `<` | `<2.0.0` | Less than |
| `==` | `==1.5.0` | Exact match |
| `!=` | `!=1.3.0` | Not equal |
| `~=` | `~=1.4.0` | Compatible release (>=1.4.0,<1.5.0) |

Combined constraints are separated by commas: `>=1.0.0,<2.0.0`

## Error Handling

When resolution fails, `ResolutionError` is raised with a human-readable explanation:

```python
from pubgrub_py import resolve, ResolutionError

try:
    resolve(
        requirements={"a": ">=1.0.0", "b": ">=1.0.0"},
        available={
            "a": {"1.0.0": {"shared": ">=2.0.0"}},
            "b": {"1.0.0": {"shared": "<2.0.0"}},
            "shared": {"1.0.0": {}, "2.0.0": {}},
        },
    )
except ResolutionError as e:
    print(e)
    # Output explains the conflict:
    # "Because a 1.0.0 depends on shared >=2.0.0
    #  and b 1.0.0 depends on shared <2.0.0,
    #  a 1.0.0 and b 1.0.0 are incompatible."
```

## Development

### Building

```bash
# Install maturin
pip install maturin

# Build and install in development mode
cd packages/pubgrub-py
maturin develop

# Build release wheel
maturin build --release
```

### Testing

```bash
# Run Python tests
pytest tests/

# Run Rust tests
cargo test
```

## License

MIT
