# era-validators

> **⚠️ DEPRECATED**: This crate has been superseded by the `era-validation` crate.

All validators have moved to the `era-validation` crate with improved organization and unified error handling.

## Migration Guide

| Old (era-validators) | New (era-validation) |
|-----|-----|
| `use era_validators::*;` | `use era_validation::*;` |
| `EthereumPreMergeValidator` | `era_validation::ethereum::EthereumPreMergeValidator` |
| `EthereumPostMergeValidator` | `era_validation::ethereum::EthereumPostMergeValidator` |
| `EthereumPostCapellaValidator` | `era_validation::ethereum::EthereumPostCapellaValidator` |
| `SolanaValidator` | `era_validation::solana::SolanaValidator` |
| `EraValidationContext` | `era_validation::EraValidationContext` |
| `EraValidatorGeneric` | `era_validation::EraValidatorGeneric` |

## Why Was This Deprecated?

The `era-validators` crate has been consolidated into the `era-validation` crate to:
- Unify blockchain era validation across Ethereum and Solana
- Improve error handling with a unified `AuthenticationError` type
- Better organize code by blockchain (ethereum/, solana/)
- Reduce duplication between header-accumulator and era-validators
- Simplify the dependency graph

## Quick Start with New Crate

### Pre-merge Ethereum Validation
```rust
use era_validation::ethereum::EthereumPreMergeValidator;
use era_validation::EraValidationContext;

let validator = EthereumPreMergeValidator::default();
let result = validator.validate_era((epoch_number, headers))?;
```

### Post-merge Ethereum Validation
```rust
use era_validation::ethereum::EthereumPostMergeValidator;

let validator = EthereumPostMergeValidator;
let result = validator.validate_era((historical_roots, beacon_blocks))?;
```

### Post-Capella Ethereum Validation
```rust
use era_validation::ethereum::EthereumPostCapellaValidator;

let validator = EthereumPostCapellaValidator;
let result = validator.validate_era((historical_summaries, beacon_blocks))?;
```

### Solana Validation
```rust
use era_validation::solana::SolanaValidator;

let validator = SolanaValidator::new(tree_depth);
let result = validator.validate_era((roots, blocks))?;
```

## See Also

- [era-validation crate](../era-validation/README.md) - Full documentation
- [vee crate](../vee/README.md) - Main entry point with convenience re-exports
