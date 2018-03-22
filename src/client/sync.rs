use hyper;
use tokio_core;
use futures;
use hyper_tls;
use std;
use error;
use serde_json;

use std::str::FromStr;
use futures::{Future, Stream};
use error::Error;

#[derive(Deserialize, Debug)]
pub struct Event {
    pub content: serde_json::Value,
    #[serde(rename = "type")]
    pub event_type: String,
    pub sender: Option<String>
}

#[derive(Deserialize, Debug)]
pub struct EventContainer {
    pub events: Vec<Event>
}

#[derive(Deserialize, Debug)]
pub struct GroupSyncResult {
    pub timeline: EventContainer,
    pub ephemeral: EventContainer
}

type GroupSyncContainer = std::collections::HashMap<String, GroupSyncResult>;

#[derive(Deserialize, Debug)]
pub struct GroupsSyncResult {
    pub leave: GroupSyncContainer,
    pub join: GroupSyncContainer,
    pub invite: GroupSyncContainer
}

#[derive(Deserialize, Debug)]
pub struct SyncResult {
    pub next_batch: String,
    pub presence: EventContainer,
    pub rooms: GroupsSyncResult
}

pub fn sync(host: &str, access_token: &str, handle: &tokio_core::reactor::Handle, since: Option<String>) -> Box<Future<Item=SyncResult,Error=Error>> {
    let http = hyper::Client::configure()
        .connector(box_fut_try!(hyper_tls::HttpsConnector::new(1, &handle)
                                                 .map_err(|e| e.into())))
        .build(&handle);
    let params = if let Some(since) = since {
        format!("&since={}&timeout=30000", since)
    } else {
        "".to_owned()
    };
    let mut request = hyper::Request::new(hyper::Method::Get,
                                      box_fut_try!(hyper::Uri::from_str(&format!("{}/_matrix/client/r0/sync?access_token={}{}", host, access_token, params)).map_err(|e| Error::HTTP(e.into()))));
    Box::new(http.request(request)
        .and_then(|response| {
            response.body().concat2()
        })
        .map_err(|e| e.into())
        .and_then(|body| {
            String::from_utf8(body.to_vec()).map_err(|e| e.into())
        })
        .and_then(|body| {
            println!("{}", body);
            serde_json::from_str(&body).map_err(|e| e.into())
        })
        )
}
