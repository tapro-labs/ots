use rocket::{Build, Rocket};
use std::sync::Mutex;
use store::redis_store::RedisStore;
use store::SecretStore;
use utils::global_options::GlobalOptions;
use utils::logger::info;

mod app;
mod integrations;

/// Builds the rocket instance with all fairings, routes and the given secret store.
fn build_rocket(store: Box<dyn SecretStore>) -> Rocket<Build> {
    let mut server = rocket::build();

    #[cfg(debug_assertions)]
    {
        server = utils::cors::init_rocket_module(server);
    }

    server = utils::versioning::init_rocket_module(server);
    server = integrations::init_routes(server);

    let store: Mutex<Box<dyn SecretStore>> = Mutex::new(store);
    server = server.manage(store);

    app::init_routes(server)
}

#[rocket::main]
async fn main() {
    let server = build_rocket(Box::new(RedisStore::connect_default()));

    info(format!(
        "App version is: {}",
        GlobalOptions::default().build_version
    ));

    let _ = server.launch().await;
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::app::secrets::test_support::InMemoryStore;
    use rocket::http::{ContentType, Status};
    use rocket::local::blocking::Client;

    fn client() -> Client {
        Client::tracked(build_rocket(Box::<InMemoryStore>::default()))
            .expect("valid rocket instance")
    }

    fn has_route(client: &Client, method: &str, path: &str) -> bool {
        client
            .rocket()
            .routes()
            .any(|r| r.method.as_str() == method && r.uri.path() == path)
    }

    #[test]
    fn secret_routes_are_mounted_under_api() {
        let client = client();
        assert!(has_route(&client, "POST", "/api/secrets"));
        assert!(has_route(&client, "GET", "/api/secrets/<secret>"));
    }

    #[test]
    fn full_secret_flow_through_built_rocket() {
        let client = client();
        let response = client
            .post("/api/secrets")
            .header(ContentType::JSON)
            .body(r#"{"secret":"payload"}"#)
            .dispatch();
        assert_eq!(response.status(), Status::Ok);
        let body: rocket::serde::json::Value = response.into_json().expect("json body");
        let id = body["secretId"]
            .as_str()
            .expect("secretId string")
            .to_owned();

        let first = client.get(format!("/api/secrets/{}", id)).dispatch();
        assert_eq!(first.status(), Status::Ok);
        assert_eq!(first.into_string().unwrap(), r#"{"secret":"payload"}"#);

        let second = client.get(format!("/api/secrets/{}", id)).dispatch();
        assert_eq!(second.status(), Status::NotFound);
    }

    #[test]
    fn responses_carry_version_header() {
        let client = client();
        let response = client.get("/api/secrets/missing").dispatch();
        let version = response
            .headers()
            .get_one("X-Api-Version")
            .expect("version header");
        assert_eq!(version, GlobalOptions::default().build_version);
        assert!(!version.is_empty());
    }

    #[test]
    fn unknown_route_returns_404_with_version_header() {
        let client = client();
        let response = client.get("/does-not-exist").dispatch();
        assert_eq!(response.status(), Status::NotFound);
        assert!(response.headers().get_one("X-Api-Version").is_some());
    }

    #[cfg(debug_assertions)]
    #[test]
    fn debug_build_answers_cors_preflight() {
        let client = client();
        let response = client.options("/api/secrets").dispatch();
        assert_eq!(response.status(), Status::Ok);
        let headers = response.headers();
        assert_eq!(headers.get_one("Access-Control-Allow-Origin"), Some("*"));
        assert_eq!(
            headers.get_one("Access-Control-Allow-Methods"),
            Some("POST, GET, PATCH, OPTIONS")
        );
        assert_eq!(headers.get_one("Access-Control-Allow-Headers"), Some("*"));
    }

    #[cfg(feature = "slack_feature")]
    #[test]
    fn slack_routes_are_mounted_when_feature_enabled() {
        let client = client();
        assert!(has_route(&client, "GET", "/api/integrations/slack/users"));
        assert!(has_route(
            &client,
            "POST",
            "/api/integrations/slack/send-message"
        ));
        assert!(has_route(&client, "GET", "/integrations/slack/webhook"));
    }

    #[cfg(not(feature = "slack_feature"))]
    #[test]
    fn slack_routes_are_absent_without_feature() {
        let client = client();
        assert!(!client
            .rocket()
            .routes()
            .any(|r| r.uri.path().contains("slack")));
    }
}
