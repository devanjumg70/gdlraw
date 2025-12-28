//! HTTP Body Coverage Tests
//!
//! Covers:
//! - `RequestBody` conversions and methods
//! - `ResponseBody` API surface (compilation check and structure)

use bytes::Bytes;
use chromenet::http::RequestBody;
use http_body::Body;

#[test]
fn test_request_body_empty_methods() {
    let mut body = RequestBody::Empty;
    assert!(body.is_empty());
    assert_eq!(body.len(), 0);
    assert!(body.take_bytes().is_empty());
}

#[test]
fn test_request_body_bytes_methods() {
    let data = Bytes::from("hello world");
    let mut body = RequestBody::Bytes(data.clone());

    assert!(!body.is_empty());
    assert_eq!(body.len(), 11);

    let taken = body.take_bytes();
    assert_eq!(taken, data);
    assert!(body.is_empty()); // Should be empty after take
}

#[test]
fn test_request_body_conversions() {
    // From String
    let b1: RequestBody = String::from("test").into();
    assert_eq!(b1.len(), 4);

    // From &str
    let b2: RequestBody = "test".into();
    assert_eq!(b2.len(), 4);

    // From Vec<u8>
    let b3: RequestBody = vec![1, 2, 3].into();
    assert_eq!(b3.len(), 3);

    // From Bytes
    let b4: RequestBody = Bytes::from("test").into();
    assert_eq!(b4.len(), 4);

    // From &[u8]
    let b5: RequestBody = b"test".as_slice().into();
    assert_eq!(b5.len(), 4);
}

#[test]
fn test_request_body_into_full() {
    let body = RequestBody::from("test");
    let full = body.into_full();
    assert_eq!(full.size_hint().exact(), Some(4));
}

#[test]
fn test_request_body_wrapper() {
    // This wrapper is used internally for hyper compatibility
    use chromenet::http::requestbody::BodyWrapper;

    let body = RequestBody::from("test");
    let wrapper: BodyWrapper = body.into();
    assert_eq!(wrapper.size_hint().exact(), Some(4));
    assert!(!wrapper.is_end_stream());
}

#[test]
fn test_request_body_empty_wrapper() {
    use chromenet::http::requestbody::BodyWrapper;

    let body = RequestBody::Empty;
    let wrapper: BodyWrapper = body.into();
    assert_eq!(wrapper.size_hint().exact(), Some(0));
    // It might be conceptually ended, but implementation details vary
    // Checking strict size is enough
}

#[test]
fn test_response_body_api_check() {
    // We can't easily construct a real ResponseBody without a network connection
    // or deep mocking of hyper::Incoming type.
    // Just verify the type exists and is accessible.
    use chromenet::http::responsebody::BodyStream;

    // Compile-time check that BodyStream implements Stream
    fn assert_stream<S: futures::Stream>() {}
    assert_stream::<BodyStream>();
}
