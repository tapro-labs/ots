use rocket::response::status::BadRequest;
use rocket::serde::{json::Json, Deserialize, Serialize};
use utils::{time::Time, uuid::Uuid};

use store::redis_store::RedisStore;

use rocket::{get, post, routes, Route};

#[derive(Deserialize)]
struct CreateSecretData {
    secret: String,
}

#[derive(Serialize)]
struct CreateSecretResponse {
    #[serde(rename = "secretId")]
    id: Uuid,
}

impl CreateSecretResponse {
    fn new(id: Uuid) -> Self {
        Self { id }
    }
}

#[derive(Serialize)]
struct GetSecretResponse {
    secret: String,
}

impl GetSecretResponse {
    fn new(secret: String) -> Self {
        Self { secret }
    }
}

#[post("/secrets", format = "json", data = "<data>")]
fn store_secret(
    data: Json<CreateSecretData>,
) -> Result<Json<CreateSecretResponse>, BadRequest<String>> {
    let mut redis_store = RedisStore::connect_default();
    let id = Uuid::new_v4();

    match redis_store.store_for_a_period(format!("secret:{}", id), &data.secret, Time::from_days(1))
    {
        Ok(_) => Ok(Json(CreateSecretResponse::new(id))),
        _ => Err(BadRequest(None)),
    }
}

#[get("/secrets/<secret>")]
fn get_secret(secret: String) -> Option<Json<GetSecretResponse>> {
    let mut redis_store = RedisStore::connect_default();
    let key = format!("secret:{}", secret);

    match redis_store.get_and_forget(key) {
        Ok(value) => Some(Json(GetSecretResponse::new(value))),
        _ => None,
    }
}

pub fn routes() -> Vec<Route> {
    routes![get_secret, store_secret]
}
