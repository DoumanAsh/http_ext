use core::{str, fmt};

struct DisplayHeaderValues<'a, T=http::HeaderValue>(http::header::GetAll<'a, T>);

impl<T: AsRef<[u8]>> fmt::Debug for DisplayHeaderValues<'_, T> {
    fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
        const FALLBACK_STR: &str = "<non-utf8>";

        let mut headers = self.0.iter();
        if let Some(header) = headers.next() {
            match str::from_utf8(header.as_ref()) {
                Ok(header) => fmt.write_str(header)?,
                Err(_) => fmt.write_str(FALLBACK_STR)?,
            }

            for header in headers {
                fmt.write_str(", ")?;
                match str::from_utf8(header.as_ref()) {
                    Ok(header) => fmt.write_str(header)?,
                    Err(_) => fmt.write_str(FALLBACK_STR)?,
                }
            }
        }

        Ok(())
    }
}

///Utility formatter to inspect specific headers within [HeaderMap](https://docs.rs/http/latest/http/header/struct.HeaderMap.html) via `Debug` implementation
pub struct InspectHeaders<'a, T=http::HeaderValue> {
    ///List of header names to inspect
    pub header_list: &'a [&'a http::HeaderName],
    ///HeaderMap under inspection
    pub headers: &'a http::HeaderMap<T>,
}

impl<'a, T> InspectHeaders<'a, T> {
    #[inline(always)]
    ///Returns iterator over pairs of header key and value iterator.
    pub fn iter(&self) -> impl Iterator<Item = (&'a http::HeaderName, http::header::GetAll<'a, T>)> + 'a {
        self.header_list.iter().map(|key| (*key, self.headers.get_all(*key)))
    }
}

impl<T: AsRef<[u8]>> fmt::Debug for InspectHeaders<'_, T> {
    fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut out = fmt.debug_map();
        for (key, all_values) in self.iter() {
            if all_values.iter().next().is_some() {
                out.entry(&key.as_str(), &DisplayHeaderValues(all_values));
            }
        }

        out.finish()
    }
}

///String only [http::HeaderValue] buffer
///
///This buffer implements `fmt::Write` interface that restricts buffer to valid ASCII characters
///only, rejecting any invalid write.
///
///As long as you're source of the data though, you can safely ignore write errors as you would
///know correct .
pub struct HeaderValueFmtBuffer(bytes::BytesMut);

impl HeaderValueFmtBuffer {
    #[inline(always)]
    ///Creates new buffer
    pub fn new() -> Self {
        Self(bytes::BytesMut::new())
    }

    #[inline(always)]
    ///Checks if specified string is valid for use in self.
    pub fn is_valid_str(text: &str) -> bool {
        #[inline]
        fn is_visible_ascii(b: u8) -> bool {
            b >= 32 && b < 127 || b == b'\t'
        }

        let text = text.as_bytes();
        let mut is_bad = false;
        for &b in text {
            is_bad |= !is_visible_ascii(b);
        }

        is_bad
    }

    #[inline(always)]
    ///Writes fmt arguments created by `fmt::format_args` into self
    ///
    ///```rust
    ///use http_ext::http::header::{self, HeaderMap};
    ///use http_ext::header::HeaderValueFmtBuffer;
    ///
    /////Create simple value
    ///let token = "my super secret token";
    ///let value = HeaderValueFmtBuffer::new().format(format_args!("Bearer {token}")).expect("to generate auth header");
    ///assert_eq!(value.as_str(), "Bearer my super secret token");
    ///
    /////Ready to be inserted into header map
    ///let mut headers = HeaderMap::new();
    ///headers.insert(header::AUTHORIZATION, value.into());
    ///```
    pub fn format(mut self, args: fmt::Arguments<'_>) -> Result<Self, fmt::Error> {
        fmt::Write::write_fmt(&mut self, args).map(|_| self)
    }

    #[inline(always)]
    ///Returns textual representation of the header value
    pub fn as_str(&self) -> &str {
        match str::from_utf8(self.0.as_ref()) {
            Ok(value) => value,
            Err(_) => unreach!()
        }
    }

    #[inline(always)]
    ///Extracts written bytes out of self
    pub fn take(&mut self) -> bytes::Bytes {
        self.0.split().freeze()
    }

    #[inline(always)]
    ///Extracts written bytes out of self leaving self empty
    pub fn take_header_value(&mut self) -> http::HeaderValue {
        Self(self.0.split()).into()
    }
}

impl Into<http::HeaderValue> for HeaderValueFmtBuffer {
    #[inline]
    fn into(self) -> http::HeaderValue {
        //We cannot fail creating header value because we guarantee that only valid UTF-8 is written
        unsafe {
            http::HeaderValue::from_maybe_shared_unchecked(self.0.freeze())
        }
    }
}

impl fmt::Write for HeaderValueFmtBuffer {
    #[inline]
    fn write_str(&mut self, text: &str) -> fmt::Result {
        if Self::is_valid_str(text) {
            Err(fmt::Error)
        } else {
            self.0.extend_from_slice(text.as_bytes());
            Ok(())
        }
    }
}
