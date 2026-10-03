# http_ext

[![Rust](https://github.com/DoumanAsh/http_ext/actions/workflows/rust.yml/badge.svg)](https://github.com/DoumanAsh/http_ext/actions/workflows/rust.yml)
[![Crates.io](https://img.shields.io/crates/v/http_ext.svg)](https://crates.io/crates/http_ext)
[![Documentation](https://docs.rs/http_ext/badge.svg)](https://docs.rs/crate/http_ext/)

Extensions to provide useful utilities for http crate

MSRV 1.85

## uri

Uri module provides utilities for working with [Uri](https://docs.rs/http/latest/http/uri/struct.Uri.html)

### Path and query builder

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
