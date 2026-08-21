use std::collections::HashMap;
use std::fs;
use std::io::Cursor;
use std::path::PathBuf;
use std::str::FromStr;
use log::{error, info};
use serde::{Deserialize, Serialize};
use serde_json::from_str;
use tiny_http::{Header, Request, Response};
use rand::random;
use crate::query::{Path, Query};

pub fn webring_request(request: &Request) -> Response<Cursor<Vec<u8>>> {
    let url = request.url();
    let Some(path) = url.path() else {
        info!("Wasn't a valid path somehow?? {}", request.url());
        return Response::from_string("").with_status_code(400)
    };
    let Some(direction) = path.get(2) else {
        error!("Failed to get the direction for {}", request.url());
        return Response::from_string("").with_status_code(500)
    };
    let Some(direction) = (match direction.to_owned() {
        "next" => Some(1),
        "previous" => Some(-1),
        "random" => Some(random::<i32>()),
        _ => None,
    }) else {
        info!("Ignoring webring request with an invalid direction parameter: {:?}", direction);
        return Response::from_string("").with_status_code(400)
    };

    let Some(query) = request.url().query() else {
        info!("Ignoring webring request with an invalid query: {:?}", request.url());
        return Response::from_string("").with_status_code(400)
    };
    let Some(source) = query.get("source") else {
        info!("Ignoring webring request without a source parameter: {:?}", query);
        return Response::from_string("").with_status_code(400)
    };
    let Ok(webring_file) = fs::read_to_string("webring.json") else {
        error!("Couldn't read the webring !!!!");
        return Response::from_string("").with_status_code(500)
    };
    let Ok(webring) = from_str::<Webring>(&webring_file) else {
        error!("Couldn't parse the webring!!!!");
        return Response::from_string("").with_status_code(500)
    };
    let Some(source_idx) = webring.0.iter().position(|entry| entry.name.to_owned() == source.to_owned()) else {
        info!("Ignoring webring request with an unkown user: {source}");
        return Response::from_string("").with_status_code(404)
    };

    let new_idx = (source_idx + direction as usize) % webring.0.len();

    let Some(new_page) = webring.0.get(new_idx) else {
        error!("Couldn't get a new page that SHOULD exist—index {} of {:?}", new_idx, webring);
        return Response::from_string("").with_status_code(500)
    };

    let Ok(redirect_header) = Header::from_str(format!("Location: {}", new_page.url).as_str()) else {
        error!("Couldn't make a redirect header 😭");
        return Response::from_string("").with_status_code(500)
    };

    Response::from_string("").with_status_code(302).with_header(redirect_header)
}

#[derive(Serialize, Deserialize, Debug)]
struct Webring(Vec<WebringEntry>);
#[derive(Serialize, Deserialize, Debug)]
struct WebringEntry {
    name: String,
    url: String,
}
