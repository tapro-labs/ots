use std::collections::HashMap;

use envconfig::Envconfig;
use reqwest::Error as ReqwestError;
use rocket::http::Status;
use rocket::serde::{json::serde_json, json::Value, Deserialize, Serialize};
use utils::global_options::GlobalOptions;
use utils::logger;

use crate::member::Member;

#[derive(Debug)]
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

/// The `error` code of a failed Slack response, or "" if Slack sent none.
fn error_code(response_json: &Value) -> &str {
    response_json
        .get("error")
        .and_then(Value::as_str)
        .unwrap_or_default()
}

#[derive(Envconfig)]
pub struct ClientOptions {
    #[envconfig(from = "SLACK_URL")]
    pub(crate) url: String,

    #[envconfig(from = "SLACK_CLIENT_ID")]
    pub(crate) client_id: String,

    #[envconfig(from = "SLACK_CLIENT_SECRET")]
    pub(crate) client_secret: String,
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

                return Err(self.parse_slack_error(error_code(&response_json)));
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
            if !is_ok.as_bool().unwrap_or(false) {
                logger::debug(format!(
                    "[SLACK] - Fetching users failed: {:?}",
                    response_json
                ));

                return Err(self.parse_slack_error(error_code(&response_json)));
            }

            logger::debug(format!("[SLACK] Users fetched {:?}", response_json));

