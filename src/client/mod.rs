use hyper;
use hyper_tls;

mod sync;
mod message;
mod media;

use futures::Future;

use std::sync::Arc;
use error::Error;
use HttpsClient;
pub use self::sync::{SyncResult, SyncStream, Event};

pub struct Client {
    http_client: Arc<HttpsClient>,
    host: String,
    token: String
}

impl Client {
    pub fn new(host: String, token: String) -> Result<Self, Error> {
        let http_client = Arc::new(hyper::Client::builder()
            .build(hyper_tls::HttpsConnector::new(1)?));
        Ok(Self {
            http_client,
            host,
            token
        })
    }

    pub fn sync(&self, since: Option<String>) -> Box<Future<Item=SyncResult, Error=Error> + Send> {
        sync::sync(&self.http_client, &self.host, &self.token, since)
    }
    pub fn sync_stream(&self) -> SyncStream {
        sync::sync_stream(self.http_client.clone(), &self.host, &self.token)
    }
    pub fn upload_media<B: Into<hyper::Body>>(&self, content_type: &str, filename: &str, body: B) -> Box<Future<Item=String, Error=Error> + Send> {
        media::upload(&self.http_client, &self.host, &self.token, content_type, filename, body)
    }
    pub fn send_message(&self, room: &str, msg: &str) -> Box<Future<Item=(), Error=Error> + Send> {
        message::send_message(&self.http_client, &self.host, &self.token, room, msg)
    }
    pub fn send_image(&self, room: &str, url: &str, msg: &str) -> Box<Future<Item=(), Error=Error> + Send> {
        message::send_image(&self.http_client, &self.host, &self.token, room, url, msg)
    }
    pub fn send_file(&self, room: &str, url: &str, msg: &str) -> Box<Future<Item=(), Error=Error> + Send> {
        message::send_file(&self.http_client, &self.host, &self.token, room, url, msg)
    }
}
