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
