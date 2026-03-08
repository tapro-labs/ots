use rocket::response::status::Custom;
use rocket::serde::{json::Json, Deserialize, Serialize};
use rocket::{get, http::Status, post, routes, Route, State};
use std::sync::Mutex;
use store::SecretStore;
use utils::{time::Time, uuid::Uuid};

/// Allowed expiry values in seconds. Any value not in this list is rejected with 422.
const ALLOWED_EXPIRY_SECONDS: &[u64] = &[
    300,    // 5 minutes
    1800,   // 30 minutes
    3600,   // 1 hour
    14400,  // 4 hours
    43200,  // 12 hours
    86400,  // 24 hours (default)
    172800, // 2 days
    432000, // 5 days
];

const DEFAULT_EXPIRY_SECONDS: u64 = 86400;

#[derive(Deserialize)]
struct CreateSecretData {
    secret: String,
    expiry_seconds: Option<u64>,
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

#[derive(Serialize)]
struct ErrorResponse {
    message: String,
}

type StoreState = Mutex<Box<dyn SecretStore>>;

#[post("/secrets", format = "json", data = "<data>")]
fn store_secret(
    data: Json<CreateSecretData>,
    store: &State<StoreState>,
) -> Result<Json<CreateSecretResponse>, Custom<Json<ErrorResponse>>> {
    let expiry = data.expiry_seconds.unwrap_or(DEFAULT_EXPIRY_SECONDS);

    if !ALLOWED_EXPIRY_SECONDS.contains(&expiry) {
        return Err(Custom(
            Status::UnprocessableEntity,
            Json(ErrorResponse {
                message: format!(
                    "Invalid expiry_seconds value: {}. Allowed values are: {:?}",
                    expiry, ALLOWED_EXPIRY_SECONDS
                ),
            }),
        ));
    }

    let id = Uuid::new_v4();
    let key = format!("secret:{}", id);

    let mut store = store.lock().unwrap();
    match store.store_for_a_period(key, &data.secret, Time::from_seconds(expiry as usize)) {
        Ok(_) => Ok(Json(CreateSecretResponse::new(id))),
        Err(_) => Err(Custom(
            Status::InternalServerError,
            Json(ErrorResponse {
                message: "Failed to create secret".into(),
            }),
        )),
    }
}

#[get("/secrets/<secret>")]
fn get_secret(
    secret: String,
    store: &State<StoreState>,
) -> Result<Json<GetSecretResponse>, Status> {
    let key = format!("secret:{}", secret);
    let mut store = store.lock().unwrap();

    match store.get_and_forget(key) {
        Ok(value) => Ok(Json(GetSecretResponse::new(value))),
        Err(_) => Err(Status::NotFound),
    }
}

pub fn routes() -> Vec<Route> {
    routes![get_secret, store_secret]
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use mockall::mock;
    use rocket::http::ContentType;
    use rocket::local::blocking::Client;
    use utils::time::Time;

    // Define a mock for SecretStore using mockall
    mock! {
        pub SecretStore {}
        impl SecretStore for SecretStore {
            fn store_for_a_period(&mut self, key: String, value: &str, time: Time) -> Result<(), String>;
            fn get_and_forget(&mut self, key: String) -> Result<String, String>;
        }
    }

    fn build_client(mock_store: MockSecretStore) -> Client {
        let rocket = rocket::build()
            .manage(Mutex::new(Box::new(mock_store) as Box<dyn SecretStore>))
            .mount("/api", routes());
        Client::tracked(rocket).expect("valid rocket instance")
    }

    // --- POST /api/secrets ---

    #[test]
    fn test_create_secret_with_default_expiry() {
        let mut mock = MockSecretStore::new();
        mock.expect_store_for_a_period()
            .withf(|key, _value, time| key.starts_with("secret:") && time.as_seconds() == 86400)
            .times(1)
            .returning(|_, _, _| Ok(()));

        let client = build_client(mock);
        let response = client
            .post("/api/secrets")
            .header(ContentType::JSON)
            .body(r#"{"secret":"encrypted-data"}"#)
            .dispatch();

        assert_eq!(response.status(), Status::Ok);
        let body = response.into_string().unwrap();
        assert!(body.contains("secretId"));
    }

    #[test]
    fn test_create_secret_with_each_valid_expiry() {
        for &expiry in ALLOWED_EXPIRY_SECONDS {
            let mut mock = MockSecretStore::new();
            let expected_seconds = expiry as usize;
            mock.expect_store_for_a_period()
                .withf(move |key, _value, time| {
                    key.starts_with("secret:") && time.as_seconds() == expected_seconds
                })
                .times(1)
                .returning(|_, _, _| Ok(()));

            let client = build_client(mock);
            let body = format!(r#"{{"secret":"data","expiry_seconds":{}}}"#, expiry);
            let response = client
                .post("/api/secrets")
                .header(ContentType::JSON)
                .body(body)
                .dispatch();

            assert_eq!(
                response.status(),
                Status::Ok,
                "expected 200 for expiry_seconds={}",
                expiry
            );
        }
    }

    #[test]
    fn test_create_secret_with_invalid_expiry_returns_422() {
        for &bad_expiry in &[0u64, 299, 301, 999, 86401, 500000] {
            let mock = MockSecretStore::new(); // store should never be called
            let client = build_client(mock);
            let body = format!(r#"{{"secret":"data","expiry_seconds":{}}}"#, bad_expiry);
            let response = client
                .post("/api/secrets")
                .header(ContentType::JSON)
                .body(body)
                .dispatch();

            assert_eq!(
                response.status(),
                Status::UnprocessableEntity,
                "expected 422 for expiry_seconds={}",
                bad_expiry
            );
        }
    }

    #[test]
    fn test_create_secret_store_error_returns_500() {
        let mut mock = MockSecretStore::new();
        mock.expect_store_for_a_period()
            .times(1)
            .returning(|_, _, _| Err("redis unavailable".into()));

        let client = build_client(mock);
        let response = client
            .post("/api/secrets")
            .header(ContentType::JSON)
            .body(r#"{"secret":"data"}"#)
            .dispatch();

        assert_eq!(response.status(), Status::InternalServerError);
    }

    // --- GET /api/secrets/<id> ---

    #[test]
    fn test_get_secret_found() {
        let mut mock = MockSecretStore::new();
        mock.expect_get_and_forget()
            .withf(|key| key == "secret:test-id-123")
            .times(1)
            .returning(|_| Ok("encrypted-payload".into()));

        let client = build_client(mock);
        let response = client.get("/api/secrets/test-id-123").dispatch();

        assert_eq!(response.status(), Status::Ok);
        let body = response.into_string().unwrap();
        assert!(body.contains("encrypted-payload"));
    }

    #[test]
    fn test_get_secret_not_found_returns_404() {
        let mut mock = MockSecretStore::new();
        mock.expect_get_and_forget()
            .times(1)
            .returning(|_| Err("key not found".into()));

        let client = build_client(mock);
        let response = client.get("/api/secrets/unknown-id").dispatch();

        assert_eq!(response.status(), Status::NotFound);
    }
}
