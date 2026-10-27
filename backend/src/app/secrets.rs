use rocket::response::status::Custom;
use rocket::serde::{json::Json, Deserialize, Serialize};
use rocket::{get, http::Status, post, routes, Route, State};
use std::sync::{Mutex, MutexGuard, PoisonError};
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

/// Locks the store. A panic in an earlier request poisons the mutex; we recover
/// the guard so that one failed request does not break all later requests.
fn lock_store(store: &StoreState) -> MutexGuard<'_, Box<dyn SecretStore>> {
    store.lock().unwrap_or_else(PoisonError::into_inner)
}

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

    let mut store = lock_store(store);
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
    let mut store = lock_store(store);

    match store.get_and_forget(key) {
        Ok(value) => Ok(Json(GetSecretResponse::new(value))),
        Err(_) => Err(Status::NotFound),
    }
}

pub fn routes() -> Vec<Route> {
    routes![get_secret, store_secret]
}

#[cfg(test)]
pub(crate) mod test_support {
    use std::collections::HashMap;
    use store::SecretStore;
    use utils::time::Time;

    /// In-memory store with the same one-time semantics as the Redis store:
    /// a read removes the value.
    #[derive(Default)]
    pub struct InMemoryStore {
        values: HashMap<String, String>,
    }

    impl SecretStore for InMemoryStore {
        fn store_for_a_period(
            &mut self,
            key: String,
            value: &str,
            _time: Time,
        ) -> Result<(), String> {
            self.values.insert(key, value.to_owned());
            Ok(())
        }

        fn get_and_forget(&mut self, key: String) -> Result<String, String> {
            self.values
                .remove(&key)
                .ok_or_else(|| "key not found".to_owned())
        }
    }
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

    // --- HTTP-level tests with an in-memory store ---

    use super::test_support::InMemoryStore;
    use rocket::data::{Limits, ToByteUnit};
    use rocket::serde::json::Value;

    fn build_client_with_store(store: impl SecretStore + 'static) -> Client {
        let rocket = rocket::build()
            .manage(Mutex::new(Box::new(store) as Box<dyn SecretStore>))
            .mount("/api", routes());
        Client::tracked(rocket).expect("valid rocket instance")
    }

    fn create_secret(client: &Client, body: &str) -> String {
        let response = client
            .post("/api/secrets")
            .header(ContentType::JSON)
            .body(body)
            .dispatch();
        assert_eq!(response.status(), Status::Ok);
        let json: Value = response.into_json().expect("json body");
        json["secretId"]
            .as_str()
            .expect("secretId is a string")
            .to_owned()
    }

    fn assert_is_uuid_v4(id: &str) {
        assert_eq!(id.len(), 36, "unexpected id length: {}", id);
        for (i, c) in id.chars().enumerate() {
            if [8, 13, 18, 23].contains(&i) {
                assert_eq!(c, '-', "expected '-' at {} in {}", i, id);
            } else {
                assert!(
                    c.is_ascii_hexdigit() && !c.is_ascii_uppercase(),
                    "bad char in {}",
                    id
                );
            }
        }
        assert_eq!(&id[14..15], "4", "not a v4 uuid: {}", id);
    }

