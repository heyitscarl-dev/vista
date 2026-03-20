use futures::StreamExt;
use vista_driver::source::{Source, mjpeg::MjpegSource};

const URL: &'static str = "http://192.168.178.85:4747/video";

#[tokio::main]
async fn main() {
    env_logger::init();

    let mut frames = MjpegSource::new(URL).into_stream().await;

    while let Some(frame) = frames.next().await {
        log::debug!("got frame, {} bytes", frame.len());
    }
}
