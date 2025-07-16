# Inventory

## Potential Removals
- None at the moment.

## Desired Functionality
- Add ByteSource integration for rope-like stores.
- Provide asynchronous-friendly wrappers without forcing async code in the core.

## Discovered Issues
- `ByteOwner` implementations could expose safe methods for reclaiming owned data.
