# Random Word API

> [!CAUTION]
>
> This API is NOT meant for production usage. It's a simple project I used for
> learning purposes only!

My first [RESTful](https://restfulapi.net/rest-architectural-constraints/) API
made with [Axum](https://github.com/tokio-rs/axum). The proverbial itch to
scratch to learn REST API design and development. Its main purpose, besides
learning, is to be a simple API to use with my
[Speak and Spell](https://github.com/andreacfromtheapp/elm_speakandspell) app.
This, however, didn't limit the extent of my learning. Au contraire, this was an
opportunity to learn as much as possible about RESTful APIs, improving idiomatic
Rust skills; and to learn a number of techniques, concepts, and best practices:

- [x] CLI interface, with parameters validation, to instantiate the service
- [x] Use an environment file or configuration file to setup the API
- [x] `rustdoc` documentation (run `just doc` from within the `word-api-axum`
      directory)
- [x] Use TLS encryption (learned and removed, as it's best left to the proxy)
- [x] User database with
      [RBAC](https://en.wikipedia.org/wiki/Role-based_access_control) for users
      and administrative accounts
- [x] Authentication with database credentials for administrative endpoints
- [x] Authorization with JWT on protected administrative endpoints
- [x] Compile-time checked queries validation with
      [SQLx](https://github.com/launchbadge/sqlx?tab=readme-ov-file#sqlx-is-not-an-orm)
      to prevent SQL Injections.
- [x] Requests validation to make sure all parameters are as expected
- [x] Extensive error handling for REST and database operations
- [x] Appropriate HTTP status codes for each request case
- [x] Middleware pattern with:
  - [x] Compression for faster transfers
  - [x] Requests time out to avoid client hanging too long
  - [x] Security headers to apply restrictions and
        [OWASP](https://owasp.org/www-project-secure-headers/) security list
  - [x] Request limiting to avoid abuse
  - [x] Body size limiting to avoid abuse
  - [x] Requests rate limiting to avoid abuse
  - [x] CORS Methods restrictions to control HTTP verbs and allow only what's
        needed on each route
  - [x] CORS Origins restrictions to control which domains can access the API
  - [x] Tracing for API logging
- [x] [Open API](https://www.openapis.org/) documentation with:
  - [x] [Swagger UI](https://swagger.io/tools/swagger-ui/)
  - [x] [Redoc](https://redocly.com/)
  - [x] [Scalar](https://scalar.com/)
  - [x] [RapiDoc](https://rapidocweb.com/)
- [x] Simple landing page made with Leptos for demo purposes
- [x] Containerized everything with Docker for demo purposes
- [x] Password protected OpenAPI endpoints with Nginx (user and password: admin)

## Omitted RESTful requirements

For a simple API as this is, I have deliberately chosen to omit implementing the
following, although part of a RESTful API specs:

- **Metadata and links ([HATEOAS](https://restfulapi.net/hateoas/))**: although
  a recommendation I particularly agree with (I'm always in favor of code and
  tool being as informative and self-explaining as possible), this particular
  project API schema was always going to be simple. To allow focusing all
  head-space to internalize all the _required-by-the-spec_ concepts and API
  patterns.

- **TLS encryption**: I have learned how to implement TLS encryption with tower
  middleware and subsequently removed the functionality for two main reasons.
  The first is to leave the certificate management to the proxy (Nginx); and the
  second, purely demo related, is to not ask users to trust a self-signed cert.

I'm sure there's plenty more to learn about RESTful APIs and 1) I'll update
and/or refactor this project if/when necessary; 2) I'm going to develop other
APIs and build them _more complex and better_. This should come with
improvements for me too.

## Available endpoints

- `/health/alive` and `/health/ready` - Public health check endpoints
- `/{lang}/random` and `/{lang}/{type}` - Public word retrieval endpoints
- `/auth` - Authentication and authorization (requires admin user)
- `/admin/{lang}/words` - Administrative CRUD endpoints (requires auth)
- `/swagger-ui`, `/redoc`, `/scalar,` `/rapidoc` - OpenAPI documentation

## See it in action

### Online demo page

An easy experience landing page, for the less technically inclined, is also
available! Just visit:
[https://word-api-axum.netlify.app](https://word-api-axum.netlify.app) and enjoy
🙌

### Docker compose

I put together a little demo with Docker that you can run by following these
three simple actions:

- Clone the repository:
  `git clone https://github.com/andreacfromtheapp/random-word-api.git`
- Move into the cloned repository: `cd random-word-api`
- Run `docker compose up --build`

Rust takes a while on Docker, be patient. When that's ready, visit
[http://localhost](http://localhost) in your web browser and enjoy.

### Run locally

You could also peruse this API as if it was a deployed service:

- Clone the repository:
  `git clone https://github.com/andreacfromtheapp/random-word-api.git`
- Move into the API web service repository: `cd random-word-api/word-api-axum`
- Running it locally from a terminal: `just run`
- Using `curl` or similar to query the [API endpoints](#available-endpoints):
- For administrative endpoints see
  [AUTHENTICATION](/AUTHENTICATION.md#usage-examples)

### Run Elm Speak and Spell

To see this in action:

- Clone the repository:
  `git clone https://github.com/andreacfromtheapp/random-word-api.git`
- Move into the API web service repository: `cd random-word-api/word-api-axum`
  and run the API: `just run`
- In a new terminal move into the `fe-elm_speakandspell` directory and run the
  app with `npm run dev`
- Browse [http://localhost:5173/](http://localhost:5173/) and enjoy

## Acknowledgments

Random Word API was inspired by <https://github.com/mcnaveen/random-words-api>,
which I used to use when developing my Speak and Spell toy project. Then they
closed the spigot, because it was costing them too much.

Random Word API code initially based on
[Code Like a Pro in Rust](https://www.manning.com/books/code-like-a-pro-in-rust);
which I own and have used to learn more about Rust, after studying
[The Book](https://doc.rust-lang.org/book/).
