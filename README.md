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

You'll also need to make a `.env` file containing the following
```
# This one should be a URL, and should look something like postgresql://username@host/dbname
# In my case, it's postgresql://rileybell@localhost/dbname". This is for sqlx's query! macro
DATABASE_URL="<REDACTED>"
```

## Running