    #[test]
    fn test_secret_can_be_read_only_once() {
        let client = build_client_with_store(InMemoryStore::default());
        let id = create_secret(&client, r#"{"secret":"top-secret"}"#);

        let first = client.get(format!("/api/secrets/{}", id)).dispatch();
        assert_eq!(first.status(), Status::Ok);
        assert_eq!(first.content_type(), Some(ContentType::JSON));
        assert_eq!(first.into_string().unwrap(), r#"{"secret":"top-secret"}"#);

        for _ in 0..2 {
            let again = client.get(format!("/api/secrets/{}", id)).dispatch();
            assert_eq!(again.status(), Status::NotFound);
        }
    }

    #[test]
    fn test_create_response_shape() {
        let client = build_client_with_store(InMemoryStore::default());
        let response = client
            .post("/api/secrets")
            .header(ContentType::JSON)
            .body(r#"{"secret":"x"}"#)
            .dispatch();

        assert_eq!(response.status(), Status::Ok);
        assert_eq!(response.content_type(), Some(ContentType::JSON));
        let json: Value = response.into_json().unwrap();
        let object = json.as_object().expect("json object");
        assert_eq!(object.len(), 1, "unexpected body: {}", json);
        assert_is_uuid_v4(object["secretId"].as_str().unwrap());
    }

    #[test]
    fn test_each_secret_gets_its_own_id() {
        let client = build_client_with_store(InMemoryStore::default());
        let first = create_secret(&client, r#"{"secret":"one"}"#);
        let second = create_secret(&client, r#"{"secret":"two"}"#);
        assert_ne!(first, second);

        // reading one secret does not consume the other
        let response = client.get(format!("/api/secrets/{}", second)).dispatch();
        assert_eq!(response.into_string().unwrap(), r#"{"secret":"two"}"#);
        let response = client.get(format!("/api/secrets/{}", first)).dispatch();
        assert_eq!(response.into_string().unwrap(), r#"{"secret":"one"}"#);
    }

    #[test]
    fn test_secret_value_round_trips_unchanged() {
        let value = "line1\nline2 \"quoted\" \\ unicode: \u{00e9}\u{4e2d}\u{1f512}";
        let body =
            rocket::serde::json::to_string(&GetSecretResponse::new(value.to_owned())).unwrap();
        let client = build_client_with_store(InMemoryStore::default());
        let id = create_secret(&client, &body);

        let response = client.get(format!("/api/secrets/{}", id)).dispatch();
        let json: Value = response.into_json().unwrap();
        assert_eq!(json["secret"].as_str(), Some(value));
    }

    #[test]
    fn test_store_receives_prefixed_key_and_exact_value() {
        let mut mock = MockSecretStore::new();
        mock.expect_store_for_a_period()
            .withf(|key, value, _| {
                let id = key.strip_prefix("secret:").unwrap_or("");
                id.len() == 36 && value == "cipher-text"
            })
            .times(1)
            .returning(|_, _, _| Ok(()));

        let client = build_client(mock);
        create_secret(&client, r#"{"secret":"cipher-text"}"#);
    }

    #[test]
    fn test_null_expiry_uses_default() {
        let mut mock = MockSecretStore::new();
        mock.expect_store_for_a_period()
            .withf(|_, _, time| time.as_seconds() == DEFAULT_EXPIRY_SECONDS as usize)
            .times(1)
            .returning(|_, _, _| Ok(()));

        let client = build_client(mock);
        create_secret(&client, r#"{"secret":"x","expiry_seconds":null}"#);
    }

    #[test]
    fn test_unknown_fields_are_ignored() {
        let client = build_client_with_store(InMemoryStore::default());
        create_secret(&client, r#"{"secret":"x","unknown":true}"#);
    }

    #[test]
    fn test_get_unknown_id_with_real_store_returns_404() {
        let client = build_client_with_store(InMemoryStore::default());
        let response = client
            .get("/api/secrets/00000000-0000-4000-8000-000000000000")
            .dispatch();
        assert_eq!(response.status(), Status::NotFound);
    }

    #[test]
    fn test_get_without_id_does_not_hit_store() {
        // the mock has no expectations, so any store call would fail the request with 500
        let client = build_client(MockSecretStore::new());
        for path in ["/api/secrets", "/api/secrets/", "/api/secrets/a/b"] {
            let response = client.get(path).dispatch();
            assert_eq!(response.status(), Status::NotFound, "path {}", path);
        }
    }

    #[test]
    fn test_get_passes_decoded_id_to_store() {
        let mut mock = MockSecretStore::new();
        mock.expect_get_and_forget()
            .withf(|key| key == "secret:a b")
            .times(1)
            .returning(|_| Err("key not found".into()));

        let client = build_client(mock);
        let response = client.get("/api/secrets/a%20b").dispatch();
        assert_eq!(response.status(), Status::NotFound);
    }

    #[test]
    fn test_create_with_wrong_content_type_is_not_routed() {
        let client = build_client(MockSecretStore::new());
        for content_type in [ContentType::Plain, ContentType::Form, ContentType::HTML] {
            let response = client
                .post("/api/secrets")
                .header(content_type.clone())
                .body(r#"{"secret":"x"}"#)
                .dispatch();
            assert_eq!(
                response.status(),
                Status::NotFound,
                "content type {}",
                content_type
            );
        }

        let response = client
            .post("/api/secrets")
            .body(r#"{"secret":"x"}"#)
            .dispatch();
        assert_eq!(response.status(), Status::NotFound, "missing content type");
    }

    #[test]
    fn test_create_with_malformed_json_returns_400() {
        let client = build_client(MockSecretStore::new());
        for body in ["", "{", "not json", r#"{"secret":"x""#] {
            let response = client
                .post("/api/secrets")
                .header(ContentType::JSON)
                .body(body)
                .dispatch();
            assert_eq!(response.status(), Status::BadRequest, "body {:?}", body);
        }
    }

    #[test]
    fn test_create_with_invalid_fields_returns_422() {
        let client = build_client(MockSecretStore::new());
        for body in [
            r#"{}"#,
            r#"{"secret":null}"#,
            r#"{"secret":42}"#,
            r#"{"secret":["x"]}"#,
            r#"{"secret":"x","expiry_seconds":-300}"#,
            r#"{"secret":"x","expiry_seconds":"300"}"#,
            r#"{"secret":"x","expiry_seconds":300.5}"#,
            r#"["x"]"#,
        ] {
            let response = client
                .post("/api/secrets")
                .header(ContentType::JSON)
                .body(body)
                .dispatch();
            assert_eq!(
                response.status(),
                Status::UnprocessableEntity,
                "body {}",
                body
            );
        }
    }

    #[test]
    fn test_invalid_expiry_error_body_shape() {
        let client = build_client(MockSecretStore::new());
        let response = client
            .post("/api/secrets")
            .header(ContentType::JSON)
            .body(r#"{"secret":"x","expiry_seconds":1}"#)
            .dispatch();

        assert_eq!(response.status(), Status::UnprocessableEntity);
        assert_eq!(response.content_type(), Some(ContentType::JSON));
        let json: Value = response.into_json().unwrap();
        let object = json.as_object().unwrap();
        assert_eq!(object.len(), 1);
        let message = object["message"].as_str().unwrap();
        assert!(
            message.contains("Invalid expiry_seconds value: 1."),
            "{}",
            message
        );
        assert!(message.contains("86400"), "{}", message);
    }

    #[test]
    fn test_store_error_body_shape() {
        let mut mock = MockSecretStore::new();
        mock.expect_store_for_a_period()
            .returning(|_, _, _| Err("redis unavailable".into()));

        let client = build_client(mock);
        let response = client
            .post("/api/secrets")
            .header(ContentType::JSON)
            .body(r#"{"secret":"x"}"#)
            .dispatch();

        assert_eq!(response.status(), Status::InternalServerError);
        assert_eq!(response.content_type(), Some(ContentType::JSON));
        // the internal store error must not leak to the client
        assert_eq!(
            response.into_string().unwrap(),
            r#"{"message":"Failed to create secret"}"#
        );
    }

    #[test]
    fn test_secret_larger_than_rocket_default_limit_is_accepted() {
        // Rocket's built-in json limit is 1 MiB; Rocket.toml raises it to 100 MiB.
        let client = build_client_with_store(InMemoryStore::default());
        assert_eq!(
            client.rocket().config().limits.get("json"),
            Some(100.mebibytes())
        );

        let value = "a".repeat(2 * 1024 * 1024);
        let id = create_secret(&client, &format!(r#"{{"secret":"{}"}}"#, value));

        let response = client.get(format!("/api/secrets/{}", id)).dispatch();
        let json: Value = response.into_json().unwrap();
        assert_eq!(json["secret"].as_str().map(str::len), Some(value.len()));
    }

    #[test]
    fn test_secret_above_json_limit_returns_413() {
        let figment =
            rocket::Config::figment().merge(("limits", Limits::new().limit("json", 1.kibibytes())));
        let rocket = rocket::custom(figment)
            .manage(Mutex::new(
                Box::new(MockSecretStore::new()) as Box<dyn SecretStore>
            ))
            .mount("/api", routes());
        let client = Client::tracked(rocket).unwrap();

        let response = client
            .post("/api/secrets")
            .header(ContentType::JSON)
            .body(format!(r#"{{"secret":"{}"}}"#, "a".repeat(2048)))
            .dispatch();
        assert_eq!(response.status(), Status::PayloadTooLarge);
    }

    /// Panics on the first store call, then behaves like the in-memory store.
    #[derive(Default)]
    struct PanicOnceStore {
        panicked: bool,
        inner: InMemoryStore,
    }

    impl PanicOnceStore {
        fn maybe_panic(&mut self) {
            if !self.panicked {
                self.panicked = true;
                panic!("store failure");
            }
        }
    }

    impl SecretStore for PanicOnceStore {
        fn store_for_a_period(
            &mut self,
            key: String,
            value: &str,
            time: Time,
        ) -> Result<(), String> {
            self.maybe_panic();
            self.inner.store_for_a_period(key, value, time)
        }

        fn get_and_forget(&mut self, key: String) -> Result<String, String> {
            self.maybe_panic();
            self.inner.get_and_forget(key)
        }
    }

    #[test]
    fn test_store_panic_does_not_break_later_requests() {
        let client = build_client_with_store(PanicOnceStore::default());

        let response = client.get("/api/secrets/whatever").dispatch();
        assert_eq!(response.status(), Status::InternalServerError);

        // a panic while the store lock is held poisons the mutex;
        // later requests must still be served
        let id = create_secret(&client, r#"{"secret":"after-panic"}"#);
        let response = client.get(format!("/api/secrets/{}", id)).dispatch();
        assert_eq!(response.status(), Status::Ok);
        assert_eq!(
            response.into_string().unwrap(),
            r#"{"secret":"after-panic"}"#
        );
    }
}
