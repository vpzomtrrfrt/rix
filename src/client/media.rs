use hyper;
use urlencoding;
use serde_json;

use futures::{Future, Stream};
use error::Error;
use ::HttpsClient;

pub fn upload<B: Into<hyper::Body>>(http: &HttpsClient, host: &str, token: &str, content_type: &str, filename: &str, body: B) -> Box<Future<Item=String,Error=Error> + Send> {
    let request = try_future_box!(hyper::Request::post(
        &format!("{}/_matrix/media/r0/upload?filename={}&access_token={}",
                         host,
                         urlencoding::encode(filename),
                         token))
        .header(hyper::header::CONTENT_TYPE, content_type)
        .body(body.into())
        .map_err(|e| Error::Other(format!("Failed to construct request: {:?}", e))));
    Box::new(http.request(request)
             .map_err(|e| e.into())
             .and_then(|response| {
                 let status = response.status();
                 response.into_body().concat2()
                     .map_err(|e| e.into())
                     .and_then(move |body| {
                         if status == hyper::StatusCode::OK {
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
