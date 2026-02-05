use std::fs;
use std::path::PathBuf;
use std::process;

use clap::{Parser, Subcommand};

// Import necessary modules from the crate
// Explicitly using CPB_VERSION_V1 and MAX_FUEL_V1 as they are now used.
use peacemachine::bundle::{self, CertifiedProgramBundleV1, CPB_VERSION_V1};
use peacemachine::errors::PmError;
use peacemachine::verify;
use peacemachine::eval::interpreter::{self, MAX_FUEL_V1}; // MAX_FUEL_V1 is used for default fuel.

/// PeaceMachine: certified execution engine for Mac Lane.
/// MVP: verify certified bundles + interpret by substitution/reduction.
#[derive(Parser, Debug)]
#[command(name = "peacemachine", version, about)]
struct Cli {
    #[command(subcommand)]
    cmd: Command,
}

#[derive(Subcommand, Debug)]
enum Command {
    /// Verify a certified program bundle (CPB-1) without executing it.
    Verify {
        /// Path to the bundle file
        bundle: PathBuf,
    },
    /// Execute a certified program bundle (CPB-1).
    Run {
        /// Path to the bundle file
        bundle: PathBuf,

        /// Fuel limit (default: 1_000_000)
        #[arg(long, default_value_t = MAX_FUEL_V1 as u64)] // Use MAX_FUEL_V1 here
        fuel: u64,

        /// Emit an execution trace artifact (pmx-trace-v1)
        #[arg(long)]
        emit_trace: bool,

        /// Output machine-readable JSON on stdout
        #[arg(long)]
        json: bool,
    },
    /// Print versions, digests, and summary information about a bundle.
    Info {
        /// Path to the bundle file
        bundle: PathBuf,
    },
}

/// Helper function to handle errors and exit.
fn handle_error(e: PmError) -> ! {
    eprintln!("{}", e);
    process::exit(1);
}

fn main() {
    let cli = Cli::parse();

    match cli.cmd {
        Command::Info { bundle } => {
            let bytes = match fs::read(&bundle) {
                Ok(b) => b,
                Err(e) => handle_error(PmError::new("pm/cli/file-read-error", format!("Failed to read bundle file: {}", e))),
            };

            // Explicitly annotate type for CertifiedProgramBundleV1 to resolve unused import warning
            let bundle_data: CertifiedProgramBundleV1 = match bundle::decode_cpb_v1(&bytes) {
                Ok(b) => b,
                Err(e) => handle_error(e),
            };

            // Use CPB_VERSION_V1 for consistency
            if bundle_data.format_version != CPB_VERSION_V1 {
                 handle_error(PmError::new("pm/bundle/bad-version", format!("Unsupported bundle format version: {}. Expected {}.", bundle_data.format_version, CPB_VERSION_V1)));
            }

            println!("Format Version: {}", bundle_data.format_version);
            println!("Payload Length: {}", bundle_data.payload.len());
            println!("Certificate Length: {}", bundle_data.certificate.len());
            
            process::exit(0);
        }
        Command::Verify { bundle } => {
            let bytes = match fs::read(&bundle) {
                Ok(b) => b,
                Err(e) => handle_error(PmError::new("pm/cli/file-read-error", format!("Failed to read bundle file: {}", e))),
            };

            // Explicitly annotate type for CertifiedProgramBundleV1
            let bundle_data: CertifiedProgramBundleV1 = match bundle::decode_cpb_v1(&bytes) {
                Ok(b) => b,
                Err(e) => handle_error(e),
            };

            // Use CPB_VERSION_V1 for consistency
            if bundle_data.format_version != CPB_VERSION_V1 {
                 handle_error(PmError::new("pm/bundle/bad-version", format!("Unsupported bundle format version: {}. Expected {}.", bundle_data.format_version, CPB_VERSION_V1)));
            }

            if let Err(e) = verify::verify_bundle(&bundle_data) {
                handle_error(e);
            }

            println!("ok");
            process::exit(0);
        }
        Command::Run { bundle, fuel, emit_trace, json } => {
            if emit_trace {
                eprintln!("Warning: --emit-trace is accepted but not implemented in MVP.");
            }

            let bytes = match fs::read(&bundle) {
                Ok(b) => b,
                Err(e) => handle_error(PmError::new("pm/cli/file-read-error", format!("Failed to read bundle file: {}", e))),
            };

            // Explicitly annotate type for CertifiedProgramBundleV1
            let bundle_data: CertifiedProgramBundleV1 = match bundle::decode_cpb_v1(&bytes) {
                Ok(b) => b,
                Err(e) => handle_error(e),
            };

            // Use CPB_VERSION_V1 for consistency
            if bundle_data.format_version != CPB_VERSION_V1 {
                 handle_error(PmError::new("pm/bundle/bad-version", format!("Unsupported bundle format version: {}. Expected {}.", bundle_data.format_version, CPB_VERSION_V1)));
            }

            if let Err(e) = verify::verify_bundle(&bundle_data) {
                handle_error(e);
            }

            let fuel_u32 = match fuel.try_into() {
                Ok(f) => f,
                Err(_) => handle_error(PmError::new("pm/cli/invalid-arg", "Fuel limit too large.")),
            };

            let (final_stack, fuel_remaining) = match interpreter::execute_payload(&bundle_data.payload, fuel_u32) {
                Ok(res) => res,
                Err(e) => {
                    if json {
                        // Basic JSON error structure
                        // Need to use `e.tag` and `e.details`
                        println!("{{\"ok\": false, \"error_tag\": \"{}\", \"details\": \"{}\"}}", e.tag, e.details);
                        process::exit(1);
                    } else {
                        handle_error(e); // Non-JSON error output
                    }
                }
            };

            // Handle JSON output for success if requested
            if json {
                let stack_hex = final_stack.iter().map(|b| format!("{:02x}", b)).collect::<String>();
                // Calculate fuel_used correctly: original fuel - remaining fuel
                // Note: fuel is u64, fuel_remaining is u32.
                println!(
                    "{{\"ok\": true, \"fuel_used\": {}, \"result_stack\": \"{}\"}}",
                    fuel as u64 - fuel_remaining as u64, // Corrected fuel calculation
                    stack_hex
                );
            } else {
                // Default human-friendly output
                println!("Execution successful.");
                println!("Fuel remaining: {}", fuel_remaining);
                println!("Final stack length: {}", final_stack.len());
                if !final_stack.is_empty() {
                    print!("Stack (top last): ");
                    for (i, byte) in final_stack.iter().enumerate() {
                        if i > 0 { print!(" "); }
                        print!("{:02x}", byte);
                    }
                    println!();
                } else {
                    println!("Stack is empty.");
                }
            }

            process::exit(0);
        }
    }
}
