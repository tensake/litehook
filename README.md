# Litehook

[![Rust](https://img.shields.io/badge/Rust-%23000000.svg?e&logo=rust&logoColor=white)](https://www.rust-lang.org/)
[![Docker](https://img.shields.io/badge/Docker-2496ED?logo=docker&logoColor=fff)](https://hub.docker.com/r/kitfc/litehook)
[![Build Status](https://img.shields.io/github/actions/workflow/status/tensake/litehook/test.yml)](https://github.com/tensake/litehook/actions)
[![Release](https://img.shields.io/github/v/release/tensake/litehook)](https://github.com/tensake/litehook/releases)
[![Ask DeepWiki](https://deepwiki.com/badge.svg)](https://deepwiki.com/tensake/litehook)

Litehook is a self-hosted social media monitoring tool and a webhook server. Supports public and private channels or even DMs if you use a self-bot. It has support for SOCKS proxies, Docker deployment, has HTTP API and includes a lightweight web dashboard.

## Dashboard Screenshot

![Dashboard Screenshot](https://10ku.net/litehook/demo/dashboard-v3.0.png)

## Features

| Platform | Scraper Support | Self-Bot Support |
| -------- | --------------- | ---------------- |
| Telegram | ✅              | ✅               |

## Quick start

1. Create `docker-compose.yml`:

   ```yaml
   services:
     litehook:
       image: ghcr.io/tensake/litehook:latest
       container_name: litehook
       ports:
         - "4101:4101"
       environment:
         - WEBHOOK_SECRET=put_something_random_here
       volumes:
         - ./data:/data:rw
       restart: unless-stopped
   ```

2. Run `docker compose up -d`.

3. Open <http://localhost:4101/> to see the dashboard.

## How it works

Litehook works by scraping public telegram channels at a set interval, which doesn't require any authorization, or authenticate with user account to get all DMs and private channels. It saves posts to the database and sends webhook if the post is new. You can see the [Webhook Documentation](#webhook-documentation) below. You can also setup [Environment Variables](#environment-variables) for litehook.

## Build from source

### Requirements

- [Rust](https://rust-lang.org/tools/install/) version 1.93.0 or higher
- [Cargo](https://doc.rust-lang.org/cargo/getting-started/installation.html) version 1.93.0 or higher
- **Libraries**: libssl3, libstdc++6, libc++1, glibc >= 2.38

> [!TIP]
> You can install Rust and Cargo by using [rustup](https://rustup.rs/).

### Build and run with

```bash
cargo run --release
```

## Environment Variables

Environment variables used by litehook, for example in your `.env` file in the same directory as the litehook binary.

| Environment Variable | Description                                                 |
| -------------------- | ----------------------------------------------------------- |
| PORT                 | Port for web interface, default is `4101`                   |
| WEBHOOK_SECRET       | Webhook secret in `x-secret` header                         |
| PROXY_LIST_URL       | URL to SOCKS5 proxy list                                    |
| DB_PATH              | Path to SQLite database file, default is `data/litehook.db` |

> [!TIP]
> You can try using [IPLocate proxy list](https://github.com/iplocate/free-proxy-list).
> Be aware that proxy can be slow and timeout the HTTP request.

## Webhook Documentation

Webhook will be sent to webhook url with `POST` method, the server must return a `2xx` HTTP status code, otherwise the webhook will be retried 4 additional times with a 1 second interval. If all retries fail, the data is still stored in the database and webhook will be dropped.

Webhook request will include a `x-secret` header with the webhook secret from `WEBHOOK_SECRET` environment variable that **you should verify on server before trusting the payload**.

Example of the webhook payload:

```json
{
  "channel": {
    "id": "str",
    "name": "str",
    "image": "https://...",
    "counters": {
      "subscribers": "1.2M",
      "photos": "392",
      "videos": "104",
      "links": "39"
    },
    "description": "str"
  },
  "new_posts": [
    {
      "id": "channel_id/post_id",
      "author": "str",
      "text": "str",
      "media": ["https://...", "https://..."],
      "reactions": [
        {
          "emoji": "♥",
          "count": "35"
        }
      ],
      "views": "13.4K",
      "date": "2026-03-04T12:00:00Z"
    }
  ]
}
```
