use rocket::http::Status;
use rocket::request::{FromRequest, Outcome};
use rocket::{async_trait, Request};

use crate::integrations::slack::auth::AccessTokenError::{Invalid, Missing};
use crate::store::redis_store::RedisStore;

#[derive(Debug)]
pub enum AccessTokenError {
    Missing,
    Invalid,
}

pub struct SlackAccessToken {
    pub access_token: String,
}

#[async_trait]
impl<'r> FromRequest<'r> for SlackAccessToken {
    type Error = AccessTokenError;

    async fn from_request(request: &'r Request<'_>) -> Outcome<Self, Self::Error> {
        let authorization = request.headers().get_one("Authorization");

        match authorization {
            Some(authorization) => {
                if !authorization.contains("Bearer") {
                    return Outcome::Failure((Status::Unauthorized, Missing));
                }

                let token: &str = authorization
                    .split("Bearer ")
                    .collect::<Vec<&str>>()
                    .get(1)
                    .unwrap();
                let mut redis_client = RedisStore::connect_default();
                let redis_key = format!("session:{}", token);

                if let Ok(access_token) = redis_client.get::<String, &str>(&redis_key) {
                    return Outcome::Success(Self { access_token });
                }

                Outcome::Failure((Status::Unauthorized, Invalid))
            }
            _ => Outcome::Failure((Status::Unauthorized, Missing)),
        }
    }
}
