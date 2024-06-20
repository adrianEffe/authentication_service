#!/usr/bin/env bash
set -x
set -eo pipefail

# Function to wait for a service to be ready
wait_for_service() {
  local service_name=$1
  local host=$2
  local port=$3

  echo "Waiting for $service_name to be ready on $host:$port..."

  while ! nc -z $host $port; do
    sleep 1
  done

  echo "$service_name is ready!"
}

# check if postgres cli is installed
if ! [ -x "$(command -v psql)" ]; then
  echo >&2 "Error: psql is not installed."
  exit 1
fi

# check if sqlx cli is installed
if ! [ -x "$(command -v sqlx)" ]; then
  echo >&2 "Error: sqlx is not installed."
  echo >&2 "Use:"
  echo >&2 "    cargo install --version='~0.7.4' sqlx-cli --no-default-features --features rustls,postgres"
  exit 1
fi

# Allow to skip Docker if a dockerized Postgres database is already running
if [[ -z "${SKIP_DOCKER}" ]]
then
  # if a postgres container is running, print instructions to kill it and exit
  RUNNING_POSTGRES_CONTAINER=$(docker ps --filter 'name=postgres' --format '{{.ID}}')
  if [[ -n $RUNNING_POSTGRES_CONTAINER ]]; then
    echo >&2 "there is a postgres container already running, kill it with"
    echo >&2 "    docker kill ${RUNNING_POSTGRES_CONTAINER}"
    exit 1
  fi

  # if a redis container is running, print instructions to kill it and exit
  RUNNING_REDIS_CONTAINER=$(docker ps --filter 'name=redis' --format '{{.ID}}')
  if [[ -n $RUNNING_REDIS_CONTAINER ]]; then
    echo >&2 "there is a redis container already running, kill it with"
    echo >&2 "    docker kill ${RUNNING_REDIS_CONTAINER}"
    exit 1
  fi

  docker-compose up -d postgres redis
fi

# Load environment variables from .env file
if [ -f .env ]; then
  export $(grep -v '^#' .env | xargs)
fi

# Keep pinging Postgres until it's ready to accept connections
wait_for_service "Postgres" "${POSTGRES_HOST}" "${POSTGRES_PORT}"

# Keep pinging Redis until it's ready to accept connections
wait_for_service "Redis" "${REDIS_HOST}" "${REDIS_PORT}"

>&2 echo "Postgres and Redis are up and running - running migrations now!"

# SQLX
sqlx database create
sqlx migrate run

cargo sqlx prepare -- --tests

>&2 echo "Postgres has been migrated, ready to go!"
