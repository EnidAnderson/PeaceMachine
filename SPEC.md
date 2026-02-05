Below is a formal, implementation-ready specification for the new execution-engine subproject PeaceMachine. It is written to be “deterministic for an implementer” (Claude/Gemini/humans), with explicit MUST/SHOULD/MAY requirements, versioned artifacts, and guardrails that preserve the core contracts of Mac Lane.

⸻

PeaceMachine Specification v0.1

0. Status and Scope

Status: New standalone module / repository (sibling directory to clean_kernel/ and EdgeLorD/).
Primary deliverable (MVP): A deterministic, auditable interpreter that executes certified Mac Lane programs by substitution + reduction, producing a result and optional execution trace.
Future deliverable (non-MVP): A “certified executable” emitter that represents execution as transgender bytecode (a certified, replayable execution artifact), potentially transpiling to lower-level host execution.

PeaceMachine is the execution engine of the Mac Lane programming environment.

⸻

1. Naming and Identity
	•	Project name: PeaceMachine (folder PeaceMachine/).
	•	The project MAY accept alias spellings in user-facing strings (Peace Machine, PeaceMachine, peacemachiine), but the canonical crate/package name SHOULD be peacemachine (Rust convention).
	•	Branding terms:
	•	Transgender Language (TL): the certified input language accepted by PeaceMachine.
	•	Transgender Bytecode (TBC): a canonical, certified, executable representation suitable for replay and/or distribution.

Note: This spec treats these as names of internal technical artifacts. Nothing in this spec makes claims about people; they are simply identifiers for the DSL and its bytecode.

⸻

2. Licensing
	•	PeaceMachine MUST be released under a highly permissive variant of the Hippocratic License, consistent with the rest of the Mac Lane ecosystem.
	•	Repository MUST include a LICENSE file and MUST include license headers or a consistent license policy (e.g., top-level LICENSE + SPDX identifiers in Cargo.toml).

⸻

3. Architecture Principles

PeaceMachine must be:
	1.	High performance
	2.	Easily audited
	3.	Provably correct (relative to the certified input semantics)
	4.	Deterministic (same certified input + same config => same output)

3.1 Determinism Contract (PM-DET-1)

PeaceMachine MUST be deterministic under a fixed version of:
	•	the interpreter,
	•	the certification verifier,
	•	canonical decoding/encoding,
	•	and runtime configuration.

No hash-map iteration order may influence semantics. Use stable ordering for any collections that affect behavior.

3.2 Auditability Contract (PM-AUD-1)
	•	Default build MUST use no unsafe (deny-by-default recommended; allow exceptions only with justification + tests).
	•	Input validation and verifier code MUST be written for clarity over cleverness.
	•	Execution core MUST be structured to allow local reasoning:
	•	small modules,
	•	explicit invariants,
	•	tests that assert invariants.

3.3 Correctness Contract (PM-COR-1)

PeaceMachine MUST either:
	•	execute a program according to the semantics implied by its certificate and produce the correct result, or
	•	refuse to execute with a typed error (stable tag + diagnostics).

⸻

4. Inputs, Outputs, and Trust Model

4.1 Input Artifact: Certified Program Bundle v1 (CPB-1)

PeaceMachine accepts certified programs written in TL/TBC. For MVP, PeaceMachine MAY accept a higher-level certified bundle produced by clean_kernel, as long as it is fully certified.

CPB-1 MUST contain:
	•	format_version: integer
	•	payload: canonical bytes of the program (TL/TBC)
	•	certificate: a certificate chain or witness sufficient to validate payload
	•	scope_digest (or equivalent): binds execution semantics to a certified scope
	•	optional trace (replay artifact), if present

CPB-1 MUST be self-describing: it MUST identify which verifier and artifact versions are required (e.g., tags like doctrine-cert-v1, facet-cert-v1, trace-v1).

Design intent: PeaceMachine is not a compiler frontend. It is an executor for already-certified artifacts.

4.2 Trust Model

PeaceMachine MUST treat all inputs as untrusted bytes and MUST:
	•	parse them using a total/checked decoder,
	•	validate canonical encodings where required,
	•	verify the certificate before executing.

PeaceMachine MUST NOT execute any uncertified payload unless explicitly run in a clearly labeled “developer/insecure mode” (non-default, opt-in, loud).

