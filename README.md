# Via-Alias

[![build](https://github.com/ngundlach/via-alias/actions/workflows/ci.yml/badge.svg)](https://github.com/ngundlach/via-alias/actions/workflows/ci.yml)

**Via-Alias** is a self-hosted URL shortener built with Rust. It allows you to
create short, memorable aliases for long URLs and manage them through a REST
API. Built with Axum and SQLite. Via-Alias supports multiple users and
authentication via JWT access token.

This project currently mostly serves as a platform for playing around with Rust
and Axum, but it can theoretically be used.

## Table of Contents

- [API Documentation](#api-documentation)
- [Getting started](#getting-started)
  - [Docker](#docker)
  - [Docker-compose](#docker-compose)
  - [Podman](#podman)
- [Configuration](#configuration)
- [Building with Docker](#building-with-docker)

---

## API Documentation

The OpenApi-Spec for the latest Via-Alias release is available [here](./docs/openapi.json). Via-Alias also comes with
a Swagger-UI Endpoint at

```http
/swagger-ui
```

---

## Getting started

Via-Alias is intended to run inside a container, but you can also clone the
repository and build it manually. Container images are available for tagged
releases, or you can use the included Dockerfile to build your own container
image. See [Building with Docker](#building-with-docker)

Note that the API requires to send user credentials and JWT access tokens.
So ideally it should run behind a reverse proxy that handles TLS termination
for production use.

Horizontal scaling is **not supported**, because Via-Alias currently relies on
in-memory state for user registration tokens. This might change in the future.

A secret for signing and verifying JWT access tokens must be provided.

The following command generates 64 random bytes and outputs them as a
Base64-encoded string:

```bash
openssl rand -base64 64
```

This can either be injected into the container via an environment variable
or via container runtime secrets mounted as a file to `/run/secrets/VIA_ALIAS_JWT_SECRET`

Via-Alias will print the initial Admin credentials to stdout on first start:

```
Creating admin account...
----------------------------------------
name:      admin
password:  KuELypVEA2FsvaH4qcG4+g==

!!! Remember to change the password !!!
----------------------------------------
```

The password can and should be changed via the API-Endpoint `/api/users/password`.
See [API Documentation](#api-documentation).

### Docker

The following command will pull the latest image from GitHub Container Registry:

```bash
docker run \
  --volume via-alias:/via_data/via-alias \
  --publish 6789:6789 \
  --env VIA_ALIAS_JWT_SECRET="<your secret>" \
  ghcr.io/ngundlach/via-alias:latest
```

The Port is mapped to 6789 and a named volume is mounted at `/via_data/via-alias`.
Docker only supports secrets in Swarm mode, so we will have to inject the secret
as an environment variable.

### docker-compose

Save the secret to a file:

```bash
openssl rand -base64 64 > via-alias-jwt-secret
```

Below is an example compose.yaml file for running the latest Via-Alias image
from ghcr.io:

```yaml
services:
  via-alias:
    image: ghcr.io/ngundlach/via-alias:latest
    ports:
      - "6789:6789"
    volumes:
      - via-alias:/via_data/via-alias
    secrets:
      - VIA_ALIAS_JWT_SECRET

secrets:
  VIA_ALIAS_JWT_SECRET:
    file: ./via-alias-jwt-secret

volumes:
  via-alias:
```

The secret is mounted as a file at `/run/secrets/VIA_ALIAS_JWT_SECRET`
inside the container.

### Podman

Create the secret using Podman's secret manager:

```bash
openssl rand -base64 64 | podman secret create VIA_ALIAS_JWT_SECRET -

```

The following systemd unit file can be saved to `~/.config/containers/systemd/via-alias.container`:

```ini
[Unit]
Description=Via-Alias container

[Container]
Image=ghcr.io/ngundlach/via-alias:latest
PublishPort=6789:6789
Volume=via-alias:/via_data/via-alias
Secret=VIA_ALIAS_JWT_SECRET

[Service]
Restart=always

[Install]
WantedBy=default.target
```

To reload the systemd daemon and start the service:

```bash
systemctl --user daemon-reload
systemctl --user start via-alias
```

Container logs are available via:

```bash
journalctl --user -u via-alias -f

```

---

## Configuration

You can configure Via-Alias with environment variables.

| Env                      | Description                                             | Default        |
| ------------------------ | ------------------------------------------------------- | -------------- |
| VIA_ALIAS_PORT[^1]       | The port Via-Alias is listening on                      | `6789`         |
| VIA_ALIAS_DB[^2]         | Full path to the sqlite database                        | `via-alias.db` |
| VIA_ALIAS_JWT_TTL        | Expiration time of jwt access tokens in seconds         | `900`          |
| VIA_ALIAS_JWT_SECRET[^3] | **Required:** The secret used to sign jwt access tokens | ---            |
| VIA_ALIAS_REG_TOKEN_TTL  | Expiration time of user registration tokens in seconds  | `1800`         |

[^1]: In containerized environments, this variables should not be set. Instead, configure port mappings via the container runtime.

[^2]: If using the provided images or Dockerfile, this variables should not be set.

[^3]: This variable is required. Provide it either via the `VIA_ALIAS_JWT_SECRET` environment variable or alternatively via a file at `/run/secrets/VIA_ALIAS_JWT_SECRET` injected as a secret via the container runtime.

---

## Building with Docker

Building the container-image with the provided Dockerfile will create an
alpine-based image. The API is exposed on port `6789` and the persistent
database is stored at `/via_data/via-alias/via-alias.db`.

Clone the repository:

```bash
git clone https://github.com/ngundlach/via-alias.git via-alias
cd via-alias
```

Build the image:

```bash
docker build -t via-alias .
```

Run the container:

```bash
docker run \
  --volume via-alias:/via_data/via-alias \
  --publish 6789:6789 \
  --env VIA_ALIAS_JWT_SECRET="<your secret>" \
  via-alias
```

This will run a container based on the newly created image and create a volume
named `via_alias` for persistence mounted to the required directory. The port is
mapped to `6789`.
