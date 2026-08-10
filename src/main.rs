use std::collections::HashMap;
use std::net::TcpListener;
use small_http::{serve, Method, Request, Response, Status};
use small_http::Status::{BadRequest, InternalServerError, NotFound};
use std::fs;
use std::path::PathBuf;
use regex::{Captures, Regex};
use uuid::Uuid;
use urlencoding::decode;
use directories::ProjectDirs;
use serde_json::{to_string_pretty, Value, Error};
use simple_logger::SimpleLogger;
use log::{error, info, trace};

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
    match request.method {
        Method::Get => { // Create a room
            parse_query(request.url.query()).map_or_else(
                || {
                    info!("ignoring POST request with invalid query: {:?}", request.url.query());
                    Response::with_status(BadRequest)
                },
                |query: HashMap<String, String>| {
                    query.get("type").map_or_else(
                        || {
                            info!("ignoring POST request without a type specification");
                            Response::with_status(BadRequest)
                        },
                        |query_type: &String| {
                            match query_type.as_str() {
                                "view" => handle_view_request(&query),
                                "create" => handle_create_room_request(&query),
                                _ => {
                                    info!("ignoring POST request with invalid query type of {query_type}");
                                    Response::with_status(BadRequest)
                                }
                            }
                        }
                    )
                }
            )

        },
        Method::Connect => { // Join a room
            handle_join_request(request)
        },
        Method::Post => { // Make a move
            parse_query(request.url.query()).map_or_else(
                || {
                    info!("ignoring POST request with invalid query: {:?}", request.url.query());
                    Response::with_status(BadRequest)
                },
                |query: HashMap<String, String>| {
                    query.get("type").map_or_else(
                        || {
                            info!("ignoring POST request without a type specification");
                            Response::with_status(BadRequest)
                        },
                        |query_type: &String| {
                            match query_type.as_str() {
                                "move" => handle_move_request(&query),
                                "win" => handle_win_request(&query),
                                _ => {
                                    info!("ignoring POST request with invalid query type of {query_type}");
                                    Response::with_status(BadRequest)
                                }
                            }
                        }
                    )
                }
            )
        },
        _ => {
            Response::with_status(BadRequest)
        }
    }
}

fn handle_view_request(query: &HashMap<String, String>) -> Response {
    dir().map_or_else(
        || {
            error!("Couldn't get a project log...");
            Response::with_status(InternalServerError)
        },
        |data_dir: PathBuf| {
            query.get("room_code").map_or_else(
                || {
                    info!("ignoring POST request lacking a room code");
                    Response::with_status(BadRequest)
                },
                |room_code: &String| {
                    let room_file = data_dir.join(&room_code).with_extension("json");
                    match fs::exists(&room_file) {
                        Ok(true) => {
                            fs::read_to_string(&room_file).map_or_else(
                                |e: std::io::Error| {
                                    error!("Couldn't read valid room file {room_code} because {e}");
                                    Response::with_status(InternalServerError)
                                },
                                |raw_file: String| {
                                    trace!("Handed off the raw file for game {room_code}");
                                    Response::with_status(Status::Ok).body(raw_file.as_str())
                                }
                            )
                        }
                        _ => {
                            info!("ignoring POST request with invalid room code");
                            Response::with_status(BadRequest)
                        }
                    }
                }
            )
        }
    )
}

