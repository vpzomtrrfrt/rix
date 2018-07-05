use futures;
use hyper;
use rand;

use error::Error;
use futures::{Future, Stream};
use rand::Rng;
use HttpsClient;

pub fn send_message(
    http: &HttpsClient,
    host: &str,
    token: &str,
    room: &str,
    msg: &str,
) -> Box<Future<Item = (), Error = Error> + Send> {
    let body = json!({
        "msgtype": "m.text",
        "body": msg
    }).to_string();
    send_message_internal(http, host, token, room, body)
}

pub fn send_image(
    http: &HttpsClient,
    host: &str,
    token: &str,
    room: &str,
    url: &str,
    msg: &str,
) -> Box<Future<Item = (), Error = Error> + Send> {
    let body = json!({
        "msgtype": "m.image",
        "url": url,
        "body": msg
    }).to_string();
    send_message_internal(http, host, token, room, body)
}

pub fn send_file(
    http: &HttpsClient,
    host: &str,
    token: &str,
    room: &str,
    url: &str,
    msg: &str,
) -> Box<Future<Item = (), Error = Error> + Send> {
    let body = json!({
        "msgtype": "m.file",
        "url": url,
        "body": msg
    }).to_string();
    send_message_internal(http, host, token, room, body)
}

fn send_message_internal(
    http: &HttpsClient,
    host: &str,
    token: &str,
    room: &str,
    body: String,
) -> Box<Future<Item = (), Error = Error> + Send> {
    let mut rng = rand::thread_rng();
    let id: String = rng.gen_ascii_chars().take(16).collect();
    let request = try_future_box!(
        hyper::Request::put(&format!(
            "{}/_matrix/client/r0/rooms/{}/send/m.room.message/{}?access_token={}",
            host, room, id, token
        )).header(hyper::header::CONTENT_TYPE, "application/json")
            .header(hyper::header::CONTENT_LENGTH, body.len())
            .body(hyper::Body::from(body))
            .map_err(|e| Error::Other(format!("Failed to construct request: {:?}", e)))
    );
    Box::new(http.request(request).map_err(|e| e.into()).and_then(
        |response| -> Box<Future<Item = (), Error = Error> + Send> {
            if response.status() == hyper::StatusCode::OK {
                Box::new(futures::future::ok(()))
            } else {
                Box::new(
                    response
                        .into_body()
                        .concat2()
                        .map_err(|e| e.into())
                        .and_then(|body| match String::from_utf8(body.to_vec()) {
                            Ok(val) => {
                                Err(Error::Other(format!("Failed sending message: {}", val)))
                            }
                            Err(e) => Err(e.into()),
                        }),
                )
            }
        },
    ))
}
