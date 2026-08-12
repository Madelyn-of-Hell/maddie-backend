mod particles;
mod static_site;
mod query;

use std::ffi::OsStr;
use std::path::PathBuf;
use simple_logger::SimpleLogger;
use log::{debug, error, info};
use tiny_http::{Request, Response, Server, SslConfig};
use std::{fs, thread};
use std::io::{Cursor, Error};
use crate::particles::particles_request;
use crate::static_site::site_request;

fn main() {
    if SimpleLogger::new().env().init().is_err() {
        println!("Failed to setup logger.");
        std::process::exit(1)
    }
    fs::read("ssl/ssl-cert.pem").map_or_else(
        |e: Error| {
            error!("Couldn't read ssl certificate! {e}");
        },
        |cert: Vec<u8>| {
            info!("Successfully read certificate!");
            fs::read("ssl/ssl-key.pem").map_or_else(
                |e: Error| {
                    error!("Couldn't read ssl key! {e}");
                },
                |key: Vec<u8>| {
                    info!("Successfully read key!");
                    Server::https(
                        "0.0.0.0:443",
                        SslConfig {
                            certificate: cert,
                            private_key: key
                        }
                    ).map_or_else(
                        |error| {
                            error!("failed to start server: {error}");
                        },
                        |server: Server| {
                            loop {
                                match server.recv() {
                                    Ok(request) => {
                                        thread::spawn(
                                            move || {
                                                let response = handle_request(&request);
                                                request.respond(response).map_err(
                                                    |e| {
                                                        error!("Couldn't send response: {e}");
                                                    }
                                                )
                                            }
                                        );
                                    }
                                    Err(error) => {
                                        error!("Failed to receive a request: {error}");
                                    }
                                }
                            }
                        }
                    );
                }
            );
        }
    );
}

fn handle_request(request: &Request) -> Response<Cursor<Vec<u8>>> {
    debug!("Handling request: {request:?}");
    PathBuf::from(request.url()).iter().next().map_or_else(
        || Response::from_string("").with_status_code(404),
        |path: &OsStr| {
            let path = path.to_string_lossy().into_owned();
            match path.as_str() {
                "particles" => particles_request(request),
                _ => site_request(request)
            }
        }
   )
}