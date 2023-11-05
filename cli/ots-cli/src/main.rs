use clap::{Arg, Command};

fn main() {
    let matches = Command::new("One Time secret CLI")
        .version("1.0")
        .author("Kaloyan Yosifov<kaloqn665@gmail.com>")
        .about("The CLI for fetching/creating secrets with ots!")
        .arg_required_else_help(true)
        .subcommand(
            Command::new("create").about("Create a secret").arg(
                Arg::new("text")
                    .value_name("text")
                    .required(true)
                    .help("The text for the creation of the secret"),
            ),
        )
        .subcommand(
            Command::new("get").about("Fetch secret").arg(
                Arg::new("id")
                    .value_name("id")
                    .required(true)
                    .help("The id of the secret to fetch"),
            ),
        )
        .get_matches();

    match matches.subcommand() {
        Some(("create", m)) => {
            println!("Creating secret! {:?}", m);
        }
        Some(("get", m)) => {
            println!("Fetching secret! {:?}", m);
        }
        _ => {}
    }
}
