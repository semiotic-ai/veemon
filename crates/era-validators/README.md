# era-validators

> **⚠️ DEPRECATED**: This crate has been superseded by the `authentication` crate.

All validators have moved to the `authentication` crate with improved organization and unified error handling.

## Migration Guide

| Old (era-validators) | New (authentication) |
|-----|-----|
| `use era_validators::*;` | `use authentication::*;` |
| `EthereumPreMergeValidator` | `authentication::ethereum::EthereumPreMergeValidator` |
| `EthereumPostMergeValidator` | `authentication::ethereum::EthereumPostMergeValidator` |
| `EthereumPostCapellaValidator` | `authentication::ethereum::EthereumPostCapellaValidator` |
| `SolanaValidator` | `authentication::solana::SolanaValidator` |
| `EraValidationContext` | `authentication::EraValidationContext` |
| `EraValidatorGeneric` | `authentication::EraValidatorGeneric` |

## Why Was This Deprecated?

The `era-validators` crate has been consolidated into the `authentication` crate to:
- Unify blockchain authentication across Ethereum and Solana
- Improve error handling with a unified `AuthenticationError` type
- Better organize code by blockchain (ethereum/, solana/)
- Reduce duplication between header-accumulator and era-validators
- Simplify the dependency graph

## Quick Start with New Crate

### Pre-merge Ethereum Validation
```rust
use authentication::ethereum::EthereumPreMergeValidator;
use authentication::EraValidationContext;

let validator = EthereumPreMergeValidator::default();
let result = validator.validate_era((epoch_number, headers))?;
```

### Post-merge Ethereum Validation
```rust
use authentication::ethereum::EthereumPostMergeValidator;

let validator = EthereumPostMergeValidator;
let result = validator.validate_era((historical_roots, beacon_blocks))?;
```

### Post-Capella Ethereum Validation
```rust
use authentication::ethereum::EthereumPostCapellaValidator;

let validator = EthereumPostCapellaValidator;
let result = validator.validate_era((historical_summaries, beacon_blocks))?;
```

### Solana Validation
```rust
use authentication::solana::SolanaValidator;

let validator = SolanaValidator::new(tree_depth);
let result = validator.validate_era((roots, blocks))?;
```

## See Also

- [authentication crate](../authentication/README.md) - Full documentation
- [vee crate](../vee/README.md) - Main entry point with convenience re-exports
