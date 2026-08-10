mod particles;
mod static_site;

use std::ffi::OsStr;
use std::net::TcpListener;
use small_http::{serve, Request, Response};
use small_http::Status::{NotFound};
use std::path::PathBuf;
use simple_logger::SimpleLogger;
use log::{error};
use crate::particles::particles_request;
use crate::static_site::site_request;

fn main() {
    if SimpleLogger::new().env().init().is_err() {
        println!("Failed to setup logger.");
        std::process::exit(1)
    }

    match TcpListener::bind("0.0.0.0:80") {
        Ok(listener) => serve(listener, handle_request),
        Err(error) => {
            error!("{error}");
            std::process::exit(1);
        }
    }
}

fn handle_request(request: &Request) -> Response {
    PathBuf::from(request.url.path()).iter().next().map_or_else(
        || Response::with_status(NotFound),
        |path: &OsStr| {
            let path = path.to_string_lossy().into_owned();
            match path.as_str() {
                "particles" => particles_request(request),
                _ => site_request(request)
            }
        }
   )
}