# TODO

## Part 1

> [!NOTE]
>
> If this was a production API, I would implement proper filesystem structure
> but...

- [x] Environment file
  - [x] Add the possibility to use `.env` file
  - [x] Check if `.env` file exists
  - [x] Check for validity of `.env` file
  - [x] Check for validity of environment variable with types
- [x] Configuration file `ConfigurationFile`
  - [x] Add the possibility to use configuration file
  - [x] Add configuration default file generation in same directory
  - [x] Improve default config creation with default Trait
  - [x] Add a `cli` command to generate the config on demand with a path
- [x] Documentation:
  - [x] <https://doc.rust-lang.org/rustdoc/index.html>
  - [x] Improve `rustdoc` documentation
  - [x] Improve `README.md`
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

- [ ] Split logic and `axum` and use workspace? _may not need this after all_
- [ ] Improve idiomatic code and use patterns
- [ ] Add type checked `AppConfig` for API info/version and more
- [ ] Add testing
- [ ] Add sanity checks on `UpsertWord` data? _may not need. `Json<UpsertWord>`
      sanitizes all given fields to `String`_
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
- [ ] Finalize `README.md` with useful info about it all + usage
