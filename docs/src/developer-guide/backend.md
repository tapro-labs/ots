# Backend

- Stores Secrets
- Handles Slack Endpoints
- Does not store unencrypted data

### Built with

- [Rust](https://www.rust-lang.org/)
- [Rocket](https://rocket.rs/)

### Essential Tools

#### Required:

- [rustup](https://www.rust-lang.org/tools/install)
- [cargo-watch](https://crates.io/crates/cargo-watch)
  - *If Cargo Watch is failing on macOS. Make sure you are with the latest version of XCode*
- [Clippy](https://github.com/rust-lang/rust-clippy)
- [GNU make](https://www.gnu.org/software/make/manual/make.html)

---

#### Optional

- [mdBook](https://github.com/rust-lang/mdBook)

### Setup

- Install all [Essential Tools](#essential-tools)
- Go to backend directory `cd ./backend`
- Copy `.env.example` to `.env`. And fill out all the empty values (if needed).
- To start a dev server run `make dev`. This will compile the binary and start a server **(Default port is 8000)**. 
  - *Since we are using `cargo-watch`, there is no need to stop and rerun `make dev` as when a change is detected the server will recompile and start again.*
