use std::io::{self, Write};

use actix_web::http::header::ContentDisposition;
use actix_web::web::Bytes;
use actix_web::HttpResponse;
use futures_util::{Stream, stream};
use tokio::sync::mpsc;

const QUEUE: usize = 16;

/// Blocking writer that hands each write to an HTTP body. Run it on a blocking thread, because
/// `blocking_send` waits until the client has room for the chunk.
pub struct ChannelWriter(mpsc::Sender<Bytes>);

impl Write for ChannelWriter {
	fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
		self.0
			.blocking_send(Bytes::copy_from_slice(buf))
			.map_err(|_| io::Error::new(io::ErrorKind::BrokenPipe, "download closed"))?;
		Ok(buf.len())
	}

	fn flush(&mut self) -> io::Result<()> {
		Ok(())
	}
}

pub fn channel() -> (ChannelWriter, impl Stream<Item = Result<Bytes, actix_web::Error>>) {
	let (sender, receiver) = mpsc::channel::<Bytes>(QUEUE);
	let body = stream::unfold(receiver, |mut receiver| async move {
		receiver.recv().await.map(|chunk| (Ok::<Bytes, actix_web::Error>(chunk), receiver))
	});
	(ChannelWriter(sender), body)
}

pub fn zip_response<S>(file_name: &str, body: S) -> HttpResponse
where
	S: Stream<Item = Result<Bytes, actix_web::Error>> + 'static,
{
	HttpResponse::Ok()
		.content_type("application/zip")
		.insert_header(ContentDisposition::attachment(file_name))
		.streaming(body)
}
