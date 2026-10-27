use http_ext::{uri, RequestExt};
#[cfg(feature = "body")]
use http_ext::body;

mod serde;

#[test]
fn should_build_plain_query() {
    let mut builder = uri::UriPathBuilder::new("/test", uri::PathEscape).join("ろり").with_query(uri::QueryEscape);
    builder.add_kv("key 1", "value 1")
           .add_kv("key 2", "value 2");
    assert_eq!(builder.as_str(), "/test/%E3%82%8D%E3%82%8A?key%201=value%201&key%202=value%202");
    let output = builder.try_into_http().expect("to finish building path and query");
    assert_eq!(output.path(), "/test/%E3%82%8D%E3%82%8A");
    assert_eq!(output.query(), Some("key%201=value%201&key%202=value%202"));

    let mut builder = uri::UriPathBuilder::new("/test", ()).join("ろり").with_query(());
    builder.add_kv("key+1", "value+1")
           .add_kv("key+2", "value+2");
    assert_eq!(builder.as_str(), "/test/ろり?key+1=value+1&key+2=value+2");
    let output = builder.try_into_http().expect("to finish building path and query");
    assert_eq!(output.path(), "/test/ろり");
    assert_eq!(output.query(), Some("key+1=value+1&key+2=value+2"));

    //Ensure empty path components are collapsed
    let mut builder = uri::UriPathBuilder::new("/", ()).join_many(&["", "cute", "is", "justice"]).join("").with_query(());
    builder.add_kv("key+1", "value+1")
           .add_kv("key+2", "value+2");
    assert_eq!(builder.as_str(), "/cute/is/justice/?key+1=value+1&key+2=value+2");
    let output = builder.try_into_http().expect("to finish building path and query");
    assert_eq!(output.path(), "/cute/is/justice/");
    assert_eq!(output.query(), Some("key+1=value+1&key+2=value+2"));
}

#[test]
fn should_process_accept_encoding() {
    const VALUE: http::HeaderValue = http::HeaderValue::from_static("zstd;, gzip;q=0.8, *;q=0.1");
    let mut req = http::Request::new(());
    req.headers_mut().insert(http::header::ACCEPT_ENCODING, VALUE);

    let result = req.accept_encoding_iter().collect::<Vec<_>>();
    assert_eq!(result.len(), 3);

    assert_eq!(result[0].0, "zstd");
    assert_eq!(result[0].1, 1.0);
    assert_eq!(result[1].0, "gzip");
    assert_eq!(result[1].1, 0.8);
    assert_eq!(result[2].0, "*");
    assert_eq!(result[2].1, 0.1);
}

#[cfg(feature = "std")]
#[test]
fn should_read_plain_body() {
    let body = "123456789".repeat(1024);

    let mut accum = Vec::new();
    let mut collector = body::Collect::new(body.as_bytes(), accum).with_size_limit(body::BytesLimit(body.len()));
    let result = collector.read_all().expect("to read");
    assert!(body.as_bytes() == result, "Expected result with size={} but got size={}", body.len(), result.len());

    accum = result;
    accum.clear();
    let mut collector = body::Collect::new(body.as_bytes(), accum).with_size_limit(body::BytesLimit(body.len() - 1));
    let error = collector.read_all().expect_err("to overflow");
    assert!(matches!(error, body::CollectError::Overflow));
}

#[cfg(feature = "http-body")]
#[test]
fn should_read_plain_http_body() {
    let body = "123456789".repeat(1024);
    let body_len = body.len();

    let mut accum = Vec::new();
    let collector = body::Collect::new(body.clone(), accum).with_size_limit(body::BytesLimit(body_len));
    let result = thread_waker::block_on(collector).expect("to read");
    assert!(body.as_bytes() == result, "Expected result with size={} but got size={}", body.len(), result.len());

    accum = Vec::new();
    accum.clear();

    let collector = body::Collect::new(body, accum).with_size_limit(body::BytesLimit(body_len - 1));
    let error = thread_waker::block_on(collector).expect_err("to overflow");
    assert!(matches!(error, body::CollectError::Overflow));
}

