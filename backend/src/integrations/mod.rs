use rocket::{Build, Rocket};

pub mod slack;

pub fn init_routes(server: Rocket<Build>) -> Rocket<Build> {
    server
        .mount("/integrations", slack::routes::integration_routes())
        .mount("/api/integrations", slack::routes::api_routes())
}
