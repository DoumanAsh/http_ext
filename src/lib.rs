#![doc = include_str!("../README.md")]
#![no_std]
#![warn(missing_docs)]
#![allow(clippy::style)]

extern crate alloc;

pub use http;

#[cfg(debug_assertions)]
macro_rules! unreach {
    () => {
        unreachable!()
    }
}

#[cfg(not(debug_assertions))]
macro_rules! unreach {
    () => {
        unsafe {
            ::core::hint::unreachable_unchecked()
        }
    }
}

pub mod uri;
pub use uri::{UriPathBuilder, UriBuilderExt};
pub mod header;
pub use header::HeaderMapExt;
#[cfg(feature = "body")]
pub mod body;
#[cfg(feature = "body")]
pub use body::Collect;

///Extension to all [http] types
pub trait HttpExt {
    ///Access request headers
    fn headers(&self) -> &http::HeaderMap;

    #[inline(always)]
    ///Accesses [header::X_REQUEST_ID] value if any is present
    fn request_id(&self) -> Option<&str> {
        self.headers().request_id_str()
    }

    #[inline(always)]
    ///Returns debug formatter to inspect specified list of `headers`
    fn inspect_headers<'a>(&'a self, headers: &'a [&'a http::HeaderName]) -> header::InspectHeaders<'a, http::HeaderValue> {
        self.headers().inspect_headers(headers)
    }

    #[inline(always)]
    ///Returns iterator [header::AcceptEncodingIter] over `Accept-Encoding` header values
    fn accept_encoding_iter<'a>(&'a self) -> header::AcceptEncodingIter<'a, http::HeaderValue> {
        self.headers().accept_encoding_iter()
    }
}

impl HttpExt for http::request::Parts {
    #[inline(always)]
    fn headers(&self) -> &http::HeaderMap {
        &self.headers
    }
}

impl<T> HttpExt for http::Request<T> {
    #[inline(always)]
    fn headers(&self) -> &http::HeaderMap {
        self.headers()
    }
}

impl<T> HttpExt for http::Response<T> {
    #[inline(always)]
    fn headers(&self) -> &http::HeaderMap {
        self.headers()
    }
}

///Extension to all [http::Request] types
pub trait RequestExt {
    ///Access request's uri for modification
    fn uri_mut(&mut self) -> &mut http::Uri;
    ///Access request headers
    fn headers(&self) -> &http::HeaderMap;
    ///Access request headers via mutable reference
    fn headers_mut(&mut self) -> &mut http::HeaderMap;

    #[inline(always)]
    ///Start building path using initial `path` from [http::Request::uri]
    ///
    ///If you need to guarantee to build valid path then use [RequestExt::build_path_escaped]
    ///
    ///```
    ///use http_ext::{uri, RequestExt};
    /////Build as part of request
    ///let mut request = http::Request::new(());
    ///request.build_path()
    ///       .join_path("api")
    ///       .join_path("v1")
    ///       .join_path("logs")
    ///       //Start query components
    ///       .build_query(uri::QueryEscape)
    ///       .add_kv_query("test 1", "value 1")
    ///       .finish_path()
    ///       .expect("to build uri");
    ///assert_eq!(request.uri().path(), "/api/v1/logs");
    ///assert_eq!(request.uri().query(), Some("test%201=value%201"));
    ///```
    fn build_path(&mut self) -> uri::UriBuilderPathBuild<&mut http::Uri, uri::UriPathBuilder<()>> {
        self.uri_mut().build_path(())
    }

    #[inline(always)]
    ///Start building path using initial `path` from [http::Request::uri] while escaping characters
    fn build_path_escaped(&mut self) -> uri::UriBuilderPathBuild<&mut http::Uri, uri::UriPathBuilder<uri::PathEscape>> {
        self.uri_mut().build_path_escaped()
    }

    #[inline(always)]
    ///Start building path using initial `path` from [http::Request::uri] without escaping characters
    fn build_path_unescaped(&mut self) -> uri::UriBuilderPathBuild<&mut http::Uri, UriPathBuilder<()>> {
        self.uri_mut().build_path_unescaped()
    }

    #[inline(always)]
    ///Accesses [header::X_REQUEST_ID] value if any is present
    fn request_id(&self) -> Option<&str> {
        self.headers().request_id_str()
    }

    #[inline(always)]
    ///Returns debug formatter to inspect specified list of `headers`
    fn inspect_headers<'a>(&'a self, headers: &'a [&'a http::HeaderName]) -> header::InspectHeaders<'a, http::HeaderValue> {
        self.headers().inspect_headers(headers)
    }

    ///Sets `Accept-Encoding` header with what available in this crate by default
    ///
    ///If `body` feature is not enabled, then no header is set (meaning identity encoding is the only one supported)
    fn set_supported_accept_encoding(&mut self) {
        #[cfg(feature = "body")]
        {
            const VALUE: http::HeaderValue = http::HeaderValue::from_static("zstd,gzip");

            self.headers_mut().insert(http::header::ACCEPT_ENCODING, VALUE);
        }
    }

    #[inline(always)]
    ///Returns iterator [header::AcceptEncodingIter] over `Accept-Encoding` header values
    fn accept_encoding_iter<'a>(&'a self) -> header::AcceptEncodingIter<'a, http::HeaderValue> {
        self.headers().accept_encoding_iter()
    }
}

impl RequestExt for http::request::Parts {
    #[inline(always)]
    fn uri_mut(&mut self) -> &mut http::Uri {
        &mut self.uri
    }
    #[inline(always)]
    fn headers(&self) -> &http::HeaderMap {
        &self.headers
    }
    #[inline(always)]
    fn headers_mut(&mut self) -> &mut http::HeaderMap {
        &mut self.headers
    }
}

impl<T> RequestExt for http::Request<T> {
    #[inline(always)]
    fn uri_mut(&mut self) -> &mut http::Uri {
        http::Request::uri_mut(self)
    }
    #[inline(always)]
    fn headers(&self) -> &http::HeaderMap {
        self.headers()
    }
    #[inline(always)]
    fn headers_mut(&mut self) -> &mut http::HeaderMap {
        self.headers_mut()
    }
}
