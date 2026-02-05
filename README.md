# PeaceMachine

PeaceMachine is the execution engine for the Mac Lane environment. It executes certified programs written in Transgender Language / Transgender Bytecode (TL/TBC) by substitution + reduction.

## Design Goals

*   Deterministic execution and outputs
*   High performance, but also easy to audit
*   Provably correct relative to the verified/certified inputs and the specified semantics
*   For MVP: interpreting certified programs is enough; “compile to machine code” is a later goal.

PeaceMachine accepts a Certified Program Bundle (CPB-1), verifies it, then executes the payload.

## Hard Constraints

1.  Only modify files in PeaceMachine/.
2.  No mocking, no fake servers, no stub “pretend verification” beyond clearly tagged `pm/unimplemented` returns.
3.  Prefer additive, low-risk steps.
4.  Determinism: stable ordering, stable error tags, stable serialization.
5.  All errors must use stable tags: `pm/<tag>: <details>`.

## CPB-1 MVP Format (Binary)

The Certified Program Bundle version 1 (CPB-1) is a self-contained binary format.

*   **Magic Bytes**: `PMCPB1\0` (8 bytes: `0x50 0x4d 0x43 0x50 0x42 0x31 0x00 0x00`)
*   **Format Version**: `u32` little-endian. Currently `1`.
*   **Payload Length**: `u32` little-endian. Length of the payload in bytes.
*   **Payload**: `bytes`. The TL/TBC bytecode to be executed.
*   **Certificate Length**: `u32` little-endian. Length of the certificate in bytes.
*   **Certificate**: `bytes`. Opaque bytes representing the certificate.

## PMTBC0 Payload Format (Internal MVP)

The Payload Format version 0 (PMTBC0) defines the internal structure of the executable bytecode.

*   **Magic Bytes**: `PMTBC0\0` (7 bytes: `0x50 0x4d 0x54 0x42 0x43 0x30 0x00`)
*   **Format Version**: `u32` little-endian. Currently `0`.
*   **Number of Instructions**: `u32` little-endian. The count of instructions in the payload.
*   **Instructions**: A sequence of instructions. Each instruction is one byte opcode plus optional operands.

### Instruction Set:

*   `0x00` = `HALT`
*   `0x01 <u8>` = `PUSH_U8 n` (Pushes `n` onto the stack)
*   `0x02` = `ADD_U8` (Pops `a`, `b`; pushes `(a + b) mod 256`)
*   `0x03` = `MUL_U8` (Pops `a`, `b`; pushes `(a * b) mod 256`)
*   `0x04` = `DUP` (Duplicates the top element of the stack)
*   `0x05` = `DROP` (Removes the top element of the stack)
*   `0x06` = `SWAP` (Swaps the top two elements of the stack)

Execution proceeds sequentially, consuming 1 fuel per instruction.

## CLI Usage Examples

```bash
# Display info about a bundle
peacemachine info <path/to/bundle.cpb>

# Verify a bundle without executing
peacemachine verify <path/to/bundle.cpb>

# Run a bundle with a specific fuel limit
peacemachine run <path/to/bundle.cpb> --fuel 50000

# Run a bundle with JSON output
peacemachine run <path/to/bundle.cpb> --fuel 100000 --json
```

## Determinism and Error Tags

*   **Determinism**: All outputs are stable, with no reliance on non-deterministic factors like HashMap iteration order.
*   **Error Tags**: All errors are tagged with a stable prefix `pm/` (e.g., `pm/bundle/bad-magic`, `pm/exec/stack-underflow`). These tags are used in both human-readable and JSON error outputs.

## Development

This project uses Rust and Cargo. Standard Cargo commands apply:
*   `cargo build`
*   `cargo test`
*   `cargo fmt`
*   `cargo clippy` (recommended)