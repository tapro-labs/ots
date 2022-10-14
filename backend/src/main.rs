use dotenv::dotenv;
use envconfig::Envconfig;
use rocket::{get, routes};

use crate::utils::logger;

mod app;
mod integrations;
mod store;
mod utils;

#[derive(Envconfig)]
pub struct GlobalOptions {
    #[envconfig(from = "FRONTEND_SERVER_URL", default = "")]
    frontend_server_url: String,

    #[envconfig(from = "BACKEND_SERVER_URL", default = "")]
    backend_server_url: String,
}

impl Default for GlobalOptions {
    fn default() -> Self {
        Self::init_from_env().unwrap()
    }
}

impl GlobalOptions {
    pub fn is_dev(&self) -> bool {
        #[cfg(debug_assertions)]
        {
            true
        }
        #[cfg(not(debug_assertions))]
        {
            false
        }
    }

    pub fn is_prod(&self) -> bool {
        #[cfg(debug_assertions)]
        {
            false
        }
        #[cfg(not(debug_assertions))]
        {
            true
        }
    }
}

#[get("/")]
fn index() -> &'static str {
    "Hello, world!"
}

#[rocket::main]
async fn main() {
    dotenv().ok();

    let mut server = rocket::build().mount("/", routes![index]);

    #[cfg(debug_assertions)]
    {
        server = utils::cors::init_rocket_module(server);
    }

    server = integrations::init_routes(server);
    server = app::init_routes(server);

    let _ = server.launch().await;
}
