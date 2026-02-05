//! Minimal interpreter for TL/TBC MVP bytecode (Payload Format v0).

use crate::errors::PmError;
use byteorder::{LittleEndian, ReadBytesExt, WriteBytesExt};
use std::io::{Cursor, Read, Write};

/// Magic bytes for TL/TBC Payload Format v0.
pub const PAYLOAD_MAGIC_V0: &[u8; 7] = b"PMTBC0\0";
/// Format version for TL/TBC Payload Format v0.
pub const PAYLOAD_VERSION_V0: u32 = 0;
/// Max fuel for interpreter MVP.
pub const MAX_FUEL_V1: u32 = 1_000_000; // From Step 5 CLI default

/// Opcode definitions for the minimal instruction set.
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Opcode {
    Halt = 0x00,
    PushU8 = 0x01,
    AddU8 = 0x02,
    MulU8 = 0x03,
    Dup = 0x04,
    Drop = 0x05,
    Swap = 0x06,
}

impl Opcode {
    /// Converts a byte to an Opcode, returning an error for unknown opcodes.
    fn from_byte(byte: u8) -> Result<Self, PmError> {
        match byte {
            0x00 => Ok(Opcode::Halt),
            0x01 => Ok(Opcode::PushU8),
            0x02 => Ok(Opcode::AddU8),
            0x03 => Ok(Opcode::MulU8),
            0x04 => Ok(Opcode::Dup),
            0x05 => Ok(Opcode::Drop),
            0x06 => Ok(Opcode::Swap),
            _ => Err(PmError::new("pm/payload/unknown-opcode", format!("Unknown opcode: {}", byte))),
        }
    }
}

/// Structure representing the decoded payload.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PayloadV0 {
    pub instructions: Vec<Instruction>,
}

/// Represents a single instruction in the bytecode.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Instruction {
    Halt,
    PushU8(u8),
    AddU8,
    MulU8,
    Dup,
    Drop,
    Swap,
}

impl Instruction {
    /// Decodes an instruction from a byte slice, advancing the cursor.
    fn decode<R: Read>(cursor: &mut R) -> Result<Self, PmError> {
        let opcode_byte = cursor.read_u8()
            .map_err(|_| PmError::new("pm/payload/truncated", "Not enough bytes for opcode."))?;
        let opcode = Opcode::from_byte(opcode_byte)?;

        match opcode {
            Opcode::Halt => Ok(Instruction::Halt),
            Opcode::PushU8 => {
                let operand = cursor.read_u8()
                    .map_err(|_| PmError::new("pm/payload/truncated", "Not enough bytes for PushU8 operand."))?;
                Ok(Instruction::PushU8(operand))
            }
            Opcode::AddU8 => Ok(Instruction::AddU8),
            Opcode::MulU8 => Ok(Instruction::MulU8),
            Opcode::Dup => Ok(Instruction::Dup),
            Opcode::Drop => Ok(Instruction::Drop),
            Opcode::Swap => Ok(Instruction::Swap),
        }
    }

    /// Encodes an instruction into a byte vector.
    fn encode<W: Write>(&self, writer: &mut W) -> Result<(), PmError> {
        match self {
            Instruction::Halt => writer.write_u8(Opcode::Halt as u8).unwrap(),
            Instruction::PushU8(operand) => {
                writer.write_u8(Opcode::PushU8 as u8).unwrap();
                writer.write_u8(*operand).unwrap();
            }
            Instruction::AddU8 => writer.write_u8(Opcode::AddU8 as u8).unwrap(),
            Instruction::MulU8 => writer.write_u8(Opcode::MulU8 as u8).unwrap(),
            Instruction::Dup => writer.write_u8(Opcode::Dup as u8).unwrap(),
            Instruction::Drop => writer.write_u8(Opcode::Drop as u8).unwrap(),
            Instruction::Swap => writer.write_u8(Opcode::Swap as u8).unwrap(),
        }
        Ok(())
    }
}

