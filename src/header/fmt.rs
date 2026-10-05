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
