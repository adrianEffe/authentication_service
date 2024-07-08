# Authentication Service

This is an example of an authentication service built using Rust, leveraging the `axum` web framework and `sqlx` for database interactions. The service uses RS256 (RSA Signature with SHA-256) for signing and verifying JSON Web Tokens (JWT).

## Features

- User registration, login, logout, refresh token
- JWT generation and verification
- SQLx for asynchronous database operations
- Axum for routing and middleware support