            return serde_json::from_value(response_json).map_err(|error| {
                logger::debug(format!("[SLACK] - Unexpected users response: {}", error));

                SlackRequestError::BadRequest
            });
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
            if !is_ok.as_bool().unwrap_or(false) {
                logger::debug(format!(
                    "[SLACK] - Fetching refresh auth token failed: {:?}",
                    response_json
                ));

                return Err(self.parse_slack_error(error_code(&response_json)));
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_support::{
        client_for, unreachable_url, StubServer, TEST_CLIENT_ID, TEST_CLIENT_SECRET,
    };
    use rocket::serde::json::json;

    fn member_json(id: &str, real_name: &str) -> Value {
        json!({
            "id": id,
            "name": id.to_lowercase(),
            "is_bot": false,
            "deleted": false,
            "profile": {
                "real_name": real_name,
                "image_512": format!("https://example.com/{}.png", id)
            }
        })
    }

    fn token_json() -> Value {
        json!({
            "access_token": "xoxe.xoxp-user-token",
            "refresh_token": "xoxe-1-refresh",
            "expires_in": 43200
        })
    }

    // --- error mapping ---

    #[test]
    fn test_slack_request_error_into_status() {
        let bad_request: Status = SlackRequestError::BadRequest.into();
        let unauthorized: Status = SlackRequestError::Unauthorized.into();

        assert_eq!(bad_request, Status::BadRequest);
        assert_eq!(unauthorized, Status::Unauthorized);
    }

    #[test]
    fn test_parse_slack_error_maps_auth_errors_to_unauthorized() {
        let client = client_for("http://slack.test");

        for error in [
            "token_revoked",
            "access_denied",
            "not_authed",
            "token_expired",
        ] {
            assert!(
                matches!(
                    client.parse_slack_error(error),
                    SlackRequestError::Unauthorized
                ),
                "{error} should map to Unauthorized"
            );
        }
    }

    #[test]
    fn test_parse_slack_error_maps_other_errors_to_bad_request() {
        let client = client_for("http://slack.test");

        for error in ["channel_not_found", "invalid_code", "ratelimited", ""] {
            assert!(
                matches!(
                    client.parse_slack_error(error),
                    SlackRequestError::BadRequest
                ),
                "{error} should map to BadRequest"
            );
        }
    }

    #[test]
    fn test_get_endpoint_joins_base_url_and_method() {
        let client = client_for("https://slack.com/api");

        assert_eq!(
            client.get_endpoint("users.list"),
            "https://slack.com/api/users.list"
        );
    }

    // --- response (de)serialization ---

    #[test]
    fn test_fetch_users_response_reads_members_and_writes_users() {
        let response: SlackFetchUsersResponse = serde_json::from_value(json!({
            "ok": true,
            "members": [member_json("U1", "Ann"), member_json("U2", "Bob")],
            "response_metadata": { "next_cursor": "" }
        }))
        .unwrap();

        assert_eq!(response.users.len(), 2);

        let serialized = serde_json::to_value(&response).unwrap();
        assert_eq!(serialized["users"][0]["id"], "U1");
        assert_eq!(serialized["users"][1]["name"], "Bob");
        assert!(serialized.get("members").is_none());
    }

    // --- send_message ---

    #[rocket::async_test]
    async fn test_send_message_posts_json_with_bearer_token() {
        let server = StubServer::json(r#"{"ok":true,"channel":"C1","ts":"1.2"}"#);

        server
            .client()
            .send_message("user-token", "C1", "hello there")
            .await
            .unwrap();

        let request = server.single_request();
        assert_eq!(request.method, "POST");
        assert_eq!(request.target, "/chat.postMessage");
        assert_eq!(request.header("Authorization"), Some("Bearer user-token"));
        assert_eq!(
            serde_json::from_str::<Value>(&request.body).unwrap(),
            json!({ "channel": "C1", "as_user": "true", "text": "hello there" })
        );
    }

    #[rocket::async_test]
    async fn test_send_message_maps_auth_error_to_unauthorized() {
        let server = StubServer::json(r#"{"ok":false,"error":"token_revoked"}"#);

        let result = server.client().send_message("t", "C1", "hi").await;

        assert!(matches!(result, Err(SlackRequestError::Unauthorized)));
    }

    #[rocket::async_test]
    async fn test_send_message_maps_other_error_to_bad_request() {
        let server = StubServer::json(r#"{"ok":false,"error":"channel_not_found"}"#);

        let result = server.client().send_message("t", "C1", "hi").await;

        assert!(matches!(result, Err(SlackRequestError::BadRequest)));
    }

    #[rocket::async_test]
    async fn test_send_message_without_error_code_is_bad_request() {
        let server = StubServer::json(r#"{"ok":false}"#);

        let result = server.client().send_message("t", "C1", "hi").await;

        assert!(matches!(result, Err(SlackRequestError::BadRequest)));
    }

    #[rocket::async_test]
    async fn test_send_message_without_ok_field_is_bad_request() {
        let server = StubServer::json(r#"{"channel":"C1"}"#);

        let result = server.client().send_message("t", "C1", "hi").await;

        assert!(matches!(result, Err(SlackRequestError::BadRequest)));
    }

    #[rocket::async_test]
    async fn test_send_message_with_non_json_response_is_bad_request() {
        let server = StubServer::start(502, "text/html", "<h1>Bad gateway</h1>");

        let result = server.client().send_message("t", "C1", "hi").await;

        assert!(matches!(result, Err(SlackRequestError::BadRequest)));
    }

    #[rocket::async_test]
    async fn test_send_message_when_slack_is_unreachable_is_bad_request() {
        let result = client_for(&unreachable_url())
            .send_message("t", "C1", "hi")
            .await;

        assert!(matches!(result, Err(SlackRequestError::BadRequest)));
    }

    // --- fetch_users ---

    #[rocket::async_test]
    async fn test_fetch_users_requests_list_with_limit_and_parses_members() {
        let body = json!({
            "ok": true,
            "members": [member_json("U1", "Ann"), member_json("U2", "Bob")]
        });
        let server = StubServer::json(&body.to_string());

        let response = server.client().fetch_users("user-token").await.unwrap();

        assert_eq!(response.users.len(), 2);
        assert_eq!(
            serde_json::to_value(&response.users[0]).unwrap()["name"],
            "Ann"
        );

        let request = server.single_request();
        assert_eq!(request.method, "GET");
        assert_eq!(request.target, "/users.list?limit=200");
        assert_eq!(request.header("Authorization"), Some("Bearer user-token"));
    }

    #[rocket::async_test]
    async fn test_fetch_users_maps_auth_error_to_unauthorized() {
        let server = StubServer::json(r#"{"ok":false,"error":"not_authed"}"#);

        let result = server.client().fetch_users("t").await;

        assert!(matches!(result, Err(SlackRequestError::Unauthorized)));
    }

    #[rocket::async_test]
    async fn test_fetch_users_maps_other_error_to_bad_request() {
        let server = StubServer::json(r#"{"ok":false,"error":"missing_scope"}"#);

        let result = server.client().fetch_users("t").await;

        assert!(matches!(result, Err(SlackRequestError::BadRequest)));
    }

    #[rocket::async_test]
    async fn test_fetch_users_without_error_code_is_bad_request() {
        let server = StubServer::json(r#"{"ok":false}"#);

        let result = server.client().fetch_users("t").await;

        assert!(matches!(result, Err(SlackRequestError::BadRequest)));
    }

    #[rocket::async_test]
    async fn test_fetch_users_with_unexpected_member_shape_is_bad_request() {
        let server = StubServer::json(r#"{"ok":true,"members":[{"id":"U1"}]}"#);

        let result = server.client().fetch_users("t").await;

        assert!(matches!(result, Err(SlackRequestError::BadRequest)));
    }

    #[rocket::async_test]
    async fn test_fetch_users_without_ok_field_is_bad_request() {
        let server = StubServer::json(r#"{"members":[]}"#);

        let result = server.client().fetch_users("t").await;

        assert!(matches!(result, Err(SlackRequestError::BadRequest)));
    }

    // --- fetch_auth_token / refresh_token ---

    #[rocket::async_test]
    async fn test_fetch_auth_token_posts_code_and_credentials() {
        let body = json!({ "ok": true, "app_id": "A1", "authed_user": token_json() });
        let server = StubServer::json(&body.to_string());

        server
            .client()
            .fetch_auth_token("oauth-code")
            .await
            .unwrap();

        let request = server.single_request();
        let form = request.form();
        assert_eq!(request.method, "POST");
        assert_eq!(request.target, "/oauth.v2.access");
        assert_eq!(
            request.header("Content-Type"),
            Some("application/x-www-form-urlencoded")
        );
        assert_eq!(form["code"], "oauth-code");
        assert_eq!(form["client_id"], TEST_CLIENT_ID);
        assert_eq!(form["client_secret"], TEST_CLIENT_SECRET);
        assert!(form["redirect_uri"].ends_with("/integrations/slack/webhook"));
        assert!(!form.contains_key("grant_type"));
    }

    #[rocket::async_test]
    async fn test_fetch_auth_token_reads_tokens_from_authed_user() {
        // The bot token at the top level must not be used.
        let body = json!({
            "ok": true,
            "access_token": "xoxb-bot-token",
            "token_type": "bot",
            "authed_user": token_json()
        });
        let server = StubServer::json(&body.to_string());

        let response = server.client().fetch_auth_token("code").await.unwrap();

        assert_eq!(response.access_token, "xoxe.xoxp-user-token");
        assert_eq!(response.refresh_token, "xoxe-1-refresh");
        assert_eq!(response.expires_in, 43200);
    }

    #[rocket::async_test]
    async fn test_fetch_auth_token_falls_back_to_top_level_tokens() {
        let mut body = token_json();
        body["ok"] = json!(true);
        let server = StubServer::json(&body.to_string());

        let response = server.client().fetch_auth_token("code").await.unwrap();

        assert_eq!(response.access_token, "xoxe.xoxp-user-token");
        assert_eq!(response.refresh_token, "xoxe-1-refresh");
    }

    #[rocket::async_test]
    async fn test_fetch_auth_token_without_token_rotation_is_bad_request() {
        // Without token rotation Slack sends no refresh_token or expires_in.
        let body = json!({
            "ok": true,
            "authed_user": { "id": "U1", "access_token": "xoxp-user-token" }
        });
        let server = StubServer::json(&body.to_string());

        let result = server.client().fetch_auth_token("code").await;

        assert!(matches!(result, Err(SlackRequestError::BadRequest)));
    }

    #[rocket::async_test]
    async fn test_fetch_auth_token_maps_invalid_code_to_bad_request() {
        let server = StubServer::json(r#"{"ok":false,"error":"invalid_code"}"#);

        let result = server.client().fetch_auth_token("code").await;

        assert!(matches!(result, Err(SlackRequestError::BadRequest)));
    }

    #[rocket::async_test]
    async fn test_fetch_auth_token_maps_access_denied_to_unauthorized() {
        let server = StubServer::json(r#"{"ok":false,"error":"access_denied"}"#);

        let result = server.client().fetch_auth_token("code").await;

        assert!(matches!(result, Err(SlackRequestError::Unauthorized)));
    }

    #[rocket::async_test]
    async fn test_fetch_auth_token_without_error_code_is_bad_request() {
        let server = StubServer::json(r#"{"ok":false}"#);

        let result = server.client().fetch_auth_token("code").await;

        assert!(matches!(result, Err(SlackRequestError::BadRequest)));
    }

    #[rocket::async_test]
    async fn test_refresh_token_posts_refresh_grant() {
        let mut body = token_json();
        body["ok"] = json!(true);
        let server = StubServer::json(&body.to_string());

        let response = server.client().refresh_token("xoxe-1-old").await.unwrap();

        assert_eq!(response.access_token, "xoxe.xoxp-user-token");

        let request = server.single_request();
        let form = request.form();
        assert_eq!(request.target, "/oauth.v2.access");
        assert_eq!(form["grant_type"], "refresh_token");
        assert_eq!(form["refresh_token"], "xoxe-1-old");
        assert_eq!(form["client_id"], TEST_CLIENT_ID);
        assert_eq!(form["client_secret"], TEST_CLIENT_SECRET);
        assert!(!form.contains_key("code"));
    }

    #[rocket::async_test]
    async fn test_refresh_token_maps_expired_token_to_unauthorized() {
        let server = StubServer::json(r#"{"ok":false,"error":"token_expired"}"#);

        let result = server.client().refresh_token("xoxe-1-old").await;

        assert!(matches!(result, Err(SlackRequestError::Unauthorized)));
    }
}
