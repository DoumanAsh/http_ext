use http_ext::body;


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
    let mut encoder = compu::encoder::ZlibRust::new(compu::encoder::ZlibOptions::new().mode(compu::encoder::ZlibMode::Gzip)).expect("to create zstd encoder");
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
    let mut encoder = compu::encoder::ZlibRust::new(compu::encoder::ZlibOptions::new().mode(compu::encoder::ZlibMode::Zlib)).expect("to create zstd encoder");
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
    let mut encoder = compu::encoder::ZlibRust::new(compu::encoder::ZlibOptions::new().mode(compu::encoder::ZlibMode::Deflate)).expect("to create zstd encoder");
    let mut compressed_body = Vec::new();
    encoder.encode_vec_full(body.as_bytes(), &mut compressed_body, compu::EncodeOp::Finish).expect("To compress");

    let accum = Vec::new();
    let mut collector = body::Collect::new(compressed_body.as_slice(), accum).with_size_limit(body::BytesLimit(body.len()));
    let result = collector.read_all().expect("to read");
    assert!(body.as_bytes() != compressed_body, "Expected still compressed result with size={} but got size={}", compressed_body.len(), result.len());
}
