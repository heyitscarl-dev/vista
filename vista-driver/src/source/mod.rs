use bytes::Bytes;
use futures::Stream;

pub mod mjpeg;

pub trait Source {
    async fn into_stream(self) -> impl Stream<Item = Bytes>;
}