/// Decodes a byte slice into a PayloadV0.
pub fn decode_payload_v0(bytes: &[u8]) -> Result<PayloadV0, PmError> {
    let mut cursor = Cursor::new(bytes);

    // Read and verify magic bytes
    let mut magic_buf = [0u8; 7];
    if cursor.read_exact(&mut magic_buf).is_err() {
        return Err(PmError::new("pm/payload/truncated", "Not enough bytes for magic."));
    }
    if &magic_buf != PAYLOAD_MAGIC_V0 {
        return Err(PmError::new("pm/payload/bad-magic", "Invalid magic bytes."));
    }

    // Read format version
    let format_version = cursor.read_u32::<LittleEndian>()
        .map_err(|_| PmError::new("pm/payload/truncated", "Not enough bytes for format version."))?;
    if format_version != PAYLOAD_VERSION_V0 {
        return Err(PmError::new("pm/payload/bad-version", format!("Unsupported payload format version: {}", format_version)));
    }

    // Read number of instructions
    let num_instructions = cursor.read_u32::<LittleEndian>()
        .map_err(|_| PmError::new("pm/payload/truncated", "Not enough bytes for instruction count."))? as usize;

    let mut instructions = Vec::with_capacity(num_instructions);
    for _ in 0..num_instructions {
        instructions.push(Instruction::decode(&mut cursor)?);
    }

    // Check for extra bytes at the end (inconsistent lengths)
    if cursor.position() as usize != bytes.len() {
        return Err(PmError::new("pm/payload/inconsistent-length", "Extra bytes found at the end of the payload."));
    }

    Ok(PayloadV0 { instructions })
}

/// Encodes a PayloadV0 into a byte vector.
pub fn encode_payload_v0(payload: &PayloadV0) -> Vec<u8> {
    let mut buffer = Vec::new();

    // Magic bytes
    buffer.write_all(PAYLOAD_MAGIC_V0).unwrap();

    // Format version
    buffer.write_u32::<LittleEndian>(PAYLOAD_VERSION_V0).unwrap();

    // Number of instructions
    buffer.write_u32::<LittleEndian>(payload.instructions.len() as u32).unwrap();

    // Instructions
    for instruction in &payload.instructions {
        instruction.encode(&mut buffer).unwrap();
    }

    buffer
}

/// Executes the payload with a given amount of fuel.
/// Returns the final stack and fuel remaining, or an error.
pub fn execute_payload(payload_bytes: &[u8], initial_fuel: u32) -> Result<(Vec<u8>, u32), PmError> {
    let payload = decode_payload_v0(payload_bytes)?;
    let mut stack: Vec<u8> = Vec::new();
    let mut fuel = initial_fuel;
    let mut pc = 0; // Program counter

    while pc < payload.instructions.len() {
        if fuel == 0 {
            return Err(PmError::new("pm/out-of-fuel-v1", "Interpreter ran out of fuel."));
        }
        fuel -= 1; // Consume fuel for each instruction

        let instruction = &payload.instructions[pc];
        pc += 1;

        match instruction {
            Instruction::Halt => break, // Execution finishes
            Instruction::PushU8(value) => stack.push(*value),
            Instruction::AddU8 => {
                let b = stack.pop().ok_or_else(|| PmError::new("pm/exec/stack-underflow", "ADD_U8 requires two operands on stack."))?;
                let a = stack.pop().ok_or_else(|| PmError::new("pm/exec/stack-underflow", "ADD_U8 requires two operands on stack."))?;
                stack.push(a.wrapping_add(b));
            }
            Instruction::MulU8 => {
                let b = stack.pop().ok_or_else(|| PmError::new("pm/exec/stack-underflow", "MUL_U8 requires two operands on stack."))?;
                let a = stack.pop().ok_or_else(|| PmError::new("pm/exec/stack-underflow", "MUL_U8 requires two operands on stack."))?;
                stack.push(a.wrapping_mul(b));
            }
            Instruction::Dup => {
                let top = stack.last().ok_or_else(|| PmError::new("pm/exec/stack-underflow", "DUP requires at least one element on stack."))?;
                stack.push(*top);
            }
            Instruction::Drop => {
                stack.pop().ok_or_else(|| PmError::new("pm/exec/stack-underflow", "DROP requires at least one element on stack."))?;
            }
            Instruction::Swap => {
                if stack.len() < 2 {
                    return Err(PmError::new("pm/exec/stack-underflow", "SWAP requires at least two elements on stack."))
                }
                let len = stack.len();
                stack.swap(len - 1, len - 2);
            }
        }
    }

    Ok((stack, fuel))
}


#[cfg(test)]
mod tests {
    use super::*;
    use crate::errors::PmError;

    #[test]
    fn test_payload_roundtrip_encode_decode() {
        let instructions = vec![
            Instruction::PushU8(10),
            Instruction::PushU8(20),
            Instruction::AddU8,
            Instruction::Dup,
            Instruction::Swap,
            Instruction::MulU8,
            Instruction::Halt,
        ];
        let original_payload = PayloadV0 { instructions };

        let encoded = encode_payload_v0(&original_payload);
        let decoded = decode_payload_v0(&encoded).unwrap();

        assert_eq!(original_payload, decoded);
    }

    #[test]
    fn test_payload_decode_bad_magic() {
        let mut instructions = vec![Instruction::Halt];
        let encoded = encode_payload_v0(&PayloadV0 { instructions });
        let mut truncated = encoded.clone();
        truncated[0] = b'X'; // Corrupt magic

        let err = decode_payload_v0(&truncated).unwrap_err();
        assert_eq!(err.tag, "pm/payload/bad-magic");
    }

