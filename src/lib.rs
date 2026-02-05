//! PeaceMachine: certified execution engine for Mac Lane.
//!
//! MVP responsibilities:
//! - Decode + validate a certified program bundle (CPB-1)
//! - Verify certificates before execution
//! - Deterministically interpret TL/TBC by substitution + reduction

pub mod bundle;
pub mod errors;
pub mod verify;
pub mod eval;
