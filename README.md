# Rust Backend

Look, I love coding in typescript just as much as the next guy, but just getting it running was a bit too much of a nightmare for me

## Installation

Run the following and follow the onscreen instructions

```
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

OR, check out the rust website
https://www.rust-lang.org/tools/install

## Setup

### Rocket.toml
You've got to set up a `Rocket.toml` file in the root directory, containing
something like this
```
[default]
secret_key = "<redacted - generate with `openssl rand -base64 32`>"

[default.databases.rust]
url = "postgres://rileybell@localhost/rust"

[default.tls]
certs = "/etc/ssl/server.crt"
key = "/etc/ssl/server.key"
```

### .env
You'll also need to make a `.env` file containing a definition for `DATABASE_URL`. 

This one should be a URL, and should look something like postgresql://username@host/dbname
In my case, it's postgresql://rileybell@localhost/dbname". This is used for
- SQLX's `query!` macro
- Connecting to the database itself

```
DATABASE_URL="<REDACTED>"
```

## Running

## Nginx quirks

`sudo chown -R <your_admin_user>:admin client_body_temp/` might help if you're getting an error like this
2023/10/12 14:57:55 [crit] 35613#0: *84629 open() "/opt/homebrew/var/run/nginx/client_body_temp/0000000092" failed (13: Permission denied), client: 127.0.0.1, server: dev.com, request: "POST /api/images HTTP/1.1", host: "dev.com", referrer: "https://dev.com/notes?id=165"