⸻

5. Certification Verification Requirements (PM-VER-1)

5.1 Verification Gate

Before any execution step:
	•	PeaceMachine MUST verify the certificate(s) attached to the program bundle.
	•	If verification fails, execution MUST halt with a stable error tag:
	•	pm/verify-failed: ...

5.2 Versioned Verifier Policy

PeaceMachine MUST enforce version compatibility:
	•	If the certificate version is unsupported:
	•	pm/unsupported-cert-version: ...
	•	If the payload version is unsupported:
	•	pm/unsupported-payload-version: ...

5.3 Canonicality

Where the certificate relies on canonical encoding:
	•	PeaceMachine MUST re-encode and confirm canonical bytes match (or validate via a canonical-decoder invariant).
	•	If canonicality fails:
	•	pm/non-canonical-input: ...

⸻

6. Execution Semantics

PeaceMachine executes by substitution and reduction.

6.1 Execution Mode: Interpreter MVP (PM-EXEC-MVP-1)

For MVP, PeaceMachine MUST implement a deterministic evaluator:
	•	Input: certified TL/TBC payload
	•	Output: final result value (or normal form), plus optional execution artifact(s)

MVP evaluator MUST support:
	•	fuel-bounded evaluation (to prevent nontermination attacks)
	•	deterministic rewrite selection (no “lottery semantics”)

6.2 Fuel and Outcomes

PeaceMachine MUST provide these outcomes:
	1.	Success
	•	returns final value/result and a result hash if available
	•	tag: pm/exec-ok-v1
	2.	Out-of-fuel
	•	returns partial progress + checkpoint + optional trace segment
	•	tag: pm/out-of-fuel-v1
	3.	Runtime rejection
	•	e.g., malformed bytecode step, invalid instruction, illegal substitution
	•	tag prefix: pm/runtime-error/...

6.3 Replayability and Trace Interop (PM-TRACE-1)

PeaceMachine SHOULD support emitting an execution trace artifact that is:
	•	stable,
	•	canonical,
	•	replay-checkable,
	•	and aligned with the ecosystem’s existing trace-v1 semantics where appropriate.

If PeaceMachine emits a trace, the trace MUST be:
	•	versioned
	•	canonical in field ordering
	•	deterministically constructed

Failure modes MUST use stable tags (see §9).

Important: PeaceMachine may treat its trace as an execution trace (operational) rather than a proof trace (certification). But it should still be structured and replayable.

⸻

7. “Certified Executable” Output (Non-MVP, PM-CEX-1)

PeaceMachine’s aspirational feature is producing a certified executable representing the execution as transgender bytecode (or similar representation of host computation).

7.1 Certified Executable Artifact v1 (CEX-1)

If implemented, PeaceMachine SHOULD be able to output:
	•	cex_version
	•	input_program_hash
	•	result_hash
	•	execution_trace or compressed replay plan
	•	verification_witness binding it back to the original certificate
	•	optional “host plan” that maps steps to host primitives (still auditable)

7.2 Soundness of “Compilation”

PeaceMachine MUST NOT claim compilation-to-host-instructions correctness unless it can:
	•	prove semantics preservation, or
	•	attach a witness that is verifiable by a checker.

For MVP, interpret-only is acceptable and preferred.

⸻

8. Project Layout and Integration

8.1 Repository Layout (recommended)

PeaceMachine/
  Cargo.toml
  LICENSE
  README.md
  src/
    lib.rs
    cli.rs
    bundle.rs          // CPB-1 decode/encode, validation
    verify.rs          // certificate verification gate
    eval/
      mod.rs
      interpreter.rs   // MVP evaluator
      fuel.rs
      trace.rs         // trace emission (if any)
    errors.rs          // stable error tags
  tests/
    golden/
    exec_mvp.rs
    verify.rs

8.2 Dependency Policy
	•	PeaceMachine MAY depend on clean_kernel crates as libraries (path dependency or workspace dependency), but MUST keep its own correctness boundary explicit:
	•	verifier gate,
	•	canonical decode,
	•	deterministic evaluation.

If reuse is possible, prefer it for:
	•	canonical encoding/decoding
	•	certificate verification primitives
	•	hash/digest utilities

But PeaceMachine MUST NOT import “developer convenience” code that weakens auditability or determinism.

⸻

