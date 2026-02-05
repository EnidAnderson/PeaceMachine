/// Placeholder for pmx-trace-v1 emission.
/// Keep this canonical + deterministic when implemented.
#[derive(Debug, Clone)]
pub struct ExecTraceV1 {
    pub version: u32,
    pub fuel_used: u64,
}
