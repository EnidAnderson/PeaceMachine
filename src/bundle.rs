use crate::errors::PmError;
use byteorder::{LittleEndian, ReadBytesExt, WriteBytesExt};
use std::io::{Cursor, Read, Write};

/// Magic bytes for CPB-1: ASCII "PMCPB1\0" followed by an extra null byte to make it 8 bytes.
/// The prompt stated "ASCII PMCPB1\0 (7 bytes + null)". Rust interpreted b"PMCPB1\0" as 7 bytes.
/// We explicitly define an 8-byte array to match expectations.
pub const CPB_MAGIC_V1: &[u8; 8] = &[
    0x50, // P
    0x4d, // M
    0x43, // C
    0x50, // P
    0x42, // B
    0x31, // 1
    0x00, // Null byte
    0x00, // Extra null byte to make it 8 bytes total.
];
/// Format version for CPB-1
pub const CPB_VERSION_V1: u32 = 1;

/// CPB-1 (Certified Program Bundle v1) - placeholder struct.
/// In MVP, this should decode bytes, validate canonicality, and expose the payload + certs.
#[derive(Debug, Clone, PartialEq, Eq)] // Added PartialEq, Eq for testing
pub struct CertifiedProgramBundleV1 {
    pub format_version: u32,
    pub payload: Vec<u8>,
    pub certificate: Vec<u8>,
}

/// Encodes a CertifiedProgramBundleV1 into a byte vector.
pub fn encode_cpb_v1(bundle: &CertifiedProgramBundleV1) -> Vec<u8> {
    let mut buffer = Vec::new();

    // Magic bytes
    buffer.write_all(CPB_MAGIC_V1).unwrap(); // CPB_MAGIC_V1 is &[u8; 8]

    // Format version
    buffer.write_u32::<LittleEndian>(bundle.format_version).unwrap();

    // Payload length and payload
    buffer.write_u32::<LittleEndian>(bundle.payload.len() as u32).unwrap();
    buffer.write_all(&bundle.payload).unwrap();

    // Certificate length and certificate
    buffer.write_u32::<LittleEndian>(bundle.certificate.len() as u32).unwrap();
    buffer.write_all(&bundle.certificate).unwrap();

    buffer
}

