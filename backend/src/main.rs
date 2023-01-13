

mod app;
mod integrations;

#[rocket::main]
async fn main() {
    let mut server = rocket::build();

    #[cfg(debug_assertions)]
    {
        server = utils::cors::init_rocket_module(server);
    }

    server = utils::versioning::init_rocket_module(server);
    server = integrations::init_routes(server);
    server = app::init_routes(server);

    let _ = server.launch().await;
}
