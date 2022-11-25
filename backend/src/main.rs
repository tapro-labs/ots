use rocket::{get, routes};

mod app;
mod integrations;

#[get("/")]
fn index() -> &'static str {
    "Hello, world!"
}

#[rocket::main]
async fn main() {
    let mut server = rocket::build().mount("/", routes![index]);

    #[cfg(debug_assertions)]
    {
        server = utils::cors::init_rocket_module(server);
    }

    server = integrations::init_routes(server);
    server = app::init_routes(server);

    let _ = server.launch().await;
}
