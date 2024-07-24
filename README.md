# Authentication Service

This is an example of an authentication service built using Rust, leveraging the `axum` web framework and `sqlx` for database interactions. The service uses RS256 (RSA Signature with SHA-256) for signing and verifying JSON Web Tokens (JWT).

## Getting Started

Follow these steps to set up and run the web application:

### Prerequisites

Ensure you have the following installed on your machine:
- [Docker](https://docs.docker.com/get-docker/)

### Steps to Run the Application

1. **Start Docker Daemon**

   Make sure the Docker daemon is running on your machine. You can typically start it from your system tray or by running the following command:

   ```sh
   sudo systemctl start docker
   ```

   Alternatively, on macOS and Windows, you might need to open the Docker Desktop application.

2. **Initialize the Database**

   Run the initialization script to start the necessary containers for Postgres and Redis:

   ```sh
   ./scripts/init_db.sh
   ```

## Features

- User registration, login, logout, refresh token
- JWT generation and verification
- SQLx for asynchronous database operations
- Axum for routing and middleware support
