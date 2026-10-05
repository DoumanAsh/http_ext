//! Header module

pub use http::{HeaderMap, HeaderName, HeaderValue};

mod fmt;
pub use fmt::InspectHeaders;

///Common header name for request id
pub const X_REQUEST_ID: HeaderName = HeaderName::from_static("x-request-id");

///Extension trait for [HeaderMap](https://docs.rs/http/latest/http/header/struct.HeaderMap.html)
///
///## Usage
///
///```
///use http_ext::header::{HeaderMap, HeaderMapExt, X_REQUEST_ID};
///
///let mut map = HeaderMap::new();
///
///assert_eq!(map.request_id_str(), None);
///
///map.insert(X_REQUEST_ID, "id".parse().unwrap());
///
///assert_eq!(map.request_id_str(), Some("id"));
///
///map.append(X_REQUEST_ID, "id2".parse().unwrap());
///
///assert_eq!(map.request_id_str(), Some("id"));
///
///assert_eq!(format!("{:?}", map.inspect_headers(&[&X_REQUEST_ID])), r##"{"x-request-id": id, id2}"##);
///```
pub trait HeaderMapExt<T> {
    ///Accesses [X_REQUEST_ID] value if any is present
    fn request_id(&self) -> Option<&T>;
    ///Accesses [X_REQUEST_ID] value as string if allowed
    fn request_id_str<'a>(&'a self) -> Option<&'a str> where T: 'a + AsRef<[u8]> {
        self.request_id().and_then(|value| core::str::from_utf8(value.as_ref()).ok())
    }

    ///Returns debug formatter to inspect specified list of `headers`
    fn inspect_headers<'a>(&'a self, headers: &'a [&'a http::HeaderName]) -> InspectHeaders<'a, T>;
}

impl<T> HeaderMapExt<T> for http::HeaderMap<T> {
    #[inline(always)]
    fn request_id(&self) -> Option<&T> {
        self.get(X_REQUEST_ID)
    }

    #[inline(always)]
    fn inspect_headers<'a>(&'a self, header_list: &'a [&'a http::HeaderName]) -> InspectHeaders<'a, T> {
        InspectHeaders {
            header_list,
            headers: self,
        }
    }
}
