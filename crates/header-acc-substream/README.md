# header_accumulator Substreams modules

This package was initialized via `substreams init`, using the `evm-minimal` template.

## Usage

```bash
substreams build
substreams auth
substreams gui
```

## Modules

### `map_my_data`

This module extracts small bits of block data, and does simple computations over the 
number of **transactions** in each block.

brew install llvm

export PATH="/opt/homebrew/opt/llvm/bin:$PATH"

export CC=/opt/homebrew/opt/llvm/bin/clang
export AR=/opt/homebrew/opt/llvm/bin/llvm-ar

export SUBSTREAMS_API_TOKEN=<token>

substreams run header-accumulator-v0.1.0.spkg map_accumulator --stop-block +9000 --production-mode
