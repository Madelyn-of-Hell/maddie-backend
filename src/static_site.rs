use std::borrow::Cow;
use std::path::PathBuf;
use tiny_http::{Header, Request, Response};
use urlencoding::decode;
use std::fs;
use std::io::{Error, Cursor};
use std::str::FromStr;
use std::string::FromUtf8Error;
use log::{error, info, warn};
use serde_json::{from_str, Value};

fn abs_path() -> PathBuf {PathBuf::from("./maddie_website")}
pub fn site_request(request: &Request) -> Response<Cursor<Vec<u8>>> {
    decode(request.url()).map_or_else(
        |e: FromUtf8Error| {
            warn!("Failed to decode url... {e}");
            Response::from_string("").with_status_code(400)
        },
        |request_decoded: Cow<str>| {
            fs::read_to_string(abs_path().join("manifest.json")).map_or_else(
                |e: Error| {
                    error!("Manifest couldn't be read!!!!!!!!!!!!!!! FUck. {e}");
                    Response::from_string("").with_status_code(500)
                },
                |manifest_raw: String| {
                    from_str(&manifest_raw).map_or_else(
                        |e: serde_json::Error| {
                            error!("Couldn't serialise manifest!!!! This is big fuckup not good. {e}");
                            Response::from_string("").with_status_code(500)
                        },
                        |manifest: Value| {
                            manifest.get(request_decoded.to_string()).map_or_else(
                                || {
                                    info!("Page {request_decoded} not found");
                                    Response::from_string("").with_status_code(404)
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
                                                    Response::from_string("").with_status_code(500)
                                                },
                                                |secondary_condition: &str| {
                                                    request.headers().iter().find_map(|header: &Header| { if header.field.to_string().as_str() == "Cookie" { Some(header.value.to_string()) } else { None } }).map_or_else(
                                                        || {
                                                            get_file(page_entry, "defaultPath")
                                                        },
                                                        |cookie: String| {
                                                            #[allow(clippy::match_bool)]
                                                            match cookie.contains(secondary_condition) {
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

fn get_file(page_entry: &Value, page_option: &str) -> Response<Cursor<Vec<u8>>> {
    page_entry.get("mimeType").map_or_else(
        || {
            error!("Page has no MIME type!!!! Fuck.");
            Response::from_string("").with_status_code(500)
        },
        |mime_type: &Value| {
            mime_type.as_str().map_or_else(
                || {
                    error!("MIME type wasn't a string!!!fuck");
                    Response::from_string("").with_status_code(500)
                },
                |mime_type: &str| {
                    page_entry.get(page_option).map_or_else(
                        || {
                            error!("Page has no {page_option}! shit!!!");
                            Response::from_string("").with_status_code(500)
                        },
                        |path: &Value| {
                            path.as_str().map_or_else(
                                || {
                                    error!("WHY is the {page_option} not none!!!! instead it's {path:?}");
                                    Response::from_string("").with_status_code(500)
                                },
                                |path: &str| {
                                    fs::read(abs_path().join(path)).map_or_else(
                                        |e: Error| {
                                            error!("Couldn't read file {path} because {e}");
                                            Response::from_string("").with_status_code(500)
                                        },
                                        |contents: Vec<u8>| {
                                            Header::from_str(format!("Content-Type: {mime_type}").as_str()).map_or_else(
                                                |()| {
                                                    error!("Failed to create a content type header ?????");
                                                    Response::from_string("").with_status_code(500)
                                                },
                                                |content_type: Header| {
                                                    Response::from_data(contents)
                                                        .with_header(content_type)
                                                        .with_status_code(200)
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