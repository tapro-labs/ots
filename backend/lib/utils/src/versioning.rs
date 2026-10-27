use rocket::{
    fairing::{Fairing, Info, Kind},
    http::Header,
    Build, Request, Response, Rocket,
};

use crate::global_options::GlobalOptions;

struct Versioning;

#[rocket::async_trait]
impl Fairing for Versioning {
    fn info(&self) -> Info {
        Info {
            name: "Attaching versioning headers!",
            kind: Kind::Response,
        }
    }

    async fn on_response<'r>(&self, _request: &'r Request<'_>, response: &mut Response<'r>) {
        response.set_header(Header::new(
            "X-Api-Version",
            GlobalOptions::default().build_version,
        ));
    }
}

pub fn init_rocket_module(server: Rocket<Build>) -> Rocket<Build> {
    server.attach(Versioning)
}

#[cfg(test)]
mod tests {
    use super::*;
    use rocket::http::Status;
    use rocket::local::blocking::Client;
    use rocket::{get, routes};

    #[get("/ping")]
    fn ping() -> &'static str {
        "pong"
    }

    fn client() -> Client {
        let server = rocket::build().mount("/", routes![ping]);

        Client::tracked(init_rocket_module(server)).expect("valid rocket instance")
    }

    fn expected_version() -> String {
        GlobalOptions::default().build_version
    }

    #[test]
    fn test_successful_response_has_version_header() {
        let client = client();
        let response = client.get("/ping").dispatch();

        assert_eq!(response.status(), Status::Ok);
        assert_eq!(
            response.headers().get_one("X-Api-Version"),
            Some(expected_version().as_str())
        );
        assert_eq!(response.into_string().as_deref(), Some("pong"));
    }

    #[test]
    fn test_error_response_has_version_header() {
        let client = client();
        let response = client.get("/missing").dispatch();

        assert_eq!(response.status(), Status::NotFound);
        assert_eq!(
            response.headers().get_one("X-Api-Version"),
            Some(expected_version().as_str())
        );
    }

    #[test]
    fn test_version_header_is_never_empty() {
        let client = client();
        let response = client.get("/ping").dispatch();
        let version = response.headers().get_one("X-Api-Version").unwrap();

        assert!(!version.is_empty());
    }

    #[test]
    fn test_version_header_is_set_once() {
        let client = client();
        let response = client.get("/ping").dispatch();

        assert_eq!(response.headers().get("X-Api-Version").count(), 1);
    }
}