9. Errors, Tags, and Diagnostics (PM-ERR-1)

All errors MUST begin with a stable tag prefix, followed by ::

Examples:
	•	pm/verify-failed: ...
	•	pm/unsupported-cert-version: ...
	•	pm/non-canonical-input: ...
	•	pm/out-of-fuel-v1: ...
	•	pm/runtime-error/malformed-instruction: ...

The tag portion MUST be stable across releases within the same major protocol version.

⸻

10. CLI Requirements (PM-CLI-1)

PeaceMachine MUST provide a CLI (even minimal) to:
	•	peacemachine verify <bundle>
	•	peacemachine run <bundle> [--fuel N] [--emit-trace] [--json]
	•	peacemachine info <bundle> (prints versions, digests, summary)

CLI output SHOULD support:
	•	human-readable mode
	•	machine-readable mode (JSON) for tooling

Warnings MUST go to stderr; machine-readable outputs MUST be clean on stdout.

⸻

11. Testing Requirements (PM-TEST-1)

PeaceMachine MUST include:

11.1 Golden Tests
	•	Known-certified bundles (fixtures) with expected results.
	•	Stable across platforms.

11.2 Determinism Tests
	•	Same input executed twice yields identical:
	•	result bytes / hashes
	•	trace bytes (if emitted)
	•	error tags (if failing)

11.3 Negative Tests
	•	invalid bundle structure
	•	invalid canonical bytes
	•	unsupported versions
	•	certificate mismatch
	•	out-of-fuel behavior is stable

11.4 Differential Checks (optional but valuable)

If there is an existing reference executor (even slow):
	•	compare outputs on random small programs.

⸻

12. Performance Requirements (PM-PERF-1)

Performance is a core goal, but not at the expense of auditability.

MVP performance targets SHOULD be expressed as:
	•	throughput on representative workloads
	•	memory bound behavior under fuel and/or step limits

Rules:
	•	Avoid allocations in inner loops where feasible.
	•	Prefer explicit stacks to recursion if stack overflow is plausible.
	•	Use stable data structures (e.g., Vec, BTreeMap) where ordering affects semantics.

⸻

13. Security and Resource Limits (PM-SEC-1)

PeaceMachine MUST protect hosts from hostile certified inputs that are valid but expensive:
	•	Fuel must be mandatory or have a safe default.
	•	Optional max-term-size / max-step-size guards.
	•	Decode limits for bundle sizes.

Failure should be clean:
	•	pm/resource-limit: ... or pm/out-of-fuel-v1: ...

⸻

14. Compatibility with “Bring Your Own Backend” Philosophy

PeaceMachine is one backend specialized for execution. The ecosystem may have other backends:
	•	proof-checking backends
	•	normalization backends
	•	alternative evaluators (symbolic, distributed, GPU, etc.)

Therefore:
	•	PeaceMachine MUST be modular:
	•	verify and eval are separate layers.
	•	PeaceMachine SHOULD define a minimal “backend interface” internally so alternative evaluators can be swapped without changing the bundle/verifier layer.

⸻

MVP Checklist (Implementation “Done When”)

PeaceMachine MVP is complete when:
	1.	verify refuses invalid bundles and accepts valid certified bundles.
	2.	run deterministically evaluates to a result (or out-of-fuel).
	3.	stable error tags exist and are tested.
	4.	at least:
	•	5 golden success tests,
	•	5 negative verification tests,
	•	3 determinism tests,
	•	2 out-of-fuel tests.
	5.	CLI supports verify and run.
	6.	License and README are present.

⸻

Optional Add-On: PeaceMachine Execution Trace v1 (PMX-TRACE-1)

If you want a concrete trace artifact for MVP, define:
	•	pmx-trace-v1:
	•	version
	•	start_hash
	•	end_hash
	•	fuel_used
	•	steps (list)
	•	each step includes:
	•	rule index or opcode id
	•	pre-hash/post-hash (optional)
	•	small metadata (optional)

Canonical field ordering MUST be enforced.

⸻

If you want, I can also generate:
	•	a README.md draft for PeaceMachine,
	•	a Cargo.toml skeleton with crate features (default, json, trace, dev-insecure),
	•	and a crisp “interface boundary” plan describing exactly which clean_kernel crates PeaceMachine should depend on (and which it should not).
