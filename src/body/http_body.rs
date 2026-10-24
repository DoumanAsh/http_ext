//! http_body module providing re-export of its content

use core::task::{self, Poll};
use core::pin::Pin;
use core::future::Future;

pub use http_body::{Body, Frame, SizeHint};

use super::{Accum, SizeLimit, Collect, CollectError};

impl<Data: AsRef<[u8]>, E, B: http_body::Body<Data = Data, Error = E>, Acc: Accum, LIMIT: SizeLimit> Future for Collect<B, Acc, LIMIT> {
    type Output = Result<Acc::Output, CollectError<E, Acc::Error>>;

    fn poll(self: Pin<&mut Self>, ctx: &mut task::Context<'_>) -> task::Poll<Self::Output> {

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
