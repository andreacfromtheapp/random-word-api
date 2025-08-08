# TODO

## Part 1

> [!NOTE]
>
> If this was a production API, I would implement proper filesystem structure
> (/etc /bin /var and so on) but...

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
  - [x] Use `thiserror`:
    - [x] <https://docs.rs/thiserror/latest/thiserror/>

## Part 2

- [ ] Split logic and `axum` and use workspace? _may not need this after all_
- [ ] Improve idiomatic code and use patterns
- [x] Add type checked `AppConfig` for API info/version and more
- [ ] Add testing
- [x] Add data integrity checks on `UpsertWord`
  - [x] `word`: can't contain spaces. Must be valid lemma
  - [x] `definition`: only valid description characters
  - [x] `pronunciation`: check for phonetic alphabet values
- [ ] Add better health checks (besides `/ready` and `/alive`)
- [ ] TLS:
  - [ ] <https://github.com/rustls/rustls/>
  - [ ] <https://github.com/rustls/rcgen>
- [ ] Authentication
- [x] Open API:
  - [x] <https://crates.io/crates/utoipa>
  - [x] <https://crates.io/crates/utoipauto>
  - [x] <https://crates.io/crates/utoipa-swagger-ui>
  - [x] <https://crates.io/crates/utoipa-rapidoc>
  - [x] <https://crates.io/crates/utoipa-redoc>
  - [x] <https://crates.io/crates/utoipa-scalar>
  - [ ] <https://crates.io/crates/utoipa-axum>
- [ ] Add Open Telemetry
  - [ ] <https://crates.io/search?q=opentelemetry>
  - [ ] <https://crates.io/crates/axum-tracing-opentelemetry>
- [ ] Finalize `rustdoc` crate documentation
- [ ] Finalize `openapi` documentation
- [ ] Finalize `README.md` with useful info about it all + usage
