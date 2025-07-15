# Inventory

## Potential Removals
- None at the moment.

## Desired Functionality
- Add more ByteSource integrations (e.g. memory mapped arrays, rope-like stores).
- Provide asynchronous-friendly wrappers without forcing async code in the core.
- Example showcasing integration with Python via the `pyo3` feature.

## Discovered Issues
- `ByteOwner` implementations could expose safe methods for reclaiming owned data.
