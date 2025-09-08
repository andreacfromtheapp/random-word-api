# Random Word API

> [!CAUTION]
>
> This API is NOT meant for production usage. It is a toy project I used for
> learning purposes only. DO NOT DEPLOY THIS API. Do NOT bother opening GitHub
> issues either, I won't support this project any further.

This is my first RESTful API made with [Axum](https://github.com/tokio-rs/axum).
The proverbial itch to scratch to learn about and to develop REST API design and
development with Axum. Its main purpose, besides learning, is to be a simple API
to use with my
[Speak and Spell](https://github.com/andreacfromtheapp/elm_speakandspell) toy
project, made with Elm. This, however, didn't mean I had to limit the extent of
my learning. In fact, I took this as an opportunity to learn more. Besides REST
API concepts and improving idiomatic Rust skills, I learned a number of, new to
me, techniques, concepts, and best practices:

- [x] Use an environment file or configuration file to setup the API
- [x] Use TLS encryption (learned it and removed it. Better left to the proxy)
- [x] Proper `rustdoc` crate documentation (run `cargo doc --open` from within
      the `word-api-axum` directory)
- [x] Authentication with database credentials for administrative endpoints
- [x] Authorization with JWT on protected administrative endpoints
- [x] Middleware pattern with:
  - [x] Compression
  - [x] Requests timeout
  - [x] Security headers
  - [x] Request limiting
  - [x] Body size limiting
  - [x] Requests rate limiting
  - [x] CORS (only allow certain verbs on each endpoint)
  - [x] Tracing (for API logging)
- [x] [Open API](https://www.openapis.org/) documentation with:
  - [x] [Swagger UI](https://swagger.io/tools/swagger-ui/)
  - [x] [Redoc](https://redocly.com/)
  - [x] [Scalar](https://scalar.com/)
  - [x] [RapiDoc](https://rapidocweb.com/)
- [x] Developed a simple landing page with `Leptos (CSR)` for demo purposes
- [x] Containerized everything with Docker Compose for demo purposes
- [x] Password protected OpenAPI endpoints with Nginx (user and password: admin)

## Available endpoints

- `/health/alive` and `/health/ready` - Public health check endpoints
- `/{lang}/random` and `/{lang}/{type}` - Public word retrieval endpoints
- `/auth` - Authentication and authorization (requires admin user)
- `/admin/{lang}/words` - Administrative CRUD endpoints (requires auth)
- `/swagger-ui`, `/redoc`, `/scalar,` `/rapidoc` - OpenAPI documentation

## See it in action

### Docker compose

Until I find an inexpensive solution to host my API to peruse with Speak and
Spell, I put together a little demo with Docker that you can see by following
these three simple actions:

- Clone the repository:
  `git clone https://github.com/andreacfromtheapp/random-word-api.git`
- Move into the cloned repository: `cd random-word-api`
- Run `docker compose up --build`

Unfortunately Rust will take a while to compile on Docker, please be patient.
When that's done, visit [http://localhost](http://localhost) in your web browser
and enjoy.

### Run locally

You could also see this API run as if it was deployed by:

- Clone the repository:
  `git clone https://github.com/andreacfromtheapp/random-word-api.git`
- Move into the API web service repository: `cd random-word-api/word-api-axum`
- Running it locally from a terminal (see `justfile` commands)
- Using `curl` or similar to query the API endpoints:
  - For admin endpoints see [AUTHENTICATION](/AUTHENTICATION.md#usage-examples)
  - For public endpoints run `curl` GET requests
  - For OpenAPI endpoints append to <http://localhost>:
    - `/swagger-ui`, `/redoc`, `/scalar,` `/rapidoc`

### Run Elm Speak and Spell

To see this in action:

- Clone the repository:
  `git clone https://github.com/andreacfromtheapp/random-word-api.git`
- Move into the API web service repository: `cd random-word-api/word-api-axum`
  and run the API: `cargo r`
- In a new terminal window/tab move into the `fe-elm_speakandspell` directory
  and:
  - Set the backend local environment variable `VITE_APP_URL` to
    `"http://localhost:3000"`
  - Run the app with `npm run dev`
  - Browse [http://localhost:5173/](http://localhost:5173/)

## Acknowledgments

My API is inspired by <https://github.com/mcnaveen/random-words-api>, which I
initially used to use when developing my Speak and Spell toy project. Then they
closed the spigot, presumably because it was costing them too much (due to their
success and free usage).

Random Word API initial code is based on
[Code Like a Pro in Rust](https://www.manning.com/books/code-like-a-pro-in-rust);
which I own and have used to learn more about Rust, after studying
[The Book](https://doc.rust-lang.org/book/).
