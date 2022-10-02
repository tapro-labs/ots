use std::collections::HashMap;

use envconfig::Envconfig;
use reqwest::Error as ReqwestError;
use rocket::serde::{json::serde_json, json::Value, Deserialize, Serialize};

use crate::integrations::slack::member::Member;

pub struct ClientError {
    pub message: String,
}

impl ClientError {
    pub fn new(message: String) -> Self {
        Self { message }
    }
}

impl From<ReqwestError> for ClientError {
    fn from(error: ReqwestError) -> Self {
        Self {
            message: error.to_string(),
        }
    }
}

#[derive(Debug, Deserialize, Serialize)]
pub struct SlackAccessTokenResponse {
    pub access_token: String,
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
}

impl Client {
    fn get_endpoint(&self, endpoint: &str) -> String {
        format!("{}/{}", self.options.url, endpoint)
    }

    pub async fn send_message(
        &self,
        auth_token: &str,
        channel_id: &str,
        message: &str,
    ) -> Result<(), ClientError> {
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
            if is_ok == false {
                return Err(ClientError::new(
                    response_json.get("error").unwrap().to_string(),
                ));
            }

            return Ok(());
        }

        Err(ClientError::new("Something went wrong!".to_owned()))
    }

    pub async fn fetch_users(
        &self,
        auth_token: &str,
    ) -> Result<SlackFetchUsersResponse, ClientError> {
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
            if is_ok == false {
                return Err(ClientError::new(
                    response_json.get("error").unwrap().to_string(),
                ));
            }

            return Ok(serde_json::from_str(&response_json.to_string()).unwrap());
        }

        Err(ClientError::new("Something went wrong!".to_owned()))
    }

    pub async fn fetch_auth_token(
        &self,
        code: &str,
    ) -> Result<SlackAccessTokenResponse, ClientError> {
        let mut params = HashMap::new();

        params.insert("code", code);
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
            if is_ok == false {
                return Err(ClientError::new(
                    response_json.get("error").unwrap().to_string(),
                ));
            }

            let access_token_value = response_json
                .get("authed_user")
                .unwrap()
                .get("access_token")
                .unwrap();

            if let Value::String(access_token) = access_token_value {
                return Ok(SlackAccessTokenResponse {
                    access_token: access_token.to_owned(),
                });
            }
        }

        Err(ClientError::new("Something went wrong!".to_owned()))
    }
}
