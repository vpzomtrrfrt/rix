use hyper;
use futures;
use hyper_tls;
use urlencoding;
use serde_json;
use tokio_core;

use futures::{Future, Stream};
use error::Error;
use std::str::FromStr;

pub fn upload<B: Into<hyper::Body>>(host: &str, token: &str, handle: &tokio_core::reactor::Handle, content_type: hyper::mime::Mime, filename: &str, body: B) -> Box<Future<Item=String,Error=Error>> {
    let http = hyper::Client::configure()
        .connector(box_fut_try!(hyper_tls::HttpsConnector::new(1, &handle)
                                                 .map_err(|e| e.into())))
        .build(&handle);
    let mut request = hyper::Request::new(
        hyper::Method::Post,
        box_fut_try!(hyper::Uri::from_str(
                &format!("{}/_matrix/media/r0/upload?filename={}&access_token={}",
                         host,
                         urlencoding::encode(filename),
                         token)).map_err(|e| Error::HTTP(e.into()))));
    request.headers_mut().set(hyper::header::ContentType(content_type));
    request.set_body(body);
    Box::new(http.request(request)
             .map_err(|e| e.into())
             .and_then(|response| {
                 let status = response.status();
                 response.body().concat2()
                     .map_err(|e| e.into())
                     .and_then(move |body| {
                         if status == hyper::StatusCode::Ok {
                             serde_json::from_slice::<serde_json::Value>(&body)
                                 .map_err(|e| e.into())
                         }
                         else {
                             match String::from_utf8(body.to_vec()) {
                                 Ok(val) => Err(Error::Other(format!("Failed uploading media: {}", val))),
                                 Err(e) => Err(e.into())
                             }
                         }
                     })
             })
             .and_then(|body| {
                 body["content_uri"].as_str().ok_or_else(|| Error::Other("Failed uploading media: content_uri missing in response".to_owned())).map(|x| x.to_owned())
             })
             )
}
