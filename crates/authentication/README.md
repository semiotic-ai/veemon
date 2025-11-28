# authentication

unified blockchain authentication and validation library for ethereum and solana.


## per era validation


### ethereum pre-merge validation

```rust
use authentication::ethereum::{Epoch, EthereumPreMergeValidator};
use validation::PreMergeAccumulator;

let validator = EthereumPreMergeValidator::new(accumulator);
let result = validator.validate_single_epoch(epoch)?;
```

### ethereum post-merge validation

```rust
use authentication::ethereum::EthereumPostMergeValidator;

let validator = EthereumPostMergeValidator;
let result = validator.validate_era((historical_roots, beacon_blocks))?;
```

### ethereum post-capella validation

```rust
use authentication::ethereum::EthereumPostCapellaValidator;

let validator = EthereumPostCapellaValidator;
let result = validator.validate_era((historical_summaries, beacon_blocks))?;
```

### solana validation

```rust
use authentication::solana::SolanaValidator;

let validator = SolanaValidator::new(tree_depth);
let result = validator.validate_era((roots, blocks))?;
```

### inclusion proofs

```rust
use authentication::ethereum::{generate_inclusion_proofs, verify_inclusion_proofs};

// generate proofs for headers
let proofs = generate_inclusion_proofs(epochs, headers)?;

// verify proofs
verify_inclusion_proofs(&proofs, &accumulator)?;
```

## features

- **multi-chain support**: ethereum (pow + pos) and solana
- **era-based validation**: pre-merge, post-merge, and post-capella eras
- **merkle proofs**: generate and verify inclusion proofs
- **type-safe**: parse-don't-validate design with strong type guarantees
- **generic traits**: extensible trait-based architecture

## architecture

the crate is organized by blockchain:
- `ethereum::*` - ethereum consensus validation across all eras
- `solana::*` - solana validator epoch validation
- `traits::EraValidationContext` - generic validation trait
- `error::AuthenticationError` - unified error handling

## ethereum eras

| era | block range | consensus | validator |
|-----|-------------|-----------|-----------|
| pre-merge | 0 - 15,537,393 | proof of work | `EthereumPreMergeValidator` |
| post-merge | 15,537,394 - 17,034,869 | proof of stake | `EthereumPostMergeValidator` |
| post-capella | 17,034,870+ | proof of stake | `EthereumPostCapellaValidator` |
