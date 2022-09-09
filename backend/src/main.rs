use std::fmt::{Display, Formatter};

use redis::RedisResult;
use rocket::{get, post, Request, Response, routes};
use rocket::fairing::{Fairing, Info, Kind};
use rocket::http::Header;
use rocket::response::status::BadRequest;
use rocket::serde::{Deserialize, json::Json, Serialize, Serializer};
use rocket::serde::ser::SerializeStruct;

use store::redis_store::RedisStore;
use uuid::Uuid;

use crate::utils::time::Time;

mod store;
mod utils;

#[get("/")]
fn index() -> &'static str {
  "Hello, world!"
}

#[derive(Deserialize)]
struct CreateSecretData<'r> {
  pub secret: &'r str
}

struct CreateSecretResponse {
  id: CustomUUID
}

impl CreateSecretResponse {
  pub fn new(id: CustomUUID) -> Self {
    Self { id }
  }
}

impl Serialize for CreateSecretResponse {
  fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error> where S: Serializer {
    let mut state = serializer.serialize_struct("SecretResponse", 1)?;

    state.serialize_field("secretId", &self.id.to_string())?;

    state.end()
  }
}

struct GetSecretResponse {
  secret: String
}

impl GetSecretResponse {
  pub fn new(secret: String) -> Self {
    Self { secret }
  }
}

impl Serialize for GetSecretResponse {
  fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error> where S: Serializer {
    let mut state = serializer.serialize_struct("SecretResponse", 1)?;

    state.serialize_field("secret", &self.secret.to_string())?;

    state.end()
  }
}

struct CustomUUID {
  uuid: String
}

impl CustomUUID {
  pub fn new_v4() -> Self {
    return Self { uuid: Uuid::new_v4().to_string() }
  }
}

impl Display for CustomUUID {
  fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
    write!(f, "{}", self.uuid.to_string())
  }
}

#[post("/secrets", format = "json", data = "<data>")]
fn store_secret(data: Json<CreateSecretData<'_>>) -> Result<Json<CreateSecretResponse>, BadRequest<String>> {
  let mut redis_store = RedisStore::connect_default();
  let id = CustomUUID::new_v4();

  match redis_store.store_for_a_period(
    format!("secret-{}", id),
    data.secret,
    Time::from_days(1)
  ) {
    Ok(_) => Ok(Json(CreateSecretResponse::new(id))),
    _ => Err(BadRequest(None))
  }
}

#[get("/secrets/<secret>")]
fn get_secret(secret: String) -> Option<Json<GetSecretResponse>> {
  let mut redis_store = RedisStore::connect_default();
  let key = format!("secret-{}", secret);

  match redis_store.get_and_forget(key) {
    Ok(value) => Some(Json(GetSecretResponse::new(value))),
    _ => None
  }
}

pub struct CORS;

#[rocket::async_trait]
impl Fairing for CORS {
    fn info(&self) -> Info {
        Info {
            name: "Attaching CORS headers to responses",
            kind: Kind::Response
        }
    }

    async fn on_response<'r>(&self, _request: &'r Request<'_>, response: &mut Response<'r>) {
        response.set_header(Header::new("Access-Control-Allow-Origin", "*"));
        response.set_header(Header::new("Access-Control-Allow-Methods", "POST, GET, PATCH, OPTIONS"));
        response.set_header(Header::new("Access-Control-Allow-Headers", "*"));
    }
}

#[rocket::options("/<_..>")]
fn all_options() {
}

#[rocket::main]
async fn main() {

  let _ = rocket::build()
    .attach(CORS)
    .mount("/", routes![index])
    .mount("/api", routes![all_options, get_secret, store_secret])
    .launch()
    .await;
}
