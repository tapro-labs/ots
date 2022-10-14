use rocket::http::Status;

use rocket::response::Redirect;
use rocket::serde::{json::Json, Deserialize};
use rocket::{get, post, routes, Route};

use crate::integrations::slack::auth::SlackAccessToken;
use crate::integrations::slack::client::SlackFetchUsersResponse;
use crate::integrations::slack::services::slack_authentication_service::{
    authenticate, AuthenticationStatus,
};
use crate::{logger, GlobalOptions};

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
