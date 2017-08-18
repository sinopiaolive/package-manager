## Database Setup

You'll need to install PostgreSQL to use the server. Follow your OS
specific instructions to install it, and create a database called
`registry`. You can also create a user for this database, but for
local development `postgres` should suffice.

Create a file called `.env` in the `server` directory, containing the
following:

```
DATABASE_URL=postgres://postgres@localhost/registry
# optionally
GITHUB_SECRET=<github secret, required for github auth>
GITLAB_SECRET=<github secret, required for gitlab auth>
```

Install the Diesel command line tool, to set up the database and run
migrations:

```sh
$ cargo install diesel_cli
$ cd server
$ diesel database setup
```

You should now be ready to run the server.
