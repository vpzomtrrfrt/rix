use hyper;
use serde_json;

use futures::{Future, Stream};
use {Error, HttpsClient};

pub fn whoami(
    http: &HttpsClient,
    host: &str,
    token: &str,
) -> Box<Future<Item = String, Error = Error> + Send> {
    let request = try_future_box!(
        hyper::Request::get(&format!(
            "{}/_matrix/client/r0/account/whoami?access_token={}",
            host, token
        )).body(hyper::Body::default())
            .map_err(|e| Error::Other(format!("Failed to construct whoami request: {:?}", e)))
    );

    Box::new(http.request(request).map_err(|e| e.into()).and_then(
        |response| -> Box<Future<Item = String, Error = Error> + Send> {
            if response.status() == hyper::StatusCode::OK {
                Box::new(
                    response
                        .into_body()
                        .concat2()
                        .map_err(|e| e.into())
                        .and_then(|body| {
                            #[derive(Deserialize)]
                            struct WhoamiResult {
                                user_id: String,
                            }
                            serde_json::from_slice::<WhoamiResult>(&body)
                                .map_err(|e| e.into())
                                .map(|res| res.user_id)
                        }),
                )
            } else {
                Box::new(
                    response
                        .into_body()
                        .concat2()
                        .map_err(|e| e.into())
                        .and_then(|body| {
                            Err(Error::Other(
                                String::from_utf8_lossy(&body.to_vec()).into_owned(),
                            ))
                        }),
                )
            }
        },
    ))
}