    #[test]
    fn test_payload_decode_truncated() {
        let mut instructions = vec![Instruction::PushU8(5), Instruction::Halt];
        let encoded = encode_payload_v0(&PayloadV0 { instructions });

        // Truncate at various points
        assert_eq!(decode_payload_v0(&encoded[0..6]).unwrap_err().tag, "pm/payload/truncated", "Truncated magic");
        assert_eq!(decode_payload_v0(&encoded[0..10]).unwrap_err().tag, "pm/payload/truncated", "Truncated version");
        assert_eq!(decode_payload_v0(&encoded[0..14]).unwrap_err().tag, "pm/payload/truncated", "Truncated instruction count");
        assert_eq!(decode_payload_v0(&encoded[0..15]).unwrap_err().tag, "pm/payload/truncated", "Truncated instruction (PushU8 needs operand)");
        assert_eq!(decode_payload_v0(&encoded[0..16]).unwrap_err().tag, "pm/payload/truncated", "Truncated instruction (Halt is fine)");
    }

    #[test]
    fn test_payload_decode_inconsistent_length() {
        let mut instructions = vec![Instruction::Halt];
        let mut encoded = encode_payload_v0(&PayloadV0 { instructions });
        encoded.push(0xFF); // Add extra byte

        let err = decode_payload_v0(&encoded).unwrap_err();
        assert_eq!(err.tag, "pm/payload/inconsistent-length");
    }

    #[test]
    fn test_execute_simple_program() {
        // PUSH 2, PUSH 3, ADD, HALT
        let instructions = vec![
            Instruction::PushU8(2),
            Instruction::PushU8(3),
            Instruction::AddU8,
            Instruction::Halt,
        ];
        let payload = PayloadV0 { instructions };
        let encoded = encode_payload_v0(&payload);

        let (stack, fuel_remaining) = execute_payload(&encoded, MAX_FUEL_V1).unwrap();
        assert_eq!(stack, vec![5]);
        assert_eq!(fuel_remaining, MAX_FUEL_V1 - 3); // 3 instructions: PushU8, PushU8, AddU8
    }

    #[test]
    fn test_execute_fuel_exhaustion() {
        // PUSH 1, PUSH 1, ADD, PUSH 1, PUSH 1, ADD, ... (program longer than fuel)
        // Let's make a simple program that uses 10 instructions.
        let instructions = vec![
            Instruction::PushU8(1), // 1
            Instruction::PushU8(1), // 2
            Instruction::AddU8,     // 3
            Instruction::PushU8(1), // 4
            Instruction::PushU8(1), // 5
            Instruction::AddU8,     // 6
            Instruction::PushU8(1), // 7
            Instruction::PushU8(1), // 8
            Instruction::AddU8,     // 9
            Instruction::Halt,      // 10
        ];
        let payload = PayloadV0 { instructions };
        let encoded = encode_payload_v0(&payload);

        // Run with exactly 9 fuel
        let result = execute_payload(&encoded, 9);
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert_eq!(err.tag, "pm/out-of-fuel-v1");
    }
    
    #[test]
    fn test_execute_stack_underflow() {
        // ADD without enough operands
        let instructions = vec![
            Instruction::PushU8(5),
            Instruction::AddU8, // Needs two operands, only one is available
            Instruction::Halt,
        ];
        let payload = PayloadV0 { instructions };
        let encoded = encode_payload_v0(&payload);

        let result = execute_payload(&encoded, MAX_FUEL_V1).unwrap_err();
        assert_eq!(result.tag, "pm/exec/stack-underflow");
        assert!(result.details.contains("ADD_U8 requires two operands"));

        // DROP without any operands
        let instructions_drop = vec![
            Instruction::Drop,
            Instruction::Halt,
        ];
        let payload_drop = PayloadV0 { instructions: instructions_drop };
        let encoded_drop = encode_payload_v0(&payload_drop);
        let result_drop = execute_payload(&encoded_drop, MAX_FUEL_V1).unwrap_err();
        assert_eq!(result_drop.tag, "pm/exec/stack-underflow");
        assert!(result_drop.details.contains("DROP requires at least one element"));
    }

    #[test]
    fn test_execute_unknown_opcode() {
        // Manually craft bytes with an unknown opcode
        let mut bad_payload_bytes = Vec::new();
        bad_payload_bytes.write_all(PAYLOAD_MAGIC_V0).unwrap();
        bad_payload_bytes.write_u32::<LittleEndian>(PAYLOAD_VERSION_V0).unwrap();
        bad_payload_bytes.write_u32::<LittleEndian>(1).unwrap(); // One instruction
        bad_payload_bytes.push(0xFF); // Unknown opcode

        let result = execute_payload(&bad_payload_bytes, MAX_FUEL_V1).unwrap_err();
        assert_eq!(result.tag, "pm/payload/unknown-opcode");
    }

