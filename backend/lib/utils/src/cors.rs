use rocket::{
    fairing::{Fairing, Info, Kind},
    http::Header,
    routes, Build, Request, Response, Rocket,
};

struct Cors;

#[rocket::async_trait]
impl Fairing for Cors {
    fn info(&self) -> Info {
        Info {
            name: "Attaching CORS headers to responses",
            kind: Kind::Response,
        }
    }

    async fn on_response<'r>(&self, _request: &'r Request<'_>, response: &mut Response<'r>) {
        response.set_header(Header::new("Access-Control-Allow-Origin", "*"));
        response.set_header(Header::new(
            "Access-Control-Allow-Methods",
            "POST, GET, PATCH, OPTIONS",
        ));
        response.set_header(Header::new("Access-Control-Allow-Headers", "*"));
    }
}

#[rocket::options("/<_..>")]
fn all_options() {}

pub fn init_rocket_module(server: Rocket<Build>) -> Rocket<Build> {
    server.attach(Cors).mount("/api", routes![all_options])
}

#[cfg(test)]
mod tests {
    use super::*;
    use rocket::http::{Header, Status};
    use rocket::local::blocking::{Client, LocalResponse};
    use rocket::{get, post};

    #[get("/secrets/<_id>")]
    fn get_secret(_id: &str) -> &'static str {
        "secret"
    }

    #[post("/secrets")]
    fn create_secret() -> &'static str {
        "created"
    }

    fn client() -> Client {
        let server = rocket::build().mount("/api", routes![get_secret, create_secret]);

        Client::tracked(init_rocket_module(server)).expect("valid rocket instance")
    }

    fn assert_cors_headers(response: &LocalResponse<'_>) {
        let headers = response.headers();

        assert_eq!(headers.get_one("Access-Control-Allow-Origin"), Some("*"));
        assert_eq!(
            headers.get_one("Access-Control-Allow-Methods"),
            Some("POST, GET, PATCH, OPTIONS")
        );
        assert_eq!(headers.get_one("Access-Control-Allow-Headers"), Some("*"));
    }

    #[test]
    fn test_preflight_on_any_api_path_succeeds_with_cors_headers() {
        let client = client();

        for path in [
            "/api/secrets",
            "/api/secrets/abc",
            "/api/slack/send-message",
        ] {
            let response = client
                .options(path)
                .header(Header::new("Origin", "http://localhost:3000"))
                .header(Header::new("Access-Control-Request-Method", "POST"))
                .dispatch();

            assert_eq!(response.status(), Status::Ok, "preflight for {}", path);
            assert_cors_headers(&response);
        }
    }

    #[test]
    fn test_preflight_on_api_root_succeeds() {
        // `<_..>` also matches zero segments.
        let client = client();
        let response = client.options("/api").dispatch();

        assert_eq!(response.status(), Status::Ok);
        assert_cors_headers(&response);
    }

    #[test]
    fn test_preflight_outside_api_is_not_routed() {
        let client = client();
        let response = client.options("/other").dispatch();

        assert_eq!(response.status(), Status::NotFound);
        // The fairing still adds headers so the browser can read the error.
        assert_cors_headers(&response);
    }

    #[test]
    fn test_regular_responses_get_cors_headers() {
        let client = client();

        let get = client.get("/api/secrets/abc").dispatch();
        assert_eq!(get.status(), Status::Ok);
        assert_cors_headers(&get);

        let post = client.post("/api/secrets").dispatch();
        assert_eq!(post.status(), Status::Ok);
        assert_cors_headers(&post);
    }

    #[test]
    fn test_error_responses_get_cors_headers() {
        let client = client();
        let response = client.get("/api/missing").dispatch();

        assert_eq!(response.status(), Status::NotFound);
        assert_cors_headers(&response);
    }

    #[test]
    fn test_cors_headers_are_not_duplicated() {
        let client = client();
        let response = client.get("/api/secrets/abc").dispatch();

        let origins: Vec<_> = response
            .headers()
            .get("Access-Control-Allow-Origin")
            .collect();
        assert_eq!(origins, vec!["*"]);
    }

    #[test]
    fn test_options_route_does_not_shadow_other_methods() {
        let client = client();
        let response = client.get("/api/secrets/abc").dispatch();

        assert_eq!(response.into_string().as_deref(), Some("secret"));
    }
}