fn handle_move_request(query: &HashMap<String, String>) -> Response {
    dir().map_or_else(
        || {
            error!("Couldn't get a project dir for some reason");
            Response::with_status(InternalServerError)
        },
        |data_dir| {
            query.get("room_code").map_or_else(
                || {
                    info!("ignoring POST request with no room code");
                    Response::with_status(BadRequest)
                },
                |room_code: &String| {
                    query.get("player_code").map_or_else(
                        || {
                            info!("ignoring POST request with no player code");
                            Response::with_status(BadRequest)
                        },
                        |player_code: &String| {
                            query.get("move").map_or_else(
                                || {
                                    info!("ignoring POST request with no movement attached");
                                    Response::with_status(BadRequest)
                                },
                                |movement: &String| {
                                    let room_file = data_dir.join(room_code).with_extension("json");
                                    #[allow(clippy::single_match_else)]
                                    match fs::exists(&room_file) {
                                        Ok(true) => {
                                            fs::read_to_string(&room_file).map_or_else(
                                                |e: std::io::Error| {
                                                    error!("Couldn't read valid room file because {e}");
                                                    Response::with_status(InternalServerError)
                                                },
                                                |file_raw: String| {
                                                    serde_json::from_str(&file_raw).map_or_else(
                                                        |e: Error| {
                                                            error!("Couldn't deserialise valid room file. Killing it.\nError: {e}\n File contents:\n{file_raw}");
                                                            Response::with_status(InternalServerError)
                                                        },
                                                        |mut file_serialised: Value| {
                                                            let move_reg_op = file_serialised.get_mut("game_log").map(
                                                                |game_log: &mut Value| {
                                                                    game_log.as_array_mut().map_or_else(
                                                                        || {
                                                                            error!("game_log wasn't an array for some reason?? idk killing it lol\nFile Contents:\n{file_raw}");
                                                                            let _ = fs::remove_file(&room_file);
                                                                        },
                                                                        |game_log: &mut Vec<Value>| {
                                                                            game_log.push(Value::from(movement.as_str()));
                                                                        }
                                                                    );
                                                                }
                                                            );
                                                            if move_reg_op.is_none() {
                                                                error!("game_log doesn't exist in file for some reason.");
                                                                return Response::with_status(InternalServerError);
                                                            }

                                                            fs::write(&room_file, file_serialised.to_string()).map_or_else(
                                                                |e: std::io::Error| {
                                                                    error!("Couldn't write updated file for room {room_code} because {e}.\nFile Contents:\n{file_raw} ");
                                                                    Response::with_status(InternalServerError)
                                                                },
                                                                |()| {
                                                                    info!("Player {player_code} played a piece at {movement}!");
                                                                    Response::with_status(Status::Ok)
                                                                },
                                                            )
                                                        },
                                                    )
                                                },
                                            )
                                        }
                                        _ => {
                                            info!("Ignoring POST request with invalid room code");
                                            Response::with_status(BadRequest)
                                        }
                                    }
                                }
                            )
                        }
                    )
                }
            )
        }
    )
}
fn handle_win_request(query: &HashMap<String, String>) -> Response {
    dir().map_or_else(
        || {
            error!("Couldn't get a project dir for some reason");
            Response::with_status(InternalServerError)
        },
        |data_dir| {
            query.get("room_code").map_or_else(
                || {
                    info!("ignoring POST request with no room code");
                    Response::with_status(BadRequest)
                },
                |room_code: &String| {
                    query.get("player_code").map_or_else(
                        || {
                            info!("ignoring POST request with no player code");
                            Response::with_status(BadRequest)
                        },
                        |player_code: &String| {
                            let room_file = data_dir.join(room_code).with_extension("json");
                            #[allow(clippy::single_match_else)]
                            match fs::exists(&room_file) {
                                Ok(true) => {
                                    fs::read_to_string(&room_file).map_or_else(
                                        |e: std::io::Error| {
                                            error!("Couldn't read valid room file because {e}");
                                            Response::with_status(InternalServerError)
                                        },
                                        |file_raw: String| {
                                            serde_json::from_str(&file_raw).map_or_else(
                                                |e: Error| {
                                                    error!("Couldn't deserialise valid room file. Killing it.\nError: {e}\n File contents:\n{file_raw}");
                                                    Response::with_status(InternalServerError)
                                                },
                                                |mut file_serialised: Value| {
                                                    let win_reg_op = file_serialised.get_mut("winner").map(
                                                        |winner: &mut Value| {
                                                            *winner = Value::from(player_code.as_str());
                                                        }
                                                    );
                                                    if win_reg_op.is_none() {
                                                        error!("Winner doesn't exist in file for some reason.");
                                                        return Response::with_status(InternalServerError)
                                                    }

                                                    fs::write(room_file,file_serialised.to_string()).map_or_else(
                                                        |e: std::io::Error| {
                                                            error!("Couldn't write updated file for room {room_code} because {e}.\nFile Contents:\n{file_raw} ");
                                                            Response::with_status(InternalServerError)
                                                        },
                                                        |()| {
                                                            info!("Player {player_code} won game {room_code}!");
                                                            Response::with_status(Status::Ok)
                                                        }
                                                    )
                                                }
                                            )
                                        }
                                    )
                                },
                                _ => {
                                    info!("Ignoring POST request with invalid room code");
                                    Response::with_status(BadRequest)
                                }
                            }

                        }
                    )
                }
            )
        }
    )
}
fn handle_create_room_request(query: &HashMap<String, String>) -> Response {
    dir().map_or_else(
        || {
            error!("Couldn't get a project dir for some reason");
            Response::with_status(InternalServerError)
        },
        |data_dir: PathBuf| {
            let room_code = Uuid::new_v4().to_string();
            let user_code = Uuid::new_v4().to_string();
            to_string_pretty(&query).map_or_else(
                |e: Error| {
                    error!("failed at serialising config somehow... {e}");
                    Response::with_status(InternalServerError).body(e.to_string())
                },
                |room_config: String| {
                    to_string_pretty(&[&user_code]).map_or_else(
                        |e: Error| {
                            error!("failed at serialising user somehow... {e}");
                            Response::with_status(InternalServerError).body(e.to_string())
                        },
                        |users: String| {
                            let room_file = data_dir.join(&room_code).with_extension("json");
                            match fs::write(room_file, format!("{{\n\t\"config\": {room_config},\n\t\"players\": {users},\n\t\"game_log\": [],\n\t\"winner\": \"\"\n}}")) {
                                Ok(()) => {
                                    info!("Created room {room_code}");
                                    Response::with_status(Status::Ok).body(format!("room_code={room_code}&user_code={user_code}"))
                                }
                                Err(e) => {
                                    error!("couldn't create room file! cleaning up and admitting defeat. Error: {e}");
                                    let _ = fs::remove_file(&room_code);
                                    Response::with_status(InternalServerError)
                                }
                            }
                        }
                    )
                }
            )
        }
    )
}
fn handle_join_request(request: &Request) -> Response {
    dir().map_or_else(
        || {
            error!("Couldn't get a project dir for some reason");
            Response::with_status(InternalServerError)
        },
        |data_dir: PathBuf| {
            parse_query(request.url.query()).map_or_else(
                || {
                    info!("Ignoring bad query in join request");
                    Response::with_status(BadRequest)
                },
                |query: HashMap<String, String>| {
                    query.get("room_code").map_or_else(
                        || {
                            info!("Ignoring room-less join request");
                            Response::with_status(BadRequest)
                        },
                        |room_code: &String| {
                            let room_file = data_dir.join(room_code).with_extension("json");
                            #[allow(clippy::single_match_else)] // I want this one because then I get to match against the boolean in the same thing
                            match fs::exists(&room_file) {
                                Ok(true) => {
                                    match fs::read_to_string(&room_file) { // should be guaranteed now but yk
                                        Ok(file_raw) => {
                                            let user_code = Uuid::new_v4().to_string();
                                            let mut file_serialised = serde_json::from_str::<Value>(&file_raw);
                                            let add_user_op = file_serialised.as_mut().map(
                                                |room_config: &mut Value| {
                                                    room_config.get_mut("players").map_or_else(
                                                        || {
                                                            error!("Room didn't have users for some reason??? Killing it now. File contents:\n{file_raw}");
                                                            let _ = fs::remove_file(&room_file);
                                                        },
                                                        |users: &mut Value| {
                                                            users.as_array_mut().map_or_else(
                                                                || {
                                                                    error!("for some reason users wasn't an array?? Killing it now. file contents:\n{file_raw}");
                                                                    let _ = fs::remove_file(&room_file);
                                                                },
                                                                |users: &mut Vec<Value>| {
                                                                    users.push(Value::from(user_code.as_str()));
                                                                },
                                                            );
                                                        },
                                                    );
                                                },
                                            );
                                            if add_user_op.is_err() {
                                                return Response::with_status(InternalServerError)
                                            }

                                            file_serialised.map_or_else(
                                                |e: Error| {
                                                    error!("Couldn't parse room {room_code}. Deleting room. Error: {e}\n Room contents: {file_raw}");
                                                    let _ = fs::remove_file(&room_file);

                                                    Response::with_status(InternalServerError)
                                                },
                                                |file_serialised: Value| {
                                                    let new_file = file_serialised.to_string();
                                                    match fs::write(&room_file, &new_file) {
                                                        Ok(()) => {
                                                            Response::with_status(Status::Ok).body(user_code)
                                                        }
                                                        Err(e) => {
                                                            error!("Couldn't rewrite the room file: {e}");
                                                            Response::with_status(InternalServerError)
                                                        }
                                                    }
                                                },
                                            )
                                        }
                                        Err(e) => {
                                            info!("couldn't read file (for reasons incomprehensible to compiletime maddie): {e}");
                                            Response::with_status(InternalServerError)
                                        }
                                    }
                                }
                                _ => {
                                    info!("Rejecting invalid room code");
                                    Response::with_status(NotFound)
                                }
                            }
                        }
                    )
                }
            )
        }
    )

}
fn parse_query(query: Option<&str>) -> Option<HashMap<String, String>> {
    let re = Regex::new(r"([^&]+)=([^&]+)&?").ok()?;

    let mut map:HashMap<String, String> = HashMap::new();
    for (_, [key, value]) in re.captures_iter(query?).map(|c: Captures| c.extract()) {
        let _ = map.insert(key.to_string(), decode(value).ok()?.into_owned());
    }
    Some(map)
}
fn dir() -> Option<PathBuf> {
    let dirs = ProjectDirs::from("com", "madelyn_belmen", "particles_backend")?;
    let data_dir = dirs.data_dir();
    let _ = fs::create_dir_all(data_dir);
    Some(data_dir.to_owned())
}