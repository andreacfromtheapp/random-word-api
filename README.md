# Random Word API

This is my first RESTful API made with [Axum](https://github.com/tokio-rs/axum).
The proverbial itch to scratch to learn about and to develop REST API design and
development with Axum.

Its main purpose, besides learning, is to be a simple API to use with my
[Speak and Spell](https://github.com/andreacfromtheapp/elm_speakandspell) toy
project, made with Elm. This, however, didn't mean I had to limit the extent of
my learning. In fact, I took this as an opportunity to learn more. Besides REST
API concepts and improving idiomatic Rust skills, I learned a number of, new to
me, techniques, concepts, and best practices:

- [x] Use an environment file or configuration file to setup the API
- [x] Use TLS encryption (learned it and removed it. Better left to the proxy)
- [x] Authentication with database credentials
- [x] Authorization with JWT on protected endpoints
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

## See it in action

Until I find an inexpensive solution to host my API to peruse with Speak and
Spell, I put together a little demo with Docker that you can see by following
these three simple actions:

- Clone the repository:
  `git clone https://github.com/andreacfromtheapp/random-word-api.git`
- Move into the cloned repository: `cd random-word-api`
- Run `docker compose up --build`

Unfortunately Rust will take a while to compile on Docker, please be patient.
When that's done, visit `http://localhost` in your web browser and enjoy.

> [!CAUTION]
>
> This API is NOT meant for production usage. It is a toy project I used for
> learning purposes only. DO NOT DEPLOY THIS API. Do NOT bother opening GitHub
> issues either, I won't support this project any further.

## Acknowledgments

My API is inspired by <https://github.com/mcnaveen/random-words-api>, which I
initially used to use when developing my Speak and Spell toy project. Then they
closed the spigot, presumably because it was costing them too much (due to their
success and free usage).

Random Word API initial code is based on
[Code Like a Pro in Rust](https://www.manning.com/books/code-like-a-pro-in-rust);
which I own and have used to learn more about Rust, after studying
[The Book](https://doc.rust-lang.org/book/).
