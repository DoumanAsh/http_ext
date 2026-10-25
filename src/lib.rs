#![doc = include_str!("../README.md")]
//#![no_std]
#![warn(missing_docs)]
#![allow(clippy::style)]

extern crate alloc;

pub use http;

pub mod uri;
pub use uri::UriPathBuilder;
pub mod header;
pub use header::HeaderMapExt;
#[cfg(any(feature = "std", feature = "http-body"))]
pub mod body;

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
