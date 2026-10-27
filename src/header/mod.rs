//! Header module

pub use http::{HeaderMap, HeaderName, HeaderValue};
use http::header::ACCEPT_ENCODING;

mod fmt;
pub use fmt::{InspectHeaders, HeaderValueFmtBuffer};

///Common header name for request id
pub const X_REQUEST_ID: HeaderName = HeaderName::from_static("x-request-id");

///Iterator over `Accept-Encoding` returning pairs of `(&str, f32)` which represents name of the encoding and requested quality (default quality is 1.0)
pub struct AcceptEncodingIter<'a, T> {
    headers: http::header::ValueIter<'a, T>,
    cursor: Option<core::str::Split<'a, char>>
}

impl<'a, T: AsRef<[u8]>> Iterator for AcceptEncodingIter<'a, T> {
    type Item = (&'a str, f32);

    #[inline(always)]
    fn next(&mut self) -> Option<Self::Item> {
        #[inline(always)]
        fn extract_quality_value(value: &str) -> Option<f32> {
            value.strip_prefix("q=").or_else(|| value.strip_prefix("Q=")).and_then(|value| value.trim().parse().ok())
        }

        'iter: loop {
            match &mut self.cursor {
                Some(iter) => match iter.next() {
                    Some(value) => {
                        let mut value = value.splitn(2, ';');
                        //First element in split always returns
                        let encoding = match value.next() {
                            Some(encoding) => encoding.trim(),
                            None => unreach!()
                        };
                        let quality = value.next().and_then(extract_quality_value).unwrap_or(1.0f32);
                        break 'iter Some((encoding, quality));
                    },
                    None => {
                        self.cursor.take();
                        continue 'iter;
                    }
                },
                None => {
                    let value = 'headers: loop {
                        let value = self.headers.next()?;
                        match core::str::from_utf8(value.as_ref()) {
                            Ok(value) => break 'headers value.trim(),
                            Err(_) => continue 'headers,
                        }
                    };

                    if value.is_empty() {
                        continue 'iter;
                    }

                    self.cursor = Some(value.split(','));
                }
            }
        }
    }
}

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
    ///Returns iterator [AcceptEncodingIter] over `Accept-Encoding` header values
    fn accept_encoding_iter<'a>(&'a self) -> AcceptEncodingIter<'a, T> where T: 'a + AsRef<[u8]>;
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

    #[inline(always)]
    fn accept_encoding_iter<'a>(&'a self) -> AcceptEncodingIter<'a, T> where T: 'a + AsRef<[u8]> {
        AcceptEncodingIter {
            headers: self.get_all(ACCEPT_ENCODING).into_iter(),
            cursor: None
        }
    }
}