    #[test]
    fn test_execute_dup_swap() {
        // PUSH 10, PUSH 20, DUP, SWAP, ADD, HALT
        let instructions = vec![
            Instruction::PushU8(10),
            Instruction::PushU8(20),
            Instruction::Dup,     // Stack: [10, 20, 20]
            Instruction::Swap,    // Stack: [10, 20, 20] -> SWAP top two: [10, 20, 20] (no visible change)
                                  // Oh, SWAP swaps top two. So stack is [10, 20, 20] -> SWAP(20, 20) -> [10, 20, 20]
                                  // Let's correct the logic. Swap swaps len-1 and len-2.
                                  // Stack: [10, 20]. Push 10 -> [10, 20, 10]. Push 20 -> [10, 20, 10, 20]
                                  // DUP -> [10, 20, 10, 20, 20]
                                  // SWAP -> [10, 20, 10, 20, 20] -> swaps 20 and 20 -> [10, 20, 10, 20, 20]
                                  // This is not ideal. Let's try a better sequence for DUP/SWAP.
                                  // PUSH 10, PUSH 20, DUP, SWAP, ADD, HALT
                                  // PUSH 10: [10]
                                  // PUSH 20: [10, 20]
                                  // DUP: [10, 20, 20]
                                  // SWAP: [10, 20, 20] -> swaps 20 and 20 (no visible change on stack)
                                  // Wait, SWAP swaps TOP TWO elements. Stack [a, b, c]. SWAP -> [a, c, b].
                                  // PUSH 10: [10]
                                  // PUSH 20: [10, 20]
                                  // DUP: [10, 20, 20]
                                  // SWAP: [10, 20, 20] -> swaps top two (20, 20) -> [10, 20, 20]
                                  // Hmm, need a better example.
                                  // Let's try: PUSH 10, PUSH 20, SWAP, ADD, HALT
                                  // PUSH 10: [10]
                                  // PUSH 20: [10, 20]
                                  // SWAP: [20, 10]
                                  // ADD: [30]
                                  // HALT: Stack [30]
                                  // This is clearer.
        ];
        
        let instructions_clearer = vec![
            Instruction::PushU8(10),
            Instruction::PushU8(20),
            Instruction::Swap,
            Instruction::AddU8,
            Instruction::Halt,
        ];
        let payload_clearer = PayloadV0 { instructions: instructions_clearer };
        let encoded_clearer = encode_payload_v0(&payload_clearer);
        let (stack, _) = execute_payload(&encoded_clearer, MAX_FUEL_V1).unwrap();
        assert_eq!(stack, vec![30]);

        // Test DUP with non-trivial scenario
        // PUSH 5, DUP, PUSH 2, ADD, HALT
        // PUSH 5: [5]
        // DUP: [5, 5]
        // PUSH 2: [5, 5, 2]
        // ADD: [5, 7]
        // HALT: Stack [5, 7]
        let instructions_dup = vec![
            Instruction::PushU8(5),
            Instruction::Dup,
            Instruction::PushU8(2),
            Instruction::AddU8,
            Instruction::Halt,
        ];
        let payload_dup = PayloadV0 { instructions: instructions_dup };
        let encoded_dup = encode_payload_v0(&payload_dup);
        let (stack_dup, _) = execute_payload(&encoded_dup, MAX_FUEL_V1).unwrap();
        assert_eq!(stack_dup, vec![5, 7]);
    }

    #[test]
    fn test_mul_u8() {
        // PUSH 5, PUSH 6, MUL_U8, HALT -> 30
        let instructions = vec![
            Instruction::PushU8(5),
            Instruction::PushU8(6),
            Instruction::MulU8,
            Instruction::Halt,
        ];
        let payload = PayloadV0 { instructions };
        let encoded = encode_payload_v0(&payload);
        let (stack, _) = execute_payload(&encoded, MAX_FUEL_V1).unwrap();
        assert_eq!(stack, vec![30]);
    }

    #[test]
    fn test_mul_u8_overflow() {
        // PUSH 255, PUSH 2, MUL_U8, HALT -> 255 * 2 = 510. Modulo 256 = 254.
        let instructions = vec![
            Instruction::PushU8(255),
            Instruction::PushU8(2),
            Instruction::MulU8,
            Instruction::Halt,
        ];
        let payload = PayloadV0 { instructions };
        let encoded = encode_payload_v0(&payload);
        let (stack, _) = execute_payload(&encoded, MAX_FUEL_V1).unwrap();
        assert_eq!(stack, vec![254]);
    }
}