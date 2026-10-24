//! Body utilities based on [http_body](https://crates.io/crates/http-body) crate

#[cfg(feature = "std")]
extern crate std;

use core::{mem, fmt};
use alloc::vec::Vec;
use alloc::collections::{TryReserveError};

#[cfg(feature = "http-body")]
pub use http_body::Body;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
///Supported [Content-Encoding](https://developer.mozilla.org/en-US/docs/Web/HTTP/Reference/Headers/Content-Encoding)
pub enum ContentEncoding {
    ///No encoding
    ///
    ///Default value for [Collect], which will prompt it to automatically deduce compression from
    ///first N bytes of the body (only zstd and zlib/gzip can be identified this way)
    Identity,
    ///Zstd encoding identified by `zstd`
    Zstd,
    ///Gzip encoding identified by `gzip`
    Gzip,
    ///Zlib encoding identified by `deflate`
    Zlib,
}

impl ContentEncoding {
    #[inline(always)]
    ///Returns whether encoding is [ContentEncoding::Identity]
    pub const fn is_identity(&self) -> bool {
        matches!(self, Self::Identity)
    }

    #[inline]
    ///Parses content-encoding header attempting to determine encoding.
    ///
    ///Returns `identity` in case of unsupported encoding
    pub const fn from_header_value(value: &[u8]) -> Self {
        match value {
            b"zstd" => Self::Zstd,
            b"deflate" => Self::Zlib,
            b"gzip" => Self::Gzip,
            _ => Self::Identity,
        }
    }
}

///Size limit for [Collect]
pub trait SizeLimit: Unpin {
    ///Should return `true` if `current_size` is over the allowed limit
    fn check_overflow(&self, size: usize) -> bool;
}

impl SizeLimit for () {
    ///Always returns `false`
    fn check_overflow(&self, _: usize) -> bool {
        false
    }
}

///Simple [SizeLimit] implementation to limit [Collect] by specified bytes size
pub struct BytesLimit(pub usize);

impl SizeLimit for BytesLimit {
    #[inline(always)]
    ///Returns `true` if `size` is greater than limit
    fn check_overflow(&self, size: usize) -> bool {
        size > self.0
    }
}

///Possible errors from [Collect]
#[derive(Debug)]
pub enum CollectError<T, C> {
    ///Underlying error from [Body]
    Transport(T),
    ///Underlying error from [Accum]
    Accum(C),
    ///Body is over limit
    Overflow,
    ///Failed to create decoder
    ///
    ///This can only happen due to insufficient memory
    DecoderMemError,
    ///Error decoding content of the [Body]
    DecoderError(compu::decoder::DecodeError),
}

impl<T, C> CollectError<T, C> {
    #[cold]
    #[inline(never)]
    fn unlikely_overflow() -> Self {
        Self::Overflow
    }

    #[cold]
    #[inline(never)]
    fn unlikely_decoder_mem_error() -> Self {
        Self::DecoderMemError
    }

    #[cold]
    #[inline(never)]
    fn unlikely_collector(error: C) -> Self {
        Self::Accum(error)
    }
}

impl<T: fmt::Display, C: fmt::Display> fmt::Display for CollectError<T, C> {
    #[inline(always)]
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::Transport(error) => fmt::Display::fmt(error, fmt),
            Self::Accum(error) => fmt::Display::fmt(error, fmt),
            Self::Overflow => fmt.write_str("Overflow"),
            Self::DecoderMemError => fmt.write_str("Failed to use decoder due to OOM"),
            Self::DecoderError(error) => fmt.write_fmt(format_args!("Collect body decode error: {error}"))
        }
    }
}

impl<T: fmt::Display + fmt::Debug, E: fmt::Display + fmt::Debug> core::error::Error for CollectError<T, E> {}

///Interface to accumulate [Body]
pub trait Accum: Unpin {
    ///Final result type
    type Output: Unpin;
    ///Potential error.
    ///
    ///If no error possible, just use `core::convert::Infallible`.
    type Error;

    ///Attempts to write data to `self`
    fn write(&mut self, data: &[u8]) -> Result<(), Self::Error>;

    ///Returns size of collected so far.
    fn len(&self) -> usize;

    ///Callback to be called when header map is encountered.
    fn on_trailers(&mut self, headers: http::HeaderMap);

    ///Callback to finalize writing `self`
    ///
    ///Only called once underlying body indicates it is consumed.
    fn finish(&mut self) -> Result<Self::Output, Self::Error>;
}

impl Accum for Vec<u8> {
    ///Content accumulated in the buffer returned from [Accum::finish]
    type Output = Vec<u8>;
    type Error = TryReserveError;

    #[inline(always)]
    fn write(&mut self, data: &[u8]) -> Result<(), Self::Error> {
        self.try_reserve(data.len())?;
        self.extend_from_slice(data);
        Ok(())
    }

    #[inline(always)]
    fn len(&self) -> usize {
        self.len()
    }

    #[inline(always)]
    fn on_trailers(&mut self, _: http::HeaderMap) {
    }

