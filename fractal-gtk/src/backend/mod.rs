use lazy_static::lazy_static;
use log::error;
use matrix_sdk::identifiers::{Error as IdentifierError, EventId, MxcUri, RoomId};
use matrix_sdk::{
    api::{error::ErrorKind as RumaErrorKind, Error as RumaClientError},
    Client as MatrixClient, Error as MatrixError, FromHttpResponseError, HttpError, ServerError,
};
use regex::Regex;
use std::fmt::Debug;
use std::io::Error as IoError;
use std::path::PathBuf;

use crate::client::Client;
use crate::util::cache_dir_path;
use matrix_sdk::api::r0::context::get_context::Request as GetContextRequest;
use matrix_sdk::api::r0::media::get_content::Request as GetContentRequest;
use matrix_sdk::api::r0::media::get_content_thumbnail::Method;
use matrix_sdk::api::r0::media::get_content_thumbnail::Request as GetContentThumbnailRequest;
use matrix_sdk::assign;

pub mod directory;
pub mod media;
pub mod register;
pub mod room;
pub mod sync;
pub mod user;

lazy_static! {
    pub static ref HTTP_CLIENT: Client = Client::new();
}

pub enum ContentType {
    Download,
    Thumbnail(u32, u32),
}

impl ContentType {
    pub fn default_thumbnail() -> Self {
        ContentType::Thumbnail(128, 128)
    }

    pub fn is_thumbnail(&self) -> bool {
        match self {
            ContentType::Download => false,
            ContentType::Thumbnail(_, _) => true,
        }
    }
}

pub async fn get_prev_batch_from(
    session_client: MatrixClient,
    room_id: &RoomId,
    event_id: &EventId,
) -> Result<String, MatrixError> {
    let request = assign!(GetContextRequest::new(room_id, event_id), {
        limit: 0_u32.into(),
    });

    let prev_batch = session_client
        .send(request, None)
        .await?
        .start
        .unwrap_or_default();
    Ok(prev_batch)
}

#[derive(Debug)]
pub enum MediaError {
    Io(IoError),
    Matrix(MatrixError),
}

impl From<MatrixError> for MediaError {
    fn from(err: MatrixError) -> Self {
        Self::Matrix(err)
    }
}

impl From<IoError> for MediaError {
    fn from(err: IoError) -> Self {
        Self::Io(err)
    }
}

impl HandleError for MediaError {}

pub async fn dw_media(
    session_client: MatrixClient,
    mxc: &MxcUri,
    media_type: ContentType,
    dest: Option<PathBuf>,
) -> Result<PathBuf, MediaError> {
    if !mxc.is_valid() {
        return Err(MatrixError::from(IdentifierError::InvalidMxcUri).into());
    }

    let default_fname = || {
        let dir = if media_type.is_thumbnail() {
            "thumbs"
        } else {
            "medias"
        };
        cache_dir_path(Some(dir), mxc.media_id().unwrap())
    };
    let fname = dest.clone().map_or_else(default_fname, Ok)?;

    // If the file is already cached and recent enough, don't download it
    let is_fname_recent = fname
        .metadata()
        .ok()
        .and_then(|md| md.modified().ok())
        .and_then(|modf| modf.elapsed().ok())
        .map_or(false, |dur| dur.as_secs() < 60);

    if fname.is_file() && (dest.is_none() || is_fname_recent) {
        return Ok(fname);
    }

    let media = if let ContentType::Thumbnail(width, height) = media_type {
        let request = assign!(GetContentThumbnailRequest::from_url(
                mxc,
                width.into(),
                height.into(),
            ).unwrap(), {
            method: Some(Method::Crop),
        });

        session_client.send(request, None).await?.file
    } else {
        let request = GetContentRequest::from_url(mxc).unwrap();
        session_client.send(request, None).await?.file
    };

    tokio::fs::write(&fname, media).await?;

    Ok(fname)
}

pub trait HandleError: Debug {
    #[track_caller]
    fn handle_error(&self) {
        let err_str = format!("{:?}", self);
        error!(
            "Query error: {}",
            remove_matrix_access_token_if_present(&err_str).unwrap_or(err_str)
        );
    }
}

// Returns the encapsulated error in case it originated at the Matrix API level
pub(self) fn get_ruma_client_error(matrix_error: &MatrixError) -> Option<&RumaClientError> {
    match matrix_error {
        MatrixError::Http(HttpError::FromHttpResponse(FromHttpResponseError::Http(
            ServerError::Known(error),
        ))) => Some(error),
        _ => None,
    }
}

// Returns the kind of error in case it originated at the Matrix API level
pub(self) fn get_ruma_error_kind(matrix_error: &MatrixError) -> Option<&RumaErrorKind> {
    get_ruma_client_error(matrix_error).map(|ruma_err| &ruma_err.kind)
}

/// This function removes the value of the `access_token` query from a URL used for accessing the Matrix API.
/// The primary use case is the removing of sensitive information for logging.
/// Specifically, the URL is expected to be contained within quotes and the token is replaced with `<redacted>`.
/// Returns `Some` on removal, otherwise `None`.
pub fn remove_matrix_access_token_if_present(message: &str) -> Option<String> {
    lazy_static! {
    static ref RE: Regex =
        Regex::new(r#""((http)|(https))://([^"]+)/_matrix/([^"]+)\?access_token=(?P<token>[^&"]+)([^"]*)""#,)
        .expect("Malformed regular expression.");
    }
    // If the supplied string doesn't contain a match for the regex, we return `None`.
    let cap = RE.captures(message)?;
    let captured_token = cap
        .name("token")
        .expect("'token' capture group not present.")
        .as_str();
    Some(message.replace(captured_token, "<redacted>"))
}
