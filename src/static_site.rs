use std::borrow::Cow;
use std::path::PathBuf;
use small_http::{Request, Response, Status};
use small_http::Status::{BadRequest, InternalServerError, NotFound};
use urlencoding::decode;
use std::fs;
use std::io::Error;
use std::string::FromUtf8Error;
use log::{error, info, warn};
use serde_json::{from_str, Value};

fn abs_path() -> PathBuf {PathBuf::from("./maddie_website")}
pub fn site_request(request: &Request) -> Response {
    decode(request.url.path()).map_or_else(
        |e: FromUtf8Error| {
            warn!("Failed to decode url... {e}");
            Response::with_status(BadRequest)
        },
        |request_decoded: Cow<str>| {
            fs::read_to_string(abs_path().join("manifest.json")).map_or_else(
                |e: Error| {
                    error!("Manifest couldn't be read!!!!!!!!!!!!!!! FUck. {e}");
                    Response::with_status(InternalServerError)
                },
                |manifest_raw: String| {
                    from_str(&manifest_raw).map_or_else(
                        |e: serde_json::Error| {
                            error!("Couldn't serialise manifest!!!! This is big fuckup not good. {e}");
                            Response::with_status(InternalServerError)
                        },
                        |manifest: Value| {
                            manifest.get(request_decoded.to_string()).map_or_else(
                                || {
                                    info!("Page {request_decoded} not found");
                                    Response::with_status(NotFound)
                                },
                                |page_entry: &Value| {
                                    page_entry.get("secondaryCondition").map_or_else(
                                        || {
                                            get_file(page_entry, "defaultPath")
                                        },
                                        |secondary_condition: &Value| {
                                            secondary_condition.as_str().map_or_else(
                                                || {
                                                    error!("The secondary condition isn't a string. what ");
                                                    Response::with_status(InternalServerError)
                                                },
                                                |secondary_condition: &str| {
                                                    request.headers.get("Cookie").map_or_else(
                                                        || {
                                                            get_file(page_entry, "defaultPath")
                                                        },
                                                        |cookie: &str| {
                                                            #[allow(clippy::match_bool)]
                                                            match cookie == secondary_condition {
                                                                true => {
                                                                    get_file(page_entry, "secondaryPath")
                                                                },
                                                                false => {
                                                                    get_file(page_entry, "defaultPath")
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
                    )
                }
            )

        }
    )
}

fn get_file(page_entry: &Value, page_option: &str) -> Response {
    page_entry.get("mimeType").map_or_else(
        || {
            error!("Page has no MIME type!!!! Fuck.");
            Response::with_status(InternalServerError)
        },
        |mime_type: &Value| {
            mime_type.as_str().map_or_else(
                || {
                    error!("MIME type wasn't a string!!!fuck");
                    Response::with_status(InternalServerError)
                },
                |mime_type: &str| {
                    page_entry.get(page_option).map_or_else(
                        || {
                            error!("Page has no {page_option}! shit!!!");
                            Response::with_status(InternalServerError)
                        },
                        |path: &Value| {
                            path.as_str().map_or_else(
                                || {
                                    error!("WHY is the {page_option} not none!!!! instead it's {path:?}");
                                    Response::with_status(InternalServerError)
                                },
                                |path: &str| {
                                    fs::read(abs_path().join(path)).map_or_else(
                                        |e: Error| {
                                            error!("Couldn't read file {path} because {e}");
                                            Response::with_status(InternalServerError)
                                        },
                                        |contents: Vec<u8>| {
                                            Response::with_status(Status::Ok)
                                                .body(contents)
                                                .header("Content-Type", mime_type)
                                        }
                                    )
                                }
                            )
                        }
                    )
                }
            )
        }
    )

}