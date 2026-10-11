mod particles;
mod static_site;
mod query;
mod webring;

use simple_logger::SimpleLogger;
use log::{error, info, warn};
use tiny_http::{Request, Response, Server, SslConfig};
use tokio::task::JoinHandle;
use tokio::time::error::Elapsed;
use tokio::time::timeout;
use std::fs;
use std::io::{Cursor, Error};
use std::time::Duration;
use crate::particles::particles_request;
use crate::query::Path;
use crate::static_site::site_request;
use crate::webring::webring_request;

#[tokio::main]
async fn main() {
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
                            let thread_count = 64;
                            let mut processes: Vec<JoinHandle<Result<(), Elapsed>>> = vec![];
                            let mut requests: Vec<Request> = vec![];
                            
                            let mut was_empty = false;
                            loop {
                                match server.recv() {
                                    Ok(request) => {
                                        info!("Free Threads: {} Received request: {request:?}", thread_count - processes.len());
                                        
                                        requests.push(request);

                                        if processes.len() == thread_count {
                                            warn!("Thread count exceeded by {} request(s)!", requests.len())
                                        }
                                    }
                                    Err(error) => {
                                        error!("Failed to receive a request: {error}");
                                    }
                                }

                                if processes.len() < thread_count {
                                    if let Some(request) = requests.pop() {
                                        processes.push(
                                            tokio::spawn(
                                        timeout(
                                                    Duration::from_secs(5), 
                                                    async move {
                                                        let response = handle_request(&request);
                                                        let _ = request.respond(response).map_err(
                                                            |e| {
                                                                error!("Couldn't send response: {e}");
                                                            }
                                                        );
                                                    }
                                                )
                                            )
                                        );
                                    } else {
                                        if was_empty == false {
                                            info!("All requests complete!");
                                            was_empty = true
                                        }
                                    }
                                }
                            
                                processes = processes.into_iter().filter_map(
                                    |process| {
                                        if process.is_finished() {None} else {Some(process)}
                                    }
                                ).collect();
                            }
                        }
                    );
                }
            );
        }
    );
}

fn handle_request(request: &Request) -> Response<Cursor<Vec<u8>>> {
    let url= request.url();
    let Some(path) = url.path() else {
        info!("Wasn't a valid path somehow?? {}", request.url());
        return Response::from_string("").with_status_code(400)
    };

    match path.get(1) {
        Some(&"particles") => particles_request(request),
        Some(&"webring") => webring_request(request),
        _ => site_request(request)
    }
}