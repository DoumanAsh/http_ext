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
    ///Start building path using initial `path` from [http::Request::uri]
    fn build_path_escaped(&mut self) -> uri::UriBuilderPathBuild<&mut http::Uri, uri::UriPathBuilder<uri::PathEscape>> {
        self.uri_mut().build_path(uri::PathEscape)
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
}
