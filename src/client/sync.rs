use futures;
use hyper;
use serde_json;
use std;

use error::Error;
use futures::{Future, Stream};
use std::sync::Arc;
use HttpsClient;

#[derive(Deserialize, Debug, Clone)]
pub struct Event {
    pub content: serde_json::Value,
    #[serde(rename = "type")]
    pub event_type: String,
    pub sender: Option<String>,
    pub room: Option<String>,
}

#[derive(Deserialize, Debug)]
pub struct EventContainer {
    pub events: Vec<Event>,
}

#[derive(Deserialize, Debug)]
pub struct GroupSyncResult {
    pub timeline: Option<EventContainer>,
    pub ephemeral: Option<EventContainer>,
}

type GroupSyncContainer = std::collections::HashMap<String, GroupSyncResult>;

#[derive(Deserialize, Debug)]
pub struct GroupsSyncResult {
    pub leave: GroupSyncContainer,
    pub join: GroupSyncContainer,
    pub invite: GroupSyncContainer,
}

#[derive(Deserialize, Debug)]
pub struct SyncResult {
    pub next_batch: String,
    pub presence: EventContainer,
    pub rooms: GroupsSyncResult,
}

fn group_events<'a>(
    container: &'a GroupSyncContainer,
) -> Box<std::iter::Iterator<Item = &'a Event> + Send + 'a> {
    Box::new(container.iter().flat_map(|(_, group)| {
        vec![&group.timeline, &group.ephemeral]
            .into_iter()
            .filter_map(|x| x.as_ref())
            .flat_map(|x| x.events.iter())
    }))
}

impl SyncResult {
    pub fn events<'a>(&'a self) -> Box<std::iter::Iterator<Item = &'a Event> + Send + 'a> {
        Box::new(
            self.presence
                .events
                .iter()
                .chain(group_events(&self.rooms.leave))
                .chain(group_events(&self.rooms.join))
                .chain(group_events(&self.rooms.invite)),
        )
    }
}

fn fill_group_result(id: &str, container: &mut EventContainer) {
    for mut evt in &mut container.events {
        evt.room = Some(id.to_owned());
    }
}

fn fill_group_container(container: &mut GroupSyncContainer) {
    for (id, room) in container {
        if let Some(ref mut timeline) = room.timeline {
            fill_group_result(id, timeline);
        }
        if let Some(ref mut ephemeral) = room.ephemeral {
            fill_group_result(id, ephemeral);
        }
    }
}

fn fill_groups_result(result: &mut GroupsSyncResult) {
    fill_group_container(&mut result.leave);
    fill_group_container(&mut result.join);
    fill_group_container(&mut result.invite);
}

pub fn sync(
    http: &HttpsClient,
    host: &str,
    access_token: &str,
    since: Option<String>,
) -> Box<Future<Item = SyncResult, Error = Error> + Send> {
    let params = if let Some(since) = since {
        format!("&since={}&timeout=30000", since)
    } else {
        "".to_owned()
    };
    let request = try_future_box!(
        hyper::Request::get(&format!(
            "{}/_matrix/client/r0/sync?access_token={}{}",
            host, access_token, params
        )).body(hyper::Body::default())
            .map_err(|e| Error::Other(format!("Unable to create request: {:?}", e)))
    );
    Box::new(
        http.request(request)
            .and_then(|response| response.into_body().concat2())
            .map_err(|e| e.into())
            .and_then(|body| String::from_utf8(body.to_vec()).map_err(|e| e.into()))
            .and_then(|body| serde_json::from_str(&body).map_err(|e| e.into()))
            .and_then(|mut body: SyncResult| {
                fill_groups_result(&mut body.rooms);
                Ok(body)
            }),
    )
}

pub struct SyncStream {
    http_client: Arc<HttpsClient>,
    host: String,
    token: String,
    current_future: Box<Future<Item = SyncResult, Error = Error> + Send>,
}

impl Stream for SyncStream {
    type Item = SyncResult;
    type Error = Error;
    fn poll(&mut self) -> futures::Poll<Option<Self::Item>, Self::Error> {
        let poll_result = self.current_future.poll();
        match poll_result {
            Ok(futures::Async::Ready(item)) => {
                self.current_future = sync(
                    &self.http_client,
                    &self.host,
                    &self.token,
                    Some(item.next_batch.clone()),
                );
                Ok(futures::Async::Ready(Some(item)))
            }
            Ok(futures::Async::NotReady) => Ok(futures::Async::NotReady),
            Err(e) => Err(e),
        }
    }
}

pub fn sync_stream(http: Arc<HttpsClient>, host: &str, access_token: &str) -> SyncStream {
    SyncStream {
        host: host.to_owned(),
        token: access_token.to_owned(),
        current_future: sync(&http, host, access_token, None),
        http_client: http,
    }
}
