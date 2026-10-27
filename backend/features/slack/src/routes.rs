use rocket::http::Status;
use rocket::response::Redirect;
use rocket::serde::{json::Json, Deserialize};
use rocket::{get, post, routes, Route};
use utils::{global_options::GlobalOptions, logger};

use crate::auth::SlackAccessToken;
use crate::client::SlackFetchUsersResponse;
use crate::services::slack_authentication_service::{authenticate, AuthenticationStatus};

use super::client::Client;

#[derive(Deserialize)]
struct SendMessageData {
    #[serde(rename(deserialize = "channelId"))]
    channel_id: String,

    message: String,
}

pub fn integration_routes() -> Vec<Route> {
    routes![handle_webhook]
}

pub fn api_routes() -> Vec<Route> {
    routes![fetch_users, send_message]
}

#[get("/slack/users")]
async fn fetch_users(
    access_token: SlackAccessToken,
) -> Result<Json<SlackFetchUsersResponse>, Status> {
    let client = Client::new_defaults();

    match client.fetch_users(&access_token.access_token).await {
        Ok(response) => Ok(Json(response)),
        Err(error) => Err(error.into()),
    }
}

#[post("/slack/send-message", format = "json", data = "<data>")]
async fn send_message(
    access_token: SlackAccessToken,
    data: Json<SendMessageData>,
) -> Result<Json<()>, Status> {
    let client = Client::new_defaults();

    match client
        .send_message(&access_token.access_token, &data.channel_id, &data.message)
        .await
    {
        Ok(response) => Ok(Json(response)),
        Err(error) => Err(error.into()),
    }
}

#[get("/slack/webhook?<code>")]
async fn handle_webhook(code: &str) -> Result<Redirect, Status> {
    logger::info("[SLACK] - Handling Webhook");

    match authenticate(code).await {
        AuthenticationStatus::Ok(_, session_id) => {
            let url = format!(
                "{}/?slack_api_token={}",
                GlobalOptions::default().frontend_server_url,
                session_id
            );
            let redirect = Redirect::to(url);

            Ok(redirect)
        }
        authentication_status => Err(authentication_status.into()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rocket::http::{ContentType, Header};
    use rocket::local::blocking::Client as LocalClient;
    use rocket::serde::json::serde_json;

    // Mounted the same way as in the root crate (src/integrations/mod.rs).
    fn build_client() -> LocalClient {
        let rocket = rocket::build()
            .mount("/integrations", integration_routes())
            .mount("/api/integrations", api_routes());
        LocalClient::tracked(rocket).expect("valid rocket instance")
    }

    fn describe(routes: Vec<Route>) -> Vec<String> {
        let mut described: Vec<String> = routes
            .into_iter()
            .map(|route| format!("{} {}", route.method, route.uri.path()))
            .collect();
        described.sort();
        described
    }

    #[test]
    fn test_routes_match_frontend_urls() {
        assert_eq!(
            describe(api_routes()),
            vec!["GET /slack/users", "POST /slack/send-message"]
        );
        assert_eq!(describe(integration_routes()), vec!["GET /slack/webhook"]);
    }

    #[test]
    fn test_send_message_data_reads_camel_case_channel_id() {
        let data: SendMessageData =
            serde_json::from_str(r#"{"channelId":"C1","message":"hi"}"#).unwrap();

        assert_eq!(data.channel_id, "C1");
        assert_eq!(data.message, "hi");
        assert!(
            serde_json::from_str::<SendMessageData>(r#"{"channel_id":"C1","message":"hi"}"#)
                .is_err()
        );
    }

    // --- GET /api/integrations/slack/users ---

    #[test]
    fn test_fetch_users_without_token_is_unauthorized() {
        let client = build_client();
        let response = client.get("/api/integrations/slack/users").dispatch();

        assert_eq!(response.status(), Status::Unauthorized);
    }

    #[test]
    fn test_fetch_users_with_malformed_token_is_unauthorized() {
        let client = build_client();
        let response = client
            .get("/api/integrations/slack/users")
            .header(Header::new("Authorization", "Bearer"))
            .dispatch();

        assert_eq!(response.status(), Status::Unauthorized);
    }

    // --- POST /api/integrations/slack/send-message ---

    #[test]
    fn test_send_message_without_token_is_unauthorized() {
        let client = build_client();
        let response = client
            .post("/api/integrations/slack/send-message")
            .header(ContentType::JSON)
            .body(r#"{"channelId":"C1","message":"hi"}"#)
            .dispatch();

        assert_eq!(response.status(), Status::Unauthorized);
    }

    #[test]
    fn test_send_message_requires_json_content_type() {
        let client = build_client();
        let response = client
            .post("/api/integrations/slack/send-message")
            .header(ContentType::Form)
            .header(Header::new("Authorization", "Bearer session"))
            .body("channelId=C1&message=hi")
            .dispatch();

        assert_eq!(response.status(), Status::NotFound);
    }

    // --- GET /integrations/slack/webhook ---

    #[test]
    fn test_webhook_without_code_is_rejected_before_calling_slack() {
        let client = build_client();
        let response = client.get("/integrations/slack/webhook").dispatch();

        assert!(response.status().class().is_client_error());
    }
}
