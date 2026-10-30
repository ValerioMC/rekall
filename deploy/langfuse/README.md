# Langfuse for Rekall

A self-hosted [Langfuse](https://langfuse.com) v3 on Docker Compose, to look at what Claude Code does
inside a Rekall session. Rekall does not need it: it is an optional place to send traces to.

Six containers (web, worker, Postgres, ClickHouse, Redis, MinIO) use about 3 GB at idle. Every port is
published on `127.0.0.1` only, and images are pinned to exact versions.

## Start

```bash
cd deploy/langfuse
cp .env.example .env     # then replace every change-me value; .env is git-ignored
docker compose up -d
```

`docker compose up` refuses to start while a value in `.env` is missing. First start runs the database
migrations and takes about a minute; the stack is ready when `http://localhost:3000/api/public/health`
answers `OK`.

## What it seeds

On the first start Langfuse creates, from `.env`, an organisation **Rekall**, a project **Rekall
sessions**, and the user `LANGFUSE_USER_EMAIL` with `LANGFUSE_USER_PASSWORD`. Sign in at
http://localhost:3000. The project's API keys are `LANGFUSE_PUBLIC_KEY` and `LANGFUSE_SECRET_KEY`. These
values are only read when the database is empty: changing them later has no effect until the volumes
are wiped.

## Send traces to it

| | |
|---|---|
| UI | `http://localhost:3000` |
| OTLP endpoint (HTTP) | `http://localhost:3000/api/public/otel` |
| Authentication | `Authorization: Basic base64(public_key:secret_key)` |

## Stop and wipe

```bash
docker compose stop          # keeps the data
docker compose down          # removes the containers, keeps the data
docker compose down -v       # removes the data too: the next start re-seeds from .env
```
