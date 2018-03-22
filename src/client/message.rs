use hyper;
use hyper_tls;
use serde_json;
use rand;
use futures;
use tokio_core;
use std;
use error;

use rand::Rng;
use error::Error;
use std::str::FromStr;
use futures::{Future, Stream};

pub fn send_message(host: &str, token: &str, handle: &tokio_core::reactor::Handle, room: &str, msg: &str) -> Box<Future<Item=(),Error=Error>> {
    let http = hyper::Client::configure()
        .connector(box_fut_try!(hyper_tls::HttpsConnector::new(1, &handle)
                                                 .map_err(|e| e.into())))
        .build(&handle);
    let mut rng = rand::thread_rng();
    let id: String = rng.gen_ascii_chars().take(16).collect();
    let body = json!({
        "body": msg.to_owned(),
        "msgtype": "m.text"
    }).to_string();
    let mut request = hyper::Request::new(hyper::Method::Put,
                                      box_fut_try!(hyper::Uri::from_str(&format!("{}/_matrix/client/r0/rooms/{}/send/m.room.message/{}?access_token={}", host, room, id, token)).map_err(|e| Error::HTTP(e.into()))));
    request.headers_mut().set(hyper::header::ContentType::json());
    request.headers_mut().set(hyper::header::ContentLength(body.len() as u64));
    request.set_body(body);
    Box::new(http.request(request)
             .map_err(|e| e.into())
             .and_then(|response| -> Box<Future<Item=(),Error=Error>> {
                 if response.status() == hyper::StatusCode::Ok {
                     Box::new(futures::future::ok(()))
                 }
                 else {
                     Box::new(response.body().concat2()
                         .map_err(|e| e.into())
                         .and_then(|body| {
                             match String::from_utf8(body.to_vec()) {
                                 Ok(val) => Err(Error::Other(format!("Failed sending message: {}", val))),
                                 Err(e) => Err(e.into())
                             }
                         }))
                 }
             }))
}
