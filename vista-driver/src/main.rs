use std::{time::Instant};

use futures::StreamExt;

const URL: &'static str = "http://192.168.178.85:4747/video";

fn handle_chunk(chunk: &[u8], buffer: &mut Vec<u8>, length: &mut Option<usize>) {
    buffer.extend_from_slice(chunk);

    // drain frame if enough data buffered.
    if length.is_some_and(|length| buffer.len() >= length) {
        // frame found.
        buffer.drain(0 .. length.unwrap());
    }

    // skip header parsing if headers already parsed.
    if length.is_some() {
        return;
    }

    // parse headers if marker present.
    if let Some(position) = buffer.windows(4).position(|b| b == b"\r\n\r\n") {
        log::info!("begin mjpeg headers.");

        let headers = std::str::from_utf8(&buffer[..position]).unwrap();
        *length = Some(headers
            .lines()
            .find(|l| l.starts_with("Content-Length"))
            .and_then(|l| l.split(": ").nth(1))
            .unwrap()
            .parse()
            .unwrap());
        buffer.drain(0 .. (position + 4));

        log::info!("begin mjpeg body.");
    }
}

#[tokio::main]
async fn main() {
    env_logger::init();

    log::info!("begin mjpeg request.");

    let mut stream = reqwest::get(URL)
        .await
        .expect("could not connect to url.")
        .bytes_stream();

    log::info!("begin mjpeg stream read.");

    let mut buffer: Vec<u8> = Vec::new();
    let mut content_length: Option<usize> = None;
    
    while let Some(Ok(chunk)) = stream.next().await {
        handle_chunk(&chunk, &mut buffer, &mut content_length);
    }

    log::info!("end mjpeg stream read.");
}