    #[inline(always)]
    fn finish(&mut self) -> Result<Self::Output, Self::Error> {
        let mut result = Vec::new();
        mem::swap(&mut result, self);
        Ok(result)
    }
}

impl Accum for bytes::BytesMut {
    ///Split [Bytes](https://docs.rs/bytes/latest/bytes/struct.BytesMut.html#method.split) to be returned from [Accum::finish]
    type Output = bytes::Bytes;
    type Error = core::convert::Infallible;

    #[inline(always)]
    fn write(&mut self, data: &[u8]) -> Result<(), Self::Error> {
        self.extend_from_slice(data);
        Ok(())
    }

    #[inline(always)]
    fn len(&self) -> usize {
        self.len()
    }

    #[inline(always)]
    fn on_trailers(&mut self, _: http::HeaderMap) {
    }

    #[inline(always)]
    ///Splits `self` to extract accumulated `data`
    fn finish(&mut self) -> Result<Self::Output, Self::Error> {
        let result = self.split();
        Ok(result.freeze())
    }
}

impl<A: Accum + ?Sized> Accum for &mut A {
    type Output = A::Output;
    type Error = A::Error;

    #[inline(always)]
    fn write(&mut self, data: &[u8]) -> Result<(), Self::Error> {
        (**self).write(data)
    }

    #[inline(always)]
    fn len(&self) -> usize {
        (**self).len()
    }

    #[inline(always)]
    fn on_trailers(&mut self, trailers: http::HeaderMap) {
        (**self).on_trailers(trailers)
    }

    #[inline(always)]
    fn finish(&mut self) -> Result<Self::Output, Self::Error> {
        (**self).finish()
    }
}

#[allow(clippy::large_enum_variant)]
enum StateDecode {
    Zstd(compu::decoder::ZstdC),
    Zlib(compu::decoder::ZlibRust),
}

#[allow(clippy::large_enum_variant)]
enum State {
    Decode(Vec<u8>, StateDecode),
    Plain,
}

impl State {
    ///Processes `data` writing it using `acc`
    fn process_data<T, A: Accum, LIMIT: SizeLimit>(&mut self, acc: &mut A, limit: &LIMIT, data: &[u8]) -> Result<(), CollectError<T, A::Error>> {
        use compu::Decoder;

        let data = match self {
            State::Plain => {
                data
            },
            State::Decode(buffer, decoder) => {
                buffer.clear();
                let result = match decoder {
                    StateDecode::Zstd(decoder) => decoder.decode_vec_full(data, buffer),
                    StateDecode::Zlib(decoder) => decoder.decode_vec_full(data, buffer),
                };

                match result {
                    Ok(status) => if let Err(error) = status.status {
                        return Err(CollectError::DecoderError(error));
                    },
                    Err(_) => return Err(CollectError::unlikely_decoder_mem_error())
                }

                buffer.as_slice()
            }
        };

        if limit.check_overflow(acc.len().saturating_add(data.len())) {
            return Err(CollectError::unlikely_overflow())
        }

        match acc.write(data) {
            Ok(()) => Ok(()),
            Err(error) => Err(CollectError::unlikely_collector(error))
        }
    }
}

///Utility to accumulate body via [Accum]
pub struct Collect<B, Acc, LIMIT> {
    body: B,
    accum: Acc,
    limit: LIMIT,
    encoding: ContentEncoding,
    state: Option<State>,
}

impl<B, Acc: Accum> Collect<B, Acc, ()> {
    #[inline(always)]
    ///Creates new instance with provided `body` and `accum` with no limit on body size
    ///
    ///`body` must be either [Body] or [Read](https://doc.rust-lang.org/std/io/trait.Read.html)
    pub const fn new(body: B, accum: Acc) -> Self {
        Self {
            body,
            accum,
            limit: (),
            encoding: ContentEncoding::Identity,
            state: None,
        }
    }

    #[inline(always)]
    ///Adds [SizeLimit] to self.
    pub fn with_bytes_limit(self, bytes: usize) -> Collect<B, Acc, BytesLimit> {
        Self::with_size_limit(self, BytesLimit(bytes))
    }
}

impl<B, Acc: Accum, LIMIT: SizeLimit> Collect<B, Acc, LIMIT> {
    #[inline(always)]
    ///Specifies [ContentEncoding] to assume rather than trying to deduce it from the first bytes of the body.
    pub fn with_encoding(mut self, encoding: ContentEncoding) -> Self {
        self.encoding = encoding;
        self
    }

    #[inline(always)]
    ///Applies specified `limit`
    pub fn with_size_limit<NL: SizeLimit>(self, limit: NL) -> Collect<B, Acc, NL> {
        Collect { body: self.body, accum: self.accum, encoding: self.encoding, state: self.state, limit }
    }

