use crate::errors::PmError;
use crate::bundle::{CertifiedProgramBundleV1, CPB_VERSION_V1}; // Imported CPB_VERSION_V1

/// Placeholder marker for the start of a certificate.
pub const PMCERT1_MARKER: &[u8] = b"PMCERT1:";

/// Verifies a CertifiedProgramBundleV1 according to MVP rules.
pub fn verify_bundle(bundle: &CertifiedProgramBundleV1) -> Result<(), PmError> {
    // MVP check 1: Require format_version == CPB_VERSION_V1
    if bundle.format_version != CPB_VERSION_V1 {
        return Err(PmError::new("pm/bundle/bad-version", format!("Unsupported bundle format version: {}. Expected {}.", bundle.format_version, CPB_VERSION_V1)));
    }

    // MVP check 2: Require certificate non-empty
    if bundle.certificate.is_empty() {
        return Err(PmError::new("pm/verify/empty-certificate", "Certificate is empty."));
    }

    // MVP check 3: Require payload non-empty
    if bundle.payload.is_empty() {
        return Err(PmError::new("pm/verify/empty-payload", "Payload is empty."));
    }

    // MVP check 4 (Optional but good for MVP): Require certificate begins with ASCII PMCERT1:
    if !bundle.certificate.starts_with(PMCERT1_MARKER) {
        return Err(PmError::new("pm/verify/bad-certificate-marker", "Certificate does not start with the required marker."));
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::bundle::{encode_cpb_v1, CertifiedProgramBundleV1, CPB_VERSION_V1};

    #[test]
    fn test_verify_success() {
        let bundle = CertifiedProgramBundleV1 {
            format_version: CPB_VERSION_V1,
            payload: vec![1, 2, 3],
            certificate: PMCERT1_MARKER.to_vec(),
        };
        assert!(verify_bundle(&bundle).is_ok());
    }

    #[test]
    fn test_verify_bad_version() {
        // Use a version different from CPB_VERSION_V1 for the test
        let bad_version = CPB_VERSION_V1 + 1;
        let bundle = CertifiedProgramBundleV1 {
            format_version: bad_version, // Invalid version
            payload: vec![1, 2, 3],
            certificate: PMCERT1_MARKER.to_vec(),
        };
        let err = verify_bundle(&bundle).unwrap_err();
        assert_eq!(err.tag, "pm/bundle/bad-version");
        // Checking that the error message includes the expected version
        assert!(err.details.contains(&format!("Expected {}", CPB_VERSION_V1)));
    }

    #[test]
    fn test_verify_empty_certificate() {
        let bundle = CertifiedProgramBundleV1 {
            format_version: CPB_VERSION_V1,
            payload: vec![1, 2, 3],
            certificate: vec![], // Empty certificate
        };
        let err = verify_bundle(&bundle).unwrap_err();
        assert_eq!(err.tag, "pm/verify/empty-certificate");
    }

    #[test]
    fn test_verify_empty_payload() {
        let bundle = CertifiedProgramBundleV1 {
            format_version: CPB_VERSION_V1,
            payload: vec![], // Empty payload
            certificate: PMCERT1_MARKER.to_vec(),
        };
        let err = verify_bundle(&bundle).unwrap_err();
        assert_eq!(err.tag, "pm/verify/empty-payload");
    }

    #[test]
    fn test_verify_bad_certificate_marker() {
        let bundle = CertifiedProgramBundleV1 {
            format_version: CPB_VERSION_V1,
            payload: vec![1, 2, 3],
            certificate: b"NOT_PMCERT1:".to_vec(), // Incorrect marker
        };
        let err = verify_bundle(&bundle).unwrap_err();
        assert_eq!(err.tag, "pm/verify/bad-certificate-marker");
    }

    #[test]
    fn test_verify_with_full_bundle() {
        // Test verify function using encoded/decoded bundle
        let original_bundle = CertifiedProgramBundleV1 {
            format_version: CPB_VERSION_V1,
            payload: vec![1, 2, 3, 4, 5],
            certificate: PMCERT1_MARKER.to_vec().into_iter().chain(b"some_cert_data".iter().cloned()).collect(),
        };
        let encoded_bytes = encode_cpb_v1(&original_bundle);
        let decoded_bundle = crate::bundle::decode_cpb_v1(&encoded_bytes).unwrap();

        // Verify the decoded bundle
        assert!(verify_bundle(&decoded_bundle).is_ok());

        // Test verification failure with a bad version in the decoded bundle
        let mut bad_version_bundle = decoded_bundle.clone();
        bad_version_bundle.format_version = CPB_VERSION_V1 + 1; // Use a version different from CPB_VERSION_V1
        let err_version = verify_bundle(&bad_version_bundle).unwrap_err();
        assert_eq!(err_version.tag, "pm/bundle/bad-version");

        // Test verification failure with an empty certificate
        let mut empty_cert_bundle = decoded_bundle.clone();
        empty_cert_bundle.certificate = vec![];
        let err_cert = verify_bundle(&empty_cert_bundle).unwrap_err();
        assert_eq!(err_cert.tag, "pm/verify/empty-certificate");

        // Test verification failure with an empty payload
        let mut empty_payload_bundle = decoded_bundle.clone();
        empty_payload_bundle.payload = vec![];
        let err_payload = verify_bundle(&empty_payload_bundle).unwrap_err();
        assert_eq!(err_payload.tag, "pm/verify/empty-payload");

        // Test verification failure with a bad certificate marker
        let mut bad_marker_bundle = decoded_bundle.clone();
        bad_marker_bundle.certificate = b"NOT_PMCERT1:some_cert_data".to_vec();
        let err_marker = verify_bundle(&bad_marker_bundle).unwrap_err();
        assert_eq!(err_marker.tag, "pm/verify/bad-certificate-marker");
    }
}