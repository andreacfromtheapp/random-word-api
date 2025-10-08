# Random Word API

> [!CAUTION]
>
> This API is NOT meant for production usage. It's a simple project I used for
> learning purposes only!

A simple RESTful API built with Axum in Rust, created as a personal project to
dive deep into web service development and to learn a number of techniques,
concepts, and best practices. Initially designed to support my Speak and Spell
application, it evolved into a comprehensive learning experience.

The model is simple by design. It fulfilled all app's requirements, and it
allowed for a broader learning scope. This project showcases end-to-end web
service development from database design through deployment, emphasizing
security, performance, and maintainability practices expected in production
environments.

## Online demo page

> [!NOTE]
>
> The free tier shuts down after some time of inactivity or stop serving
> entirely if quota is reached. You may experience slow loading times or no demo
> at all. This is not under my control. Please be patient, if that should
> happen. Thank you for your understanding 🙏

A demo landing page is available at:
[https://word-api-axum.netlify.app](https://word-api-axum.netlify.app) 🙌

## Technical Learning Summary

I built a production-ready RESTful API in Rust using Axum to demonstrate
full-stack web development capabilities and modern API design patterns.

**Core Technologies & Architecture:**

- Rust with Axum framework for high-performance async web services
- SQLx for compile-time verified database queries (prevents SQL injection)
- JWT-based authentication with role-based access control (RBAC)
- Docker containerization with multi-service orchestration

**Security & Production Readiness:**

- Comprehensive middleware stack: compression, timeouts, rate limiting, CORS
- OWASP security headers implementation
- Request validation and extensive error handling
- Proper HTTP status code usage throughout

**Developer Experience & Documentation:**

- CLI interface with parameter validation
- Complete OpenAPI documentation with multiple UI options (Swagger, Redoc,
  Scalar)
- Comprehensive rustdoc documentation
- Environment-based configuration management

**Additional Skills Demonstrated:**

- Frontend integration with Leptos framework
- Nginx reverse proxy configuration with authentication
- Database design with user management and administrative controls
- RESTful API design following industry standards

## Omitted RESTful Requirements

For this learning-focused API, I deliberately omitted certain RESTful
specifications to maintain project scope and focus:

**HATEOAS (Hypermedia as the Engine of Application State):**

- Chose to focus on core RESTful patterns rather than metadata and link
  relationships
- Simple API schema allowed deeper exploration of fundamental concepts
- Would implement in more complex, production-scale projects

**TLS Encryption:**

- Initially implemented with Tower middleware, then removed for architectural
  reasons
- Delegated certificate management to reverse proxy (Nginx) following best
  practices
- Simplified demo experience by avoiding self-signed certificate trust issues

These omissions were strategic decisions to maximize learning depth in core
areas while maintaining realistic project scope. Future projects will
incorporate these patterns as complexity and requirements warrant.

## Available endpoints

- `/health/alive` and `/health/ready` - Public health check endpoints
- `/{lang}/random` and `/{lang}/{type}` - Public word retrieval endpoints
- `/auth` - Authentication and authorization (requires admin user)
- `/admin/{lang}/words` - Administrative CRUD endpoints (requires auth)
- `/swagger-ui`, `/redoc`, `/scalar,` `/rapidoc` - OpenAPI documentation

## Docker compose

I put together a little demo with Docker that you can run by following these
three simple actions:

- Clone the repository:
  `git clone --recursive https://github.com/andreacfromtheapp/random-word-api.git`
- Move into the cloned repository: `cd random-word-api`
- Run `docker compose up --build`

Rust takes a while on Docker, be patient. When that's ready, visit
[http://localhost:8080](http://localhost:8080) in your web browser and enjoy.

## Run locally

You could also run this API as if it was a deployed service:

- Clone the repository:
  `git clone https://github.com/andreacfromtheapp/random-word-api.git`
- Move into the API web service repository: `cd random-word-api/word-api-axum`
- Running it locally from a terminal: `just run`
- Using `curl` or similar to query the [API endpoints](#available-endpoints)
- For administrative endpoints see
  [AUTHENTICATION](AUTHENTICATION.md#usage-examples)

## Acknowledgments

Random Word API was inspired by <https://github.com/mcnaveen/random-words-api>,
which I used to use when developing my Speak and Spell toy project. Then they
closed the spigot, because it was costing them too much.
