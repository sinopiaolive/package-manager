## Installing Dependencies

Install Postgres:

* Ubuntu: `sudo apt-get install postgresql-9.5 libpq5 libpq-dev`
* Mac: `brew install postgresql`

Install cmake for building git2-rs:

* Ubuntu: `sudo apt-get install cmake`
* Mac: `brew install cmake`

## Database Setup

Configure your Postgres server to [trust connections from
localhost](https://gist.github.com/p1nox/4953113).

Create a Postgres user. For local development, `postgres` should suffice:

```sh
$ createuser postgres --createdb
```

The `--createdb` option grants the `postgres` user permission to create and drop
databases.

Create a `registry` database:

```sh
$ dropdb -U postgres registry  # if it exists already
$ createdb -U postgres registry
```

Create a file called `.env` in the `server` directory, containing the
following:

```
DATABASE_URL=postgres://postgres@localhost/registry
# optionally
GITHUB_SECRET=<github secret, required for github auth>
GITLAB_SECRET=<github secret, required for gitlab auth>
```

The GitHub secret is found on GitHub in the sinopiaolive org settings, section
[OAuth
Apps](https://github.com/organizations/sinopiaolive/settings/applications).

Install the Diesel command line tool, and reset your database. (This drops and
re-creates the `registry` database and runs all migrations.)

```sh
$ cargo install diesel_cli --force --no-default-features --features postgres
$ cd server
$ diesel database reset
```

## Running the Server

```sh
$ cd server
$ cargo run
```

## Running the Client

```sh
$ cd client
$ cargo run -- login
```

## Inspecting the Database

List tables:

```sh
$ psql -U postgres -d registry -c '\dt'
```

Inspect a table:

```sh
$ psql -U postgres -d registry -c 'select * from sometable;'
```

Add a new migration:

```sh
$ cd server
$ diesel migration generate name_for_your_migration
```

Regenerate `schema.rs` after filling in your new migration:

```sh
$ cd server
$ diesel database reset && diesel print-schema > src/schema.rs
```
