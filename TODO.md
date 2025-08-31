# TODO

## Part 1

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
- [x] Improve idiomatic code:
  - [x] Add type checked `AppConfig` for API info/version and more
  - [x] Add type checked `AppState` for shared mutable state
  - [x] Add data integrity checks on `UpsertWord`
  - [x] Add `GetWord` to display only needed data to frontends
- [x] Open API:
  - [x] <https://crates.io/crates/utoipa>
  - [x] <https://crates.io/crates/utoipa-swagger-ui>
  - [x] <https://crates.io/crates/utoipa-rapidoc>
  - [x] <https://crates.io/crates/utoipa-redoc>
  - [x] <https://crates.io/crates/utoipa-scalar>
- [x] Add testing.
  - Note: adding these _late_ because as I was learning Axum and most of the API
    related tech/code, I wouldn't have known what to TDD. Now that I do, the
    next API will take advantage of TDD as well.

  - Disclaimer: I used AI and guided the process to add all tests.

- [x] Improve `rustdoc` documentation
  - Disclaimer: I used AI and guided the process to write all docs.

## Part 2

- [x] TLS (I've learned it and removed it. Better left to the proxy):
  - [x] <https://github.com/rustls/rustls/>
  - [x] <https://github.com/rustls/rcgen>
- [x] Authentication with database credentials
- [x] Authorization with JWT on protected endpoints
- [x] Middleware pattern with:
  - [x] Compression
  - [x] Timeout
  - [x] Security headers
  - [x] CORS
  - [x] Request limiting
  - [x] Body limiting
  - [x] Rate limiting
  - [x] Tracing

## Part 3

- [x] Split codebase and use a workspace:
  - [x] Add backend with `Axum`
  - [x] Add frontend with `Elm Speak and Spell` for fun UI
  - [x] Add frontend with `Leptos CSR` for random word UI + reload
  - [ ] Add frontend with `Leptos SSR` for admin with auth

## Part 4

- [ ] Docker and Compose for a demo:
  - [ ] `Nginx` for:
    - [ ] <https://localhost/> (a brief description and links to services)
    - [ ] <https://admin.localhost/> (API admin of words)
    - [ ] <https://random.localhost/> (page with random word + button)
    - [ ] <https://play.localhost/> (Elm Speak n Spell)
    - [ ] <https://swagger.localhost/> (Swagger UI)
    - [ ] <https://redoc.localhost/> (Redoc UI)
    - [ ] <https://scalar.localhost/> (Scalar UI)
    - [ ] <https://rapidoc.localhost/> (RapiDoc UI)
  - [ ] <https://cheatsheetseries.owasp.org/cheatsheets/HTTP_Headers_Cheat_Sheet.html>
  - [ ] API backend

## Part 5

- [ ] Finalize `rustdoc` crates documentation
- [ ] Finalize `openapi` documentation
- [ ] Finalize `README.md` with useful info about it all:
  - [ ] rationale
  - [ ] learning
  - [ ] deployment
  - [ ] usage