/// Decodes a byte slice into a CertifiedProgramBundleV1.
pub fn decode_cpb_v1(bytes: &[u8]) -> Result<CertifiedProgramBundleV1, PmError> {
    let mut cursor = Cursor::new(bytes);

    // Read and verify magic bytes
    let mut magic_buf = [0u8; 8]; // Expecting 8 bytes for magic
    if cursor.read_exact(&mut magic_buf).is_err() {
        return Err(PmError::new("pm/bundle/truncated", "Not enough bytes for magic."));
    }
    if &magic_buf != CPB_MAGIC_V1 {
        return Err(PmError::new("pm/bundle/bad-magic", "Invalid magic bytes."));
    }

    // Read format version
    let format_version = cursor.read_u32::<LittleEndian>()
        .map_err(|_| PmError::new("pm/bundle/truncated", "Not enough bytes for format version."))?;

    // Read payload length
    let payload_len = cursor.read_u32::<LittleEndian>()
        .map_err(|_| PmError::new("pm/bundle/truncated", "Not enough bytes for payload length."))? as usize;

    // Read payload
    let mut payload = vec![0; payload_len];
    if cursor.read_exact(&mut payload).is_err() {
        return Err(PmError::new("pm/bundle/truncated", "Not enough bytes for payload."));
    }

    // Read certificate length
    let cert_len = cursor.read_u32::<LittleEndian>()
        .map_err(|_| PmError::new("pm/bundle/truncated", "Not enough bytes for certificate length."))? as usize;

    // Read certificate
    let mut certificate = vec![0; cert_len];
    if cursor.read_exact(&mut certificate).is_err() {
        return Err(PmError::new("pm/bundle/truncated", "Not enough bytes for certificate."));
    }

    // Check for extra bytes at the end (inconsistent lengths)
    if cursor.position() as usize != bytes.len() {
        return Err(PmError::new("pm/bundle/inconsistent-length", "Extra bytes found at the end of the bundle."));
    }

    Ok(CertifiedProgramBundleV1 {
        format_version,
        payload,
        certificate,
    })
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_roundtrip_encode_decode() {
        let original_bundle = CertifiedProgramBundleV1 {
            format_version: CPB_VERSION_V1,
            payload: vec![1, 2, 3, 4, 5],
            certificate: vec![6, 7, 8, 9, 10, 11],
        };

        let encoded_bytes = encode_cpb_v1(&original_bundle);
        let decoded_bundle = decode_cpb_v1(&encoded_bytes).unwrap();

        assert_eq!(original_bundle, decoded_bundle);
    }

    #[test]
    fn test_roundtrip_empty_payload_certificate() {
        let original_bundle = CertifiedProgramBundleV1 {
            format_version: CPB_VERSION_V1,
            payload: vec![],
            certificate: vec![],
        };

        let encoded_bytes = encode_cpb_v1(&original_bundle);
        let decoded_bundle = decode_cpb_v1(&encoded_bytes).unwrap();

        assert_eq!(original_bundle, decoded_bundle);
    }

    #[test]
    fn test_reject_bad_magic() {
        let original_bundle = CertifiedProgramBundleV1 {
            format_version: CPB_VERSION_V1,
            payload: vec![1, 2],
            certificate: vec![3, 4],
        };
        let mut encoded_bytes = encode_cpb_v1(&original_bundle);
        // Corrupt the first byte of the magic sequence
        encoded_bytes[0] = b'X';

        let err = decode_cpb_v1(&encoded_bytes).unwrap_err();
        assert_eq!(err.tag, "pm/bundle/bad-magic");
    }

    #[test]
    fn test_reject_truncated_buffers() {
        let original_bundle = CertifiedProgramBundleV1 {
            format_version: CPB_VERSION_V1,
            payload: vec![1, 2, 3, 4, 5],
            certificate: vec![6, 7, 8, 9, 10, 11],
        };
        let encoded_bytes = encode_cpb_v1(&original_bundle);

        // Magic bytes are 8 bytes
        let magic_len = CPB_MAGIC_V1.len();
        let version_len = std::mem::size_of::<u32>();
        let payload_len_bytes = std::mem::size_of::<u32>();
        let payload_bytes = original_bundle.payload.len();
        let cert_len_bytes = std::mem::size_of::<u32>();
        let cert_bytes = original_bundle.certificate.len();

        // Test various truncation points
        assert_eq!(decode_cpb_v1(&encoded_bytes[0..magic_len - 1]).unwrap_err().tag, "pm/bundle/truncated", "Truncated magic");
        assert_eq!(decode_cpb_v1(&encoded_bytes[0..magic_len + version_len - 1]).unwrap_err().tag, "pm/bundle/truncated", "Truncated version");
        assert_eq!(decode_cpb_v1(&encoded_bytes[0..magic_len + version_len + payload_len_bytes - 1]).unwrap_err().tag, "pm/bundle/truncated", "Truncated payload length");
        assert_eq!(decode_cpb_v1(&encoded_bytes[0..magic_len + version_len + payload_len_bytes + payload_bytes - 1]).unwrap_err().tag, "pm/bundle/truncated", "Truncated payload");
        assert_eq!(decode_cpb_v1(&encoded_bytes[0..magic_len + version_len + payload_len_bytes + payload_bytes + cert_len_bytes - 1]).unwrap_err().tag, "pm/bundle/truncated", "Truncated certificate length");
        assert_eq!(decode_cpb_v1(&encoded_bytes[0..magic_len + version_len + payload_len_bytes + payload_bytes + cert_len_bytes + cert_bytes - 1]).unwrap_err().tag, "pm/bundle/truncated", "Truncated certificate");
    }

    #[test]
    fn test_reject_inconsistent_lengths() {
        let original_bundle = CertifiedProgramBundleV1 {
            format_version: CPB_VERSION_V1,
            payload: vec![1, 2, 3],
            certificate: vec![4, 5, 6],
        };
        let mut encoded_bytes = encode_cpb_v1(&original_bundle);
        // Add an extra byte at the end, making the total length inconsistent with declared lengths
        encoded_bytes.push(0xFF);

        let err = decode_cpb_v1(&encoded_bytes).unwrap_err();
        assert_eq!(err.tag, "pm/bundle/inconsistent-length");
    }
}
