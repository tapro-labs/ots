use rocket::{Build, Rocket};

pub fn init_routes(server: Rocket<Build>) -> Rocket<Build> {
    init_slack_routes(server)
}

fn init_slack_routes(server: Rocket<Build>) -> Rocket<Build> {
    #[cfg(not(feature = "slack_feature"))]
    return server;

    #[cfg(feature = "slack_feature")]
    return server
        .mount("/integrations", slack::routes::integration_routes())
        .mount("/api/integrations", slack::routes::api_routes());
}
