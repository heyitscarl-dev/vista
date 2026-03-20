use bytes::Bytes;
use futures::{Stream, StreamExt};
use tokio::sync::mpsc;
use tokio_stream::wrappers::ReceiverStream;

use crate::source::Source;

#[derive(Debug, Clone)]
pub struct MjpegSource {
    url: &'static str,
}

#[derive(Default, Debug, Clone)]
pub enum MjpegState {
    #[default]
    AwaitingHeaders,
    AwaitingBody { content_length: usize },
}

#[derive(Default, Debug, Clone)]
pub struct MjpegParser {
    buffer: Vec<u8>,
    state: MjpegState,
}

impl MjpegParser {
    pub(super) fn feed(&mut self, chunk: &[u8]) -> Vec<Bytes> {
        self.buffer.extend_from_slice(chunk);
        let mut frames = Vec::new();

        loop {
            match &self.state {
                MjpegState::AwaitingHeaders => {
                    let Some(pos) = self.buffer.windows(4).position(|b| b == b"\r\n\r\n") else {
                        break;
                    };
                    let headers = std::str::from_utf8(&self.buffer[..pos]).unwrap();
                    let content_length = parse_content_length(headers)
                        .expect("missing or invalid Content-Length.");

                    log::debug!("parsed headers, content_length={content_length}");
                    self.buffer.drain(..pos + 4);
                    self.state = MjpegState::AwaitingBody { content_length };
                },
                MjpegState::AwaitingBody { content_length } => {
                    let content_length = *content_length;
                    if self.buffer.len() < content_length {
                        break;
                    }
                    let frame = Bytes::copy_from_slice(&self.buffer[..content_length]);
                    log::debug!("decoded frame, {} bytes", frame.len());
                    self.buffer.drain(..content_length);
                    self.state = MjpegState::AwaitingHeaders;
                    frames.push(frame);
                }
            }
        }

        frames
    }
}

fn parse_content_length(headers: &str) -> Option<usize> {
    headers
        .lines()
        .find(|l| l.starts_with("Content-Length"))?
        .split(": ")
        .nth(1)?
        .trim()
        .parse()
        .ok()
}

const MJPEG_STREAM_BUFFER_SIZE: usize = 4;

fn mjpeg_stream(url: &'static str) -> impl Stream<Item = Bytes> {
    let (tx, rx) = mpsc::channel(MJPEG_STREAM_BUFFER_SIZE);

    tokio::spawn(async move {
        let mut byte_stream = reqwest::get(url)
            .await
            .expect("could not connect")
            .bytes_stream();

        let mut parser = MjpegParser::default();

        while let Some(Ok(chunk)) = byte_stream.next().await {
            for frame in parser.feed(&chunk) {
                if tx.send(frame).await.is_err() {
                    break;
                }
            }
        }
    });

    ReceiverStream::new(rx)
}

impl MjpegSource {
    pub fn new(url: &'static str) -> Self {
        Self { url }
    }
}

impl Source for MjpegSource {
    async fn into_stream(self) -> impl futures::Stream<Item = bytes::Bytes> {
        mjpeg_stream(self.url)
    }
}
