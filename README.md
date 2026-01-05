# flyio-dist-sys
My implementation for [fly.io distributed systems challenge](https://fly.io/dist-sys/) written in Rust.

## Usage
Code organized in Cargo workspace.
Each challenge is a separate binary, __proto__ is shared lib with protocol specific stuff.

Makefile contains [Maelstrom](https://github.com/jepsen-io/maelstrom) commands,
assuming maelstrom binary at path `~/Downloads/maelstrom/maelstrom`.
- `make test-echo`
- `make test-unique-id-generation`
- `make test-broadcast`

