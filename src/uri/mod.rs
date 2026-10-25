//!URI module

use core::{mem, ops};

pub use http::uri::{Uri, InvalidUri};

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
        http::uri::PathAndQuery::from_maybe_shared(self.output.freeze())
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
    query_set: bool,
    #[allow(unused)]
    encoder: Q,
}

impl<Q: QueryEncoding> UriPathQueryBuilder<Q> {
    const QUERY_START: &[u8] = &[b'?'];
    const QUERY_NEXT: &[u8] = &[b'&'];
    const QUERY_SEP: &[u8] = &[b'='];

    #[inline]
    ///Creates new URI's query builder
    pub fn new(path: UriPathBuilder<impl PathEncoding>, encoder: Q) -> Self {
        Self {
            output: path.output,
            query_set: false,
            encoder,
        }
    }

    fn prepare_next_query_value(&mut self) {
        if self.query_set {
            self.output.extend_from_slice(Self::QUERY_NEXT);
        } else {
            self.output.extend_from_slice(Self::QUERY_START);
        }
    }

    fn add_key(&mut self, key: &str) -> &mut Self {
        self.output.reserve(key.len().saturating_add(1));

        self.prepare_next_query_value();

        for component in Q::encode(key) {
            self.output.extend_from_slice(component.as_bytes());
        }

        self.query_set = true;
        self
    }

    fn add_value(&mut self, value: &str) -> &mut Self {
        self.output.reserve(value.len().saturating_add(1));

        self.output.extend_from_slice(Self::QUERY_SEP);

        for component in Q::encode(value) {
            self.output.extend_from_slice(component.as_bytes());
        }

        self
    }

    ///Adds `key` and `value` to the query parameters
    pub fn add_kv(&mut self, key: &str, value: &str) -> &mut Self {
        self.output.reserve(key.len().saturating_add(value.len()).saturating_add(2));

        self.prepare_next_query_value();

        for component in Q::encode(key) {
            self.output.extend_from_slice(component.as_bytes());
        }

        self.output.extend_from_slice(Self::QUERY_SEP);

        for component in Q::encode(value) {
            self.output.extend_from_slice(component.as_bytes());
        }

        self.query_set = true;
        self
    }

    #[inline(always)]
    ///Adds pairs of key and values from `object`
    ///
    ///`object` must map/struct or sequence/tuple of pairs of string and string convertable values.
    pub fn add_serde(&mut self, object: impl serde::Serialize) -> Result<&mut Self, serde::Error> {
        object.serialize(serde::QueryVisitor::new(self))?;
        Ok(self)
    }

    #[inline(always)]
    ///Finalizes builder
    pub fn try_into_http(self) -> Result<http::uri::PathAndQuery, http::uri::InvalidUri> {
        http::uri::PathAndQuery::from_maybe_shared(self.output.freeze())
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

///Ongoing url builder with following type parameters
///
///- `T` - destination where Uri will be stored
///- `B` - actual builder ongoing. Implements `DerefMut`
pub struct UriBuilderPathBuild<T, B> {
    inner: T,
    builder: B
}

impl<T, B> ops::Deref for UriBuilderPathBuild<T, B> {
    type Target = B;
    #[inline(always)]
    fn deref(&self) -> &Self::Target {
        &self.builder
    }
}

impl<T, B> ops::DerefMut for UriBuilderPathBuild<T, B> {
    #[inline(always)]
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.builder
    }
}

impl<T, P: PathEncoding> UriBuilderPathBuild<T, UriPathBuilder<P>> {
    #[inline(always)]
    ///Join `self` with specified `path`
    pub fn join_path(self, path: &str) -> Self {
        let Self { inner, builder } = self;
        Self {
            inner,
            builder: builder.join(path)
        }
    }

