# http_ext

[![Rust](https://github.com/DoumanAsh/http_ext/actions/workflows/rust.yml/badge.svg)](https://github.com/DoumanAsh/http_ext/actions/workflows/rust.yml)
[![Crates.io](https://img.shields.io/crates/v/http_ext.svg)](https://crates.io/crates/http_ext)
[![Documentation](https://docs.rs/http_ext/badge.svg)](https://docs.rs/crate/http_ext/)

Extensions to provide useful utilities for http crate

MSRV 1.85

## Feature

- `std` - Enables use of `std` library
- `http-body` - Enables use of `http-body` library

## Modules

### uri

Uri module provides utilities for working with [Uri](https://docs.rs/http/latest/http/uri/struct.Uri.html)

#### Extensions

- [UriBuilderExt](https://docs.rs/http_ext/latest/http_ext/uri/trait.UriBuilderExt.html) - Extension method to start building uri on variety of http types
- [RequestExt](https://docs.rs/http_ext/latest/http_ext/trait.RequestExt.html) - Request extension to modify existing uri

#### Path and query builder

Stress free and efficient builder for [path and query](https://docs.rs/http/latest/http/uri/struct.PathAndQuery.html)

```rust
use http_ext::uri::{self, UriPathBuilder};

//Manually escape
let query = (("key+1", "value+1"), ("key+2", "value+2"));
let mut builder = UriPathBuilder::new("/test", ()).join("get").with_query(());
builder.add_serde(&query).expect("to serialize");
let output = builder.try_into_http().expect("to finish building path and query");
assert_eq!(output.path(), "/test/get");
assert_eq!(output.query(), Some("key+1=value+1&key+2=value+2"));

//Use provided percent encoding
let query = (("key 1", "value 1"), ("key 2", "value 2"));
let mut builder = UriPathBuilder::new("/test", uri::PathEscape).join("ろり").with_query(uri::QueryEscape);
builder.add_serde(&query).expect("to serialize");
let output = builder.try_into_http().expect("to finish building path and query");
assert_eq!(output.path(), "/test/%E3%82%8D%E3%82%8A");
assert_eq!(output.query(), Some("key%201=value%201&key%202=value%202"));
```

### header

Extension module to [HeaderMap](https://docs.rs/http/latest/http/header/struct.HeaderMap.html)

#### Extensions

- [HeaderMapExt](https://docs.rs/http_ext/latest/http_ext/header/trait.HeaderMapExt.html) - Extension for [HeaderMap](https://docs.rs/http/latest/http/header/struct.HeaderMap.html)
- [HttpExt](https://docs.rs/http_ext/latest/http_ext/trait.RequestExt.html) - General extension trait provides extract of the [HeaderMapExt](https://docs.rs/http_ext/latest/http_ext/header/trait.HeaderMapExt.html)

### body

Provides utilities to work with http bodies through [http-body](https://docs.rs/http-body/latest/http_body/trait.Body.html) or std's [io::Read](https://doc.rust-lang.org/std/io/trait.Read.html)

#### Collect

[Collect](https://docs.rs/http_ext/latest/http_ext/body/struct.Collect.html) is an utility to efficiently collect body into custom accumulator

```rust
use core::future::Future;

use http_ext::body;

fn collect_body<B: body::http_body::Body>(body: B) ->
    impl Future<Output = Result<Vec<u8>, body::CollectError<B::Error, <Vec<u8> as body::Accum>::Error>>>
    where B::Data: AsRef<[u8]>
{
    //Creates future that accumulates body content into Vec
    //If body encrypted with zstd/gzip/zlib then automatically decode it when Identity is specified
    //Otherwise you can specify specific encoding to use only specific decompressor
    body::Collect::new(body, Vec::new()).with_encoding(body::ContentEncoding::Identity)
}
```
