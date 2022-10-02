use rocket::{Build, Rocket};

pub mod secrets;

pub fn init_routes(server: Rocket<Build>) -> Rocket<Build> {
    server.mount("/api", secrets::routes())
}