    #[inline(always)]
    ///Starts building query part of the Uri using provided [encoder](QueryEncoding)
    pub fn build_query<Q: QueryEncoding>(self, encoder: Q) -> UriBuilderPathBuild<T, UriPathQueryBuilder<Q>> {
        let Self { inner, builder } = self;
        UriBuilderPathBuild {
            inner,
            builder: builder.with_query(encoder),
        }
    }
}

impl<T, Q: QueryEncoding> UriBuilderPathBuild<T, UriPathQueryBuilder<Q>> {
    #[inline(always)]
    ///Adds `key` and `value` to the query parameters
    pub fn add_kv_query(self, key: &str, value: &str) -> Self {
        let Self { inner, mut builder } = self;
        builder.add_kv(key, value);
        UriBuilderPathBuild {
            inner,
            builder,
        }
    }

    #[inline(always)]
    ///Adds pairs of key and values from `object`
    ///
    ///`object` must map/struct or sequence/tuple of pairs of string and string convertable values.
    pub fn add_serde_query(self, object: impl serde::Serialize) -> Result<Self, serde::Error> {
        let Self { inner, mut builder } = self;
        builder.add_serde(object)?;
        Ok(UriBuilderPathBuild {
            inner,
            builder,
        })
    }
}

impl<B: TryInto<http::uri::PathAndQuery, Error = http::uri::InvalidUri>> UriBuilderPathBuild<http::uri::Builder, B> {
    #[inline(always)]
    ///Finish uri's path builder returning modified [http::uri::Builder]
    pub fn finish_path(self) -> http::uri::Builder {
        let Self { inner, builder } = self;
        inner.path_and_query(builder)
    }
}

impl<'a, B: TryInto<http::uri::PathAndQuery, Error = http::uri::InvalidUri>> UriBuilderPathBuild<&'a mut http::Uri, B> {
    #[inline(always)]
    ///Finish uri's path builder writing into reference of [http::Uri]
    pub fn finish_path(self) -> Result<&'a mut http::Uri, http::uri::InvalidUri> {
        let Self { inner, builder } = self;

        let temp = mem::take(inner);
        let mut parts = temp.into_parts();
        parts.path_and_query = Some(builder.try_into()?);
        match http::Uri::from_parts(parts) {
            Ok(uri) => {
                *inner = uri;
                Ok(inner)
            },
            Err(_) => unreachable!()
        }
    }
}

impl<B: TryInto<http::uri::PathAndQuery, Error = http::uri::InvalidUri>> UriBuilderPathBuild<http::Uri, B> {
    #[inline(always)]
    ///Finish uri's path builder returning modified [http::Uri]
    pub fn finish_path(self) -> Result<http::Uri, http::uri::InvalidUri> {
        let Self { inner, builder } = self;

        let mut parts = inner.into_parts();
        parts.path_and_query = Some(builder.try_into()?);
        match http::Uri::from_parts(parts) {
            Ok(uri) => Ok(uri),
            Err(_) => unreachable!()
        }
    }
}

impl<B: TryInto<http::uri::PathAndQuery, Error = http::uri::InvalidUri>> UriBuilderPathBuild<http::request::Parts, B> {
    #[inline(always)]
    ///Finish uri's path builder returning modified [http::request::Parts]
    pub fn finish_path(self) -> Result<http::request::Parts, http::uri::InvalidUri> {
        let Self { mut inner, builder } = self;

        inner.uri = UriBuilderPathBuild {
            inner: inner.uri,
            builder,
        }.finish_path()?;
        Ok(inner)
    }
}

impl<T, B: TryInto<http::uri::PathAndQuery, Error = http::uri::InvalidUri>> UriBuilderPathBuild<http::Request<T>, B> {
    #[inline(always)]
    ///Finish uri's path builder returning modified [http::Request]
    pub fn finish_path(self) -> Result<http::Request<T>, http::uri::InvalidUri> {
        let Self { inner, builder } = self;

        let (mut parts, body) = inner.into_parts();
        parts = UriBuilderPathBuild {
            inner: parts,
            builder,
        }.finish_path()?;
        Ok(http::Request::from_parts(parts, body))
    }
}

