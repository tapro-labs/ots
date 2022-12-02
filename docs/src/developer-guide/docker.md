# Docker

Docker is used to run a local development environment without installing any other unnecessary tools.

### Essential Tools

#### Required:

- [Docker](https://www.docker.com/)
- [GNU make](https://www.gnu.org/software/make/manual/make.html)

### Setup

#### Frontend

- Go to frontend `cd ./frontend`
- Copy `.env.docker` `.env`
- Replace env variable values with your credentials (optional)

#### Backend

- Go to backend `cd ./docker`
- Copy `.env.docker` `.env`
- Replace env variable values with your credentials (optional)

#### Start

- Go to root dir
- Run `make start_docker`. This does three things:
  - Spins up a frontend server on port 9010
  - Spins up a backend server on port 9011
  - Spins up a redis server on port 9015

#### Stop

- Go to root dir
- Stop the containers with `make stop_docker`
