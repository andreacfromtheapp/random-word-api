# TODO

## Part 1

- [x] Environment file
  - [x] Add the possibility to use `.env` file
  - [x] Check if `.env` file exists
  - [x] Check for validity of `.env` file
  - [x] Check for validity of environment variable with types
- [x] Configuration file (_if this was going to be a production API, I would
      implement proper Linux `/etc` and `/var` structure but..._):
  - [x] Add the possibility to use configuration file
  - [x] Add configuration default file generation in same directory
  - [x] Improve default config creation with default Trait
- [x] Documentation:
  - [x] <https://doc.rust-lang.org/rustdoc/index.html>
  - [x] Improve `rustdoc` documentation
  - [ ] Improve `README.md`
- [x] Improve tracing:
  - [x] <https://crates.io/crates/tracing-subscriber>
- [x] Errors:
  - [x] Improve errors (deprecated in favor of `anyhow`and `thiserror`):
    - [x] <https://docs.rs/sqlx/latest/sqlx/enum.Error.html>
    - [x] <https://docs.rs/http/latest/http/status/struct.StatusCode.html>
  - [x] Use `anyhow`:
    - [x] <https://crates.io/crates/anyhow>
    - [x] <https://github.com/tokio-rs/axum/tree/main/examples/anyhow-error-response>
  - [x] Use this-error:
    - [x] <https://docs.rs/thiserror/latest/thiserror/>

## Part 2

- [ ] Improve idiomatic code and use patterns
- [ ] Possibly add more to the config struct and file
- ~~[ ] Add sanity checks on endpoint data~~ _may not need. `Json<UpsertWord>`
  makes all given `fields` a `String` already_
- ~~[ ] Split logic and Axum and use workspace?~~ _may not need this after all_
- [ ] Add Testing
- [ ] Add health checks
- [ ] TLS:
  - [ ] <https://github.com/rustls/rustls/>
  - [ ] <https://github.com/rustls/rcgen>
- [ ] Authentication
- [ ] Open API:
  - [ ] <https://github.com/juhaku/utoipa>
  - [ ] <https://github.com/juhaku/utoipa/blob/master/utoipa-axum/README.md>
  - [ ] <https://github.com/juhaku/utoipa/tree/master/examples/todo-axum>
- [ ] Add Open Telemetry
  - [ ] <https://crates.io/search?q=opentelemetry>
  - [ ] <https://crates.io/crates/axum-tracing-opentelemetry>