///Extension trait to build [Uri]
///
///```
///use http_ext::uri::{self, Uri, UriBuilderExt};
///
/////Build URI only
///let uri = Uri::builder().scheme("http")
///                        .authority("localhost")
///                        //Start building path using extension trait from `/`
///                        .build_path(uri::PathEscape)
///                        .join_path("api")
///                        .join_path("v1")
///                        .join_path("logs")
///                        //Start query components
///                        .build_query(uri::QueryEscape)
///                        .add_kv_query("test 1", "value 1")
///                        //You can finish path builder at any time
///                        .finish_path()
///                        .build()
///                        .expect("to build uri");
///
///assert_eq!(uri.path(), "/api/v1/logs");
///assert_eq!(uri.query(), Some("test%201=value%201"));
///
/////Build as part of request
///let request = http::Request::new(()).build_path(uri::PathEscape)
///                                    .join_path("api")
///                                    .join_path("v1")
///                                    .join_path("logs")
///                                    //Start query components
///                                    .build_query(uri::QueryEscape)
///                                    .add_kv_query("test 1", "value 1")
///                                    .finish_path()
///                                    .expect("to build uri");
///assert_eq!(request.uri().path(), "/api/v1/logs");
///assert_eq!(request.uri().query(), Some("test%201=value%201"));
///```
pub trait UriBuilderExt: Sized {
    ///Start building path using initial path and specified [PathEncoding]
    fn build_path<P: PathEncoding>(self, encoding: P) -> UriBuilderPathBuild<Self, UriPathBuilder<P>>;
}

impl UriBuilderExt for http::uri::Builder {
    ///Starts building from `/`
    fn build_path<P: PathEncoding>(self, encoding: P) -> UriBuilderPathBuild<Self, UriPathBuilder<P>> {
        UriBuilderPathBuild {
            inner: self,
            builder: UriPathBuilder::new("/", encoding),
        }
    }
}

impl UriBuilderExt for &mut http::Uri {
    ///Start building path using initial `path` from [http::Uri::path] and specified [PathEncoding]
    ///
    ///NOTE: [http::Uri::query] is always ignored
    fn build_path<P: PathEncoding>(self, encoding: P) -> UriBuilderPathBuild<Self, UriPathBuilder<P>> {
        let builder = UriPathBuilder::new(self.path(), encoding);
        UriBuilderPathBuild {
            inner: self,
            builder,
        }
    }
}

impl UriBuilderExt for http::Uri {
    ///Start building path using initial `path` from [http::Uri::path] and specified [PathEncoding]
    ///
    ///NOTE: [http::Uri::query] is always ignored
    fn build_path<P: PathEncoding>(self, encoding: P) -> UriBuilderPathBuild<Self, UriPathBuilder<P>> {
        let builder = UriPathBuilder::new(self.path(), encoding);
        UriBuilderPathBuild {
            inner: self,
            builder,
        }
    }
}

impl UriBuilderExt for http::request::Parts {
    ///Start building path using initial `path` from [http::request::Parts::uri] and specified [PathEncoding]
    ///
    ///NOTE: [http::Uri::query] is always ignored
    fn build_path<P: PathEncoding>(self, encoding: P) -> UriBuilderPathBuild<Self, UriPathBuilder<P>> {
        let builder = UriPathBuilder::new(self.uri.path(), encoding);
        UriBuilderPathBuild {
            inner: self,
            builder,
        }
    }
}

impl<B> UriBuilderExt for http::Request<B> {
    ///Start building path using initial `path` from [http::Request::uri] and specified [PathEncoding]
    ///
    ///NOTE: [http::Uri::query] is always ignored
    fn build_path<P: PathEncoding>(self, encoding: P) -> UriBuilderPathBuild<Self, UriPathBuilder<P>> {
        let builder = UriPathBuilder::new(self.uri().path(), encoding);
        UriBuilderPathBuild {
            inner: self,
            builder,
        }
    }
}
