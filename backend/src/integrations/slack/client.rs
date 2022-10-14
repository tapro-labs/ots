use std::collections::HashMap;

use envconfig::Envconfig;
use reqwest::Error as ReqwestError;
use rocket::http::Status;
use rocket::serde::{json::serde_json, json::Value, Deserialize, Serialize};

use crate::integrations::slack::member::Member;
use crate::utils::logger;
use crate::GlobalOptions;

pub enum SlackRequestError {
    BadRequest,
    Unauthorized,
}

#[allow(clippy::from_over_into)]
impl Into<Status> for SlackRequestError {
    fn into(self) -> Status {
        match self {
            Self::BadRequest => Status::BadRequest,
            Self::Unauthorized => Status::Unauthorized,
        }
    }
}

impl From<ReqwestError> for SlackRequestError {
    fn from(_: ReqwestError) -> Self {
        Self::BadRequest
    }
}

#[derive(Debug, Deserialize, Serialize)]
pub struct SlackAccessTokenResponse {
    pub access_token: String,
    pub refresh_token: String,
    pub expires_in: u64,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct SlackFetchUsersResponse {
    #[serde(rename(deserialize = "members"))]
    pub users: Vec<Member>,
}

#[derive(Envconfig)]
pub struct ClientOptions {
    #[envconfig(from = "SLACK_URL")]
    url: String,

    #[envconfig(from = "SLACK_CLIENT_ID")]
    client_id: String,

    #[envconfig(from = "SLACK_CLIENT_SECRET")]
    client_secret: String,
}

pub struct Client {
    client: reqwest::Client,
    options: ClientOptions,
}

impl Client {
    pub fn new(options: ClientOptions) -> Self {
        Self {
            client: reqwest::Client::new(),
            options,
        }
    }

    pub fn new_defaults() -> Self {
        Self::new(ClientOptions::init_from_env().unwrap())
    }
}

impl Client {
    fn get_endpoint(&self, endpoint: &str) -> String {
        format!("{}/{}", self.options.url, endpoint)
    }

    fn parse_slack_error(&self, error: &str) -> SlackRequestError {
        match error {
            "token_revoked" | "access_denied" | "not_authed" | "token_expired" => {
                SlackRequestError::Unauthorized
            }
            _ => SlackRequestError::BadRequest,
        }
    }

    pub async fn send_message(
        &self,
        auth_token: &str,
        channel_id: &str,
        message: &str,
    ) -> Result<(), SlackRequestError> {
        let mut params = HashMap::new();

        params.insert("channel", channel_id);
        params.insert("as_user", "true");
        params.insert("text", message);

        let response = self
            .client
            .post(self.get_endpoint("chat.postMessage"))
            .header("Authorization", format!("Bearer {}", auth_token))
            .json(&params)
            .send()
            .await?;

        let response_json: Value = response.json().await?;

        if let Some(is_ok) = response_json.get("ok") {
            logger::info("[SLACK] - Sending message");
            logger::debug(format!(
                "[SLACK] - Sending message with params {:?}",
                params
            ));

            if is_ok == false {
                logger::debug(format!("[SLACK] - Error occurred: {:?}", response_json));

                return Err(
                    self.parse_slack_error(response_json.get("error").unwrap().as_str().unwrap())
                );
            }

            return Ok(());
        }

        Err(SlackRequestError::BadRequest)
    }

    pub async fn fetch_users(
        &self,
        auth_token: &str,
    ) -> Result<SlackFetchUsersResponse, SlackRequestError> {
        logger::info("[SLACK] - Fetching users");
        logger::debug(format!("[SLACK] - Fetching users with {}", auth_token));

        let mut query_params = HashMap::new();
        const MAX_USERS_PER_REQUEST: u8 = 200;

        query_params.insert("limit", MAX_USERS_PER_REQUEST);

        let response = self
            .client
            .get(self.get_endpoint("users.list"))
            .header("Authorization", format!("Bearer {}", auth_token))
            .query(&query_params)
            .send()
            .await?;

        let response_json: Value = response.json().await?;

        if let Some(is_ok) = response_json.get("ok") {
            if !is_ok.as_bool().unwrap() {
                logger::debug(format!(
                    "[SLACK] - Fetching users failed: {:?}",
                    response_json
                ));
                let error = response_json.get("error").unwrap().as_str().unwrap();

                return Err(self.parse_slack_error(error));
            }

            logger::debug(format!("[SLACK] Users fetched {:?}", response_json));

            return Ok(serde_json::from_str(&response_json.to_string()).unwrap());
        }

        logger::debug(format!(
            "[SLACK] - Fetching users failed: {:?}",
            response_json
        ));

        Err(SlackRequestError::BadRequest)
    }

    pub async fn fetch_auth_token(
        &self,
        code: &str,
    ) -> Result<SlackAccessTokenResponse, SlackRequestError> {
        logger::debug(format!("[SLACK] - Fetching auth token with code {}", code));

        let mut params = HashMap::new();

        params.insert("code", code);

        self.fetch_slack_access_token(params).await
    }

    pub async fn refresh_token(
        &self,
        refresh_token: &str,
    ) -> Result<SlackAccessTokenResponse, SlackRequestError> {
        logger::debug(format!(
            "[SLACK] - Fetching refreshed token with {}",
            refresh_token
        ));

        let mut params = HashMap::new();

        params.insert("grant_type", "refresh_token");
        params.insert("refresh_token", refresh_token);

        self.fetch_slack_access_token(params).await
    }
}

impl Client {
    async fn fetch_slack_access_token(
        &self,
        mut params: HashMap<&str, &str>,
    ) -> Result<SlackAccessTokenResponse, SlackRequestError> {
        logger::info("[SLACK] - Fetching auth token");
        logger::debug(format!("[SLACK] - Fetching token with params {:?}", params));

        let backend_server_url = format!(
            "{}/integrations/slack/webhook",
            GlobalOptions::default().backend_server_url
        );

        params.insert("redirect_uri", &backend_server_url);
        params.insert("client_id", &self.options.client_id);
        params.insert("client_secret", &self.options.client_secret);

        let response = self
            .client
            .post(self.get_endpoint("oauth.v2.access"))
            .form(&params)
            .send()
            .await?;
        let response_json: Value = response.json().await?;

        if let Some(is_ok) = response_json.get("ok") {
            if !is_ok.as_bool().unwrap() {
                logger::debug(format!(
                    "[SLACK] - Fetching refresh auth token failed: {:?}",
                    response_json
                ));
                let error = response_json.get("error").unwrap().as_str().unwrap();

                return Err(self.parse_slack_error(error));
            }

            logger::debug(format!("[SLACK] Token fetched {:?}", response_json));

            // fallback to response json if authed_user does not exist
            let auth_response_result = serde_json::from_value::<SlackAccessTokenResponse>(
                response_json
                    .get("authed_user")
                    .unwrap_or(&response_json)
                    .clone(),
            );

            if let Ok(response) = auth_response_result {
                return Ok(response);
            };
        }

        logger::debug(format!("[SLACK] - No token found: {:?}", response_json));

        Err(SlackRequestError::BadRequest)
    }
}
