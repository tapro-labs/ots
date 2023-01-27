use utils::global_options::GlobalOptions;
use utils::logger::info;

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

    info(format!("App version is: {}", GlobalOptions::default().build_version));

    let _ = server.launch().await;
}
