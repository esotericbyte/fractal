use super::AuthenticationData;
use reqwest::Client;
use reqwest::Error;
use reqwest::Request;
use serde::Serialize;
use url::Url;

#[derive(Clone, Debug, Serialize)]
pub struct Parameters {
    #[serde(skip_serializing_if = "String::is_empty")]
    pub access_token: String,
}

#[derive(Clone, Debug, Serialize)]
pub struct Body {
    pub new_password: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub auth: Option<AuthenticationData>,
}

pub fn request(base: Url, params: &Parameters, body: &Body) -> Result<Request, Error> {
    let url = base
        .join("/_matrix/client/r0/account/password")
        .expect("Malformed URL in change_password");

    Client::new().post(url).query(params).json(body).build()
}
