# Rust Backend

Look, I love coding in typescript just as much as the next guy, but just getting it running was a bit too much of a nightmare for me

## Installation

You can use the install script, or follow the instructions below. The install script is ONLY designed to be used on current Mac or Ubuntu environments

### Rust
You may need to install curl first
```bash
sudo apt install curl
```

Then, run the following and follow the onscreen instructions OR, check out the rust website
https://www.rust-lang.org/tools/install
```
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

### NGINX

#### Ubuntu

```bash
sudo apt install nginx
```
#### Mac
Requires homebrew. If you don't have homebrew, run the following
```bash
/bin/bash -c "$(curl -fsSL https://raw.githubusercontent.com/Homebrew/install/HEAD/install.sh)"
```
Then run this to install nginx
```bash
sudo brew install nginx
```
To have nginx start on startup, run
```bash
brew services start nginx
```
Then, to start nginx now run
```bash
nginx
```

### Postgres
This uses Postgres as a database. You'll need to install and configure that too. You'll need a database called `rust`, and you'll need to import the schema into it (something I've hopefully kept up to date in the repo)

If you're installing it on the same machine as we're running the rust code on, you won't have much configuring to do, otherwise you'll need to alter some stuff like `listen addresses`.

To connect to the Database, run `sudo -u postgres psql <your db's name (probably rust)>`

#### Mac
I'd reccomend downloading [postgresapp](https://postgresapp.com/), at least that's what I use, but really, download whatever you want

#### Ubuntu
[Tutorial](https://ubuntu.com/server/docs/databases-postgresql).

Install postgres
```bash
sudo apt install postgresql
```



## Setup (Configuration)

### Rocket.toml
First, setup a `Rocket.toml` file in the root directory, containing the following configuration
```
[default]
secret_key = ""

[default.databases.rust]
url = "postgres://rileybell@localhost/rust"

[default.tls]
certs = "/etc/ssl/server.crt"
key = "/etc/ssl/server.key"
```

`secret_key` should be generated with `openssl rand -base64 32`

`default.databases.rust.url` must be the url to your database (this will be the same value) placed in the .env file

`defaults.tls.certs` and `defaults.tls.key` must be paths to your HTTPS key and certificate files trusted by your computer

### .env
You'll also need to make a `.env` file containing a definition for `DATABASE_URL`. 

This one should be a URL, and should look something like postgresql://username@host/dbname
In my case, it's postgresql://rileybell@localhost/dbname". This is used for
- SQLX's `query!` macro
- Connecting to the database itself

```
DATABASE_URL=""
```

### nginx

```nginx
# IF YOU MODIFY THIS FILE, ALSO MODIFY https://rileybell0.atlassian.net/wiki/spaces/SD/pages/2129921
# If you change the server name, modify /etc/hosts too
# Also make sure to modify the certificate generation code to reflect the new server name
# version 1.3

worker_processes  1;

events {
    worker_connections  1024;
}

http {
    client_max_body_size 10M;

    server {
        listen 443 ssl;
        ssl_certificate    /etc/ssl/server.crt;
        ssl_certificate_key    /etc/ssl/server.key;

        server_name  dev.com;

        proxy_set_header Host $host;
        proxy_set_header X-Real-IP $remote_addr;
        proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;

        location /api {
            proxy_pass http://localhost:8000;
        }

        location / {
            proxy_pass https://localhost:3000;
        }
    }
}
```

## Setup (Installation)

```bash
cd "root/of/this/repository"
cargo install
```

## Running

```bash
cd "/root/of/this/repository"
cargo run
```

## Nginx quirks

`sudo chown -R <your_admin_user>:admin client_body_temp/` might help if you're getting an error like this
2023/10/12 14:57:55 [crit] 35613#0: *84629 open() "/opt/homebrew/var/run/nginx/client_body_temp/0000000092" failed (13: Permission denied), client: 127.0.0.1, server: dev.com, request: "POST /api/images HTTP/1.1", host: "dev.com", referrer: "https://dev.com/notes?id=165"

## Documentation
Further documentation is available at
https://rileybell0.atlassian.net/wiki/spaces/SD/overview

## Troubleshooting

###  I've changed my nginx configuration OR something's gone wrong
Reload nginx with this command
```bash
nginx -s reload
```

### I'm getting some sort of certificate error
- Have you generated certificates AND a CA for your device?
- Have you trusted the CA for your device?
- Have you entered the paths to the certificates you're using in the configuration files here?