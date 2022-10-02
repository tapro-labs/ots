use envconfig::Envconfig;
use rocket::response::status::BadRequest;
use rocket::response::Redirect;
use rocket::serde::{json::Json, Deserialize};
use rocket::{get, post, routes, Route};

use crate::integrations::slack::auth::SlackAccessToken;
use crate::integrations::slack::client::SlackFetchUsersResponse;
use crate::store::redis_store::RedisStore;
use crate::utils::{time::Time, uuid::Uuid};
use crate::GlobalOptions;

use super::client::{Client, ClientOptions};

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
) -> Result<Json<SlackFetchUsersResponse>, BadRequest<()>> {
    let client = Client::new(ClientOptions::init_from_env().unwrap());

    match client.fetch_users(&access_token.access_token).await {
        Ok(response) => Ok(Json(response)),
        _ => Err(BadRequest(None)),
    }
}

#[post("/slack/send-message", format = "json", data = "<data>")]
async fn send_message(
    access_token: SlackAccessToken,
    data: Json<SendMessageData>,
) -> Result<Json<()>, BadRequest<()>> {
    let client = Client::new(ClientOptions::init_from_env().unwrap());

    match client
        .send_message(&access_token.access_token, &data.channel_id, &data.message)
        .await
    {
        Ok(response) => Ok(Json(response)),
        _ => Err(BadRequest(None)),
    }
}

#[get("/slack/webhook?<code>")]
async fn handle_webhook(code: &str) -> Result<Redirect, BadRequest<()>> {
    let client = Client::new(ClientOptions::init_from_env().unwrap());

    match client.fetch_auth_token(code).await {
        Ok(response) => {
            let mut redis_store = RedisStore::connect_default();
            // TODO-SESSIONS
            // We create a session id on demand
            // in the future we might do this with initial load of our site
            let session_id = Uuid::new_v4();

            match redis_store.store_for_a_period(
                format!("session:{}", session_id),
                response.access_token,
                // TODO-SLACK-ACCESS
                // set it to smaller days
                // and use refresh tokens
                // for now we just rely on 401 returned
                // from the server and frontend asking for user to connect again
                Time::from_days(30),
            ) {
                Ok(_) => {
                    let url = format!(
                        "{}/?slack_api_token={}",
                        GlobalOptions::default().server_url,
                        session_id
                    );
                    let redirect = Redirect::to(url);

                    Ok(redirect)
                }
                _ => Err(BadRequest(None)),
            }
        }
        _ => Err(BadRequest(None)),
    }
}