    fn init_state<'a>(&'a mut self, first: &[u8]) -> (Option<&'a mut State>, &'a mut Acc, &'a LIMIT) {
        if self.state.is_none() {
            match self.encoding {
                ContentEncoding::Zstd => if let Some(zstd) = compu::decoder::ZstdC::new(Default::default()) {
                    self.state = Some(State::Decode(Vec::new(), StateDecode::Zstd(zstd)));
                },
                ContentEncoding::Zlib => if let Some(zlib) = compu::decoder::ZlibRust::new(compu::decoder::ZlibMode::Zlib) {
                    self.state = Some(State::Decode(Vec::new(), StateDecode::Zlib(zlib)));
                },
                ContentEncoding::Gzip => if let Some(zlib) = compu::decoder::ZlibRust::new(compu::decoder::ZlibMode::Gzip) {
                    self.state = Some(State::Decode(Vec::new(), StateDecode::Zlib(zlib)));
                },
                ContentEncoding::Identity => {
                    match compu::Detection::detect(first) {
                        Some(compu::Detection::Zstd) => if let Some(zstd) = compu::decoder::ZstdC::new(Default::default()) {
                            self.state = Some(State::Decode(Vec::new(), StateDecode::Zstd(zstd)));
                        },
                        Some(compu::Detection::Zlib) => if let Some(zlib) = compu::decoder::ZlibRust::new(compu::decoder::ZlibMode::Zlib) {
                            self.state = Some(State::Decode(Vec::new(), StateDecode::Zlib(zlib)));
                        },
                        Some(compu::Detection::Gzip) => if let Some(zlib) = compu::decoder::ZlibRust::new(compu::decoder::ZlibMode::Gzip) {
                            self.state = Some(State::Decode(Vec::new(), StateDecode::Zlib(zlib)));
                        },
                        //Either first chunk is just empty, or it is really just plain data
                        _ => {
                            self.state = Some(State::Plain);
                        }
                    }
                }
            }
        }

        (self.state.as_mut(), &mut self.accum, &self.limit)
    }
}

#[cfg(feature = "std")]
impl<B: std::io::Read, Acc: Accum, LIMIT: SizeLimit> Collect<B, Acc, LIMIT> {
    ///Reads body fully using [io::Read](https://doc.rust-lang.org/std/io/trait.Read.html)
    ///
    ///Requires `std` feature
    pub fn read_all(&mut self) -> Result<Acc::Output, CollectError<std::io::Error, Acc::Error>> {
        let mut buffer = [0u8; 4096];
        let mut size = 0;

        'main: loop {
            loop {
                match self.body.read(&mut buffer[size..]) {
                    Ok(0) => break,
                    Ok(read_size) => {
                        size += read_size;

                        //Try to fill as much as possible
                        if buffer.len().saturating_sub(size) > 256 {
                            continue;
                        }
                    },
                    Err(error) => match error.kind() {
                        std::io::ErrorKind::Interrupted => continue,
                        _ => break 'main Err(CollectError::Transport(error))
                    }
                }
            }

            if size == 0 {
                break match self.accum.finish() {
                    Ok(output) => Ok(output),
                    Err(error) => Err(CollectError::unlikely_collector(error))
                }
            }

            let data = &buffer[..size];
            let (state, acc, limit) = match self.init_state(data) {
                (Some(state), acc, limit) => (state, acc, limit),
                _ => break Err(CollectError::unlikely_decoder_mem_error())
            };

            state.process_data(acc, limit, data)?;
            size = 0;
        }
    }
}

#[cfg(feature = "http-body")]
impl<Data: AsRef<[u8]>, E, B: http_body::Body<Data = Data, Error = E>, Acc: Accum, LIMIT: SizeLimit> core::future::Future for Collect<B, Acc, LIMIT> {
    type Output = Result<Acc::Output, CollectError<E, Acc::Error>>;

    fn poll(self: core::pin::Pin<&mut Self>, ctx: &mut core::task::Context<'_>) -> core::task::Poll<Self::Output> {
        use core::task::Poll;

        //we do not move anything and there is no internal references
        let this = unsafe {
            self.get_unchecked_mut()
        };

        'main: loop {
            let body = 'body: loop {
                let body = unsafe {
                    core::pin::Pin::new_unchecked(&mut this.body)
                };
                match http_body::Body::poll_frame(body, ctx) {
                    Poll::Pending => break 'main Poll::Pending,
                    Poll::Ready(Some(Ok(frame))) => match frame.into_data() {
                        Ok(body) => {
                            if body.as_ref().is_empty() {
                                continue 'body
                            } else {
                                break 'body body;
                            }
                        }
                        Err(frame) => match frame.into_trailers() {
                            Ok(trailers) => this.accum.on_trailers(trailers),
                            Err(_) => unreachable!()
                        }
                    },
                    Poll::Ready(Some(Err(error))) => break 'main Poll::Ready(Err(CollectError::Transport(error))),
                    Poll::Ready(None) => break 'main match this.accum.finish() {
                        Ok(result) => Poll::Ready(Ok(result)),
                        Err(error) => Poll::Ready(Err(CollectError::unlikely_collector(error))),
                    },
                }
            };

            let data = body.as_ref();
            let (state, acc, limit) = match this.init_state(data) {
                (Some(state), acc, limit) => (state, acc, limit),
                _ => break 'main Poll::Ready(Err(CollectError::unlikely_decoder_mem_error()))
            };

            if let Err(error) = state.process_data(acc, limit, data) {
                break 'main Poll::Ready(Err(error));
            }
        }
    }
}
