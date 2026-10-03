//! URI utilities

mod serde;

mod encodings {
    pub const PATH: percent_encoding::AsciiSet = percent_encoding::CONTROLS.add(b' ')
                                                                           .add(b'"')
                                                                           .add(b'\'')
                                                                           .add(b'\\')
                                                                           .add(b'^')
                                                                           .add(b'<')
                                                                           .add(b'>')
                                                                           .add(b'`')
                                                                           .add(b'#')
                                                                           .add(b'?')
                                                                           .add(b'{')
                                                                           .add(b'}')
                                                                           .add(b'%')
                                                                           .add(b'[')
                                                                           .add(b']')
                                                                           .add(b'|');
    pub const QUERY: percent_encoding::AsciiSet = PATH.add(b'$')
                                                      .add(b'%')
                                                      .add(b'&')
                                                      .add(b'+')
                                                      .add(b',')
                                                      .add(b'/')
                                                      .add(b':')
                                                      .add(b';')
                                                      .add(b'=')
                                                      .add(b'?')
                                                      .add(b'@');
}

///Interface for path parameter encoding
pub trait PathEncoding {
    ///Encodes given `value` returning iterator of strings to collect/write
    fn encode(value: &str) -> impl Iterator<Item = &str>;
}

impl PathEncoding for () {
    #[inline(always)]
    fn encode(value: &str) -> impl Iterator<Item = &str> {
        [value].into_iter()
    }
}

#[derive(Copy, Clone)]
///[PathEncoding] to escape invalid characters within url's path
pub struct PathEscape;

impl PathEncoding for PathEscape {
    #[inline(always)]
    fn encode(value: &str) -> impl Iterator<Item = &str> {
        percent_encoding::utf8_percent_encode(value, &encodings::PATH)

    }
}

///Interface for query parameter encoding
pub trait QueryEncoding {
    ///Encodes given `value` returning iterator of strings to collect/write
    fn encode(value: &str) -> impl Iterator<Item = &str>;
}

impl QueryEncoding for () {
    #[inline(always)]
    fn encode(value: &str) -> impl Iterator<Item = &str> {
        [value].into_iter()
    }
}

#[derive(Copy, Clone)]
///[QueryEncoding] to escape invalid characters within url's query parameter list
pub struct QueryEscape;

impl QueryEncoding for QueryEscape {
    #[inline(always)]
    fn encode(value: &str) -> impl Iterator<Item = &str> {
        percent_encoding::utf8_percent_encode(value, &encodings::QUERY)
    }
}

///Efficient builder for [Uri](https://docs.rs/http/latest/http/uri/struct.Uri.html)'s path
pub struct UriPathBuilder<P> {
    output: bytes::BytesMut,
    #[allow(unused)]
    encoder: P,
}

impl<P: PathEncoding> UriPathBuilder<P> {
    #[inline]
    ///Creates new Uri's path builder
    pub fn new(path: &str, encoder: P) -> Self {
        Self {
            output: bytes::BytesMut::new(),
            encoder,
        }.join(path)
    }

    #[inline]
    ///Join `self` with specified `path`
    pub fn join(mut self, path: &str) -> Self {
        let path = path.strip_prefix('/').unwrap_or(path);
        self.output.reserve(path.len().saturating_add(1));
        if !self.output.ends_with(b"/") {
            self.output.extend_from_slice(b"/");
        }
        for path in P::encode(path) {
            self.output.extend_from_slice(path.as_bytes());
        }
        self
    }

    #[inline(always)]
    ///Finalizes builder
    pub fn try_into_http(self) -> Result<http::uri::PathAndQuery, http::uri::InvalidUri> {
        http::uri::PathAndQuery::from_maybe_shared(self.output)
    }

    #[inline]
    ///Starts building query part of the Uri using provided [encoder](QueryEncoding)
    pub fn with_query<Q: QueryEncoding>(self, encoder: Q) -> UriPathQueryBuilder<Q> {
        UriPathQueryBuilder::new(self, encoder)
    }
}

impl<P: PathEncoding> core::ops::Add<&str> for UriPathBuilder<P> {
    type Output = UriPathBuilder<P>;

    #[inline(always)]
    fn add(self, rhs: &str) -> Self::Output {
        self.join(rhs)
    }
}

///Efficient builder for [Uri](https://docs.rs/http/latest/http/uri/struct.Uri.html)'s query
pub struct UriPathQueryBuilder<Q> {
    output: bytes::BytesMut,
    len: usize,
    #[allow(unused)]
    encoder: Q,
}

impl<Q: QueryEncoding> UriPathQueryBuilder<Q> {
    #[inline]
    ///Creates new URI's query builder
    pub fn new(path: UriPathBuilder<impl PathEncoding>, encoder: Q) -> Self {
        Self {
            output: path.output,
            len: 0,
            encoder,
        }
    }

    fn add_key(&mut self, key: &str) -> &mut Self {
        self.output.reserve(key.len().saturating_add(1));

        if self.len == 0 {
            self.output.extend_from_slice(b"?");
        } else {
            self.output.extend_from_slice(b"&");
        }

        for component in Q::encode(key) {
            self.output.extend_from_slice(component.as_bytes());
        }

        self.len = self.len.saturating_add(1);
        self
    }

    fn add_value(&mut self, value: &str) -> &mut Self {
        self.output.reserve(value.len().saturating_add(1));

        self.output.extend_from_slice(b"=");

        for component in Q::encode(value) {
            self.output.extend_from_slice(component.as_bytes());
        }

        self
    }

    ///Adds `key` and `value` to the query parameters
    pub fn add_kv(&mut self, key: &str, value: &str) -> &mut Self {
        self.output.reserve(key.len().saturating_add(value.len()).saturating_add(2));

        if self.len == 0 {
            self.output.extend_from_slice(b"?");
        } else {
            self.output.extend_from_slice(b"&");
        }

        for component in Q::encode(key) {
            self.output.extend_from_slice(component.as_bytes());
        }

        self.output.extend_from_slice(b"=");

        for component in Q::encode(value) {
            self.output.extend_from_slice(component.as_bytes());
        }

        self.len = self.len.saturating_add(1);
        self
    }

    #[inline(always)]
    ///Adds pairs of key and values from `object`
    ///
    ///`object` must map/struct or sequence/tuple of pairs.
    pub fn add_serde(&mut self, object: impl serde::Serialize) -> Result<&mut Self, serde::Error> {
        object.serialize(serde::QueryVisitor::new(self))?;
        Ok(self)
    }

    #[inline(always)]
    ///Finalizes builder
    pub fn try_into_http(self) -> Result<http::uri::PathAndQuery, http::uri::InvalidUri> {
        http::uri::PathAndQuery::from_maybe_shared(self.output)
    }
}

impl<P: PathEncoding> TryFrom<UriPathBuilder<P>> for http::uri::PathAndQuery {
    type Error = http::uri::InvalidUri;

    #[inline(always)]
    fn try_from(value: UriPathBuilder<P>) -> Result<Self, Self::Error> {
        value.try_into_http()
    }
}

impl<Q: QueryEncoding> TryFrom<UriPathQueryBuilder<Q>> for http::uri::PathAndQuery {
    type Error = http::uri::InvalidUri;

    #[inline(always)]
    fn try_from(value: UriPathQueryBuilder<Q>) -> Result<Self, Self::Error> {
        value.try_into_http()
    }
}