#[cfg_attr(miri, ignore)]
#[cfg(feature = "std")]
#[test]
fn should_auto_read_zstd_body() {
    use compu::Encoder;

    let body = "123456789".repeat(1024);
    let mut encoder = compu::encoder::ZstdC::new(Default::default()).expect("to create zstd encoder");
    let mut compressed_body = Vec::new();
    encoder.encode_vec_full(body.as_bytes(), &mut compressed_body, compu::EncodeOp::Finish).expect("To compress");

    let mut accum = Vec::new();
    let mut collector = body::Collect::new(compressed_body.as_slice(), accum).with_size_limit(body::BytesLimit(body.len()));
    let result = collector.read_all().expect("to read");
    assert!(body.as_bytes() == result, "Expected result with size={} but got size={}", body.len(), result.len());

    accum = result;
    accum.clear();
    let mut collector = body::Collect::new(compressed_body.as_slice(), accum).with_size_limit(body::BytesLimit(body.len() - 1));
    let error = collector.read_all().expect_err("to overflow");
    assert!(matches!(error, body::CollectError::Overflow));
}

#[cfg(feature = "std")]
#[test]
fn should_auto_read_gzip_body() {
    use compu::Encoder;

    let body = "123456789".repeat(1024);
    let mut encoder = compu::encoder::ZlibRust::new(compu::encoder::ZlibOptions::new().mode(compu::encoder::ZlibMode::Gzip)).expect("to create zlib encoder");
    let mut compressed_body = Vec::new();
    encoder.encode_vec_full(body.as_bytes(), &mut compressed_body, compu::EncodeOp::Finish).expect("To compress");

    let mut accum = Vec::new();
    let mut collector = body::Collect::new(compressed_body.as_slice(), accum).with_size_limit(body::BytesLimit(body.len()));
    let result = collector.read_all().expect("to read");
    assert!(body.as_bytes() == result, "Expected result with size={} but got size={}", body.len(), result.len());

    accum = result;
    accum.clear();
    let mut collector = body::Collect::new(compressed_body.as_slice(), accum).with_size_limit(body::BytesLimit(body.len() - 1));
    let error = collector.read_all().expect_err("to overflow");
    assert!(matches!(error, body::CollectError::Overflow));
}

#[cfg(feature = "std")]
#[test]
fn should_auto_read_zlib_body() {
    use compu::Encoder;

    let body = "123456789".repeat(1024);
    let mut encoder = compu::encoder::ZlibRust::new(compu::encoder::ZlibOptions::new().mode(compu::encoder::ZlibMode::Zlib)).expect("to create zlib encoder");
    let mut compressed_body = Vec::new();
    encoder.encode_vec_full(body.as_bytes(), &mut compressed_body, compu::EncodeOp::Finish).expect("To compress");

    let mut accum = Vec::new();
    let mut collector = body::Collect::new(compressed_body.as_slice(), accum).with_size_limit(body::BytesLimit(body.len()));
    let result = collector.read_all().expect("to read");
    assert!(body.as_bytes() == result, "Expected result with size={} but got size={}", body.len(), result.len());

    accum = Vec::new();
    accum.clear();
    let mut collector = body::Collect::new(compressed_body.as_slice(), accum).with_size_limit(body::BytesLimit(body.len() - 1));
    let error = collector.read_all().expect_err("to overflow");
    assert!(matches!(error, body::CollectError::Overflow));
}

#[cfg(feature = "std")]
#[test]
fn should_fail_auto_read_deflate_body() {
    use compu::Encoder;

    let body = "123456789".repeat(1024);
    let mut encoder = compu::encoder::ZlibRust::new(compu::encoder::ZlibOptions::new().mode(compu::encoder::ZlibMode::Deflate)).expect("to create zlib encoder");
    let mut compressed_body = Vec::new();
    encoder.encode_vec_full(body.as_bytes(), &mut compressed_body, compu::EncodeOp::Finish).expect("To compress");

    let accum = Vec::new();
    let mut collector = body::Collect::new(compressed_body.as_slice(), accum).with_size_limit(body::BytesLimit(body.len()));
    let result = collector.read_all().expect("to read");
    assert!(body.as_bytes() != compressed_body, "Expected still compressed result with size={} but got size={}", compressed_body.len(), result.len());
}
