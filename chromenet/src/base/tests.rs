use crate::base::neterror::NetError;

#[test]
fn test_net_error_roundtrip() {
    // Standard Chromium error
    let original = NetError::ConnectionRefused;
    let code = original.as_i32();
    assert_eq!(code, -102);
    let converted = NetError::from(code);
    assert!(matches!(converted, NetError::ConnectionRefused));

    // Custom error
    let custom = NetError::RedirectCycleDetected;
    let custom_code = custom.as_i32();
    assert_eq!(custom_code, -10000);
    let custom_converted = NetError::from(custom_code);
    assert!(matches!(custom_converted, NetError::RedirectCycleDetected));
}

#[test]
fn test_unknown_error() {
    let err = NetError::from(-9999);
    assert!(matches!(err, NetError::Unknown(-9999)));
}

#[test]
fn test_collision_avoidance() {
    // Verify that we are not using the Blob error range (-900 to -906)
    // defined in Chromium's net_error_list.h
    let blob_range = -906..=-900;

    let redirect_error = NetError::RedirectCycleDetected;
    assert!(!blob_range.contains(&redirect_error.as_i32()));
}
