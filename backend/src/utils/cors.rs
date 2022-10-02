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
