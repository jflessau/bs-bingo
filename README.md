# bs-bingo

Realtime multiplayer bullshit bingo for the web.

Stack: [Rust](https://www.rust-lang.org/), [Postgres](https://www.postgresql.org/), [Svelte](https://svelte.dev/).

[Demo](https://bingo.jflessau.com)

## Development

Rename `./api/.example-env` to `.env`.

Spin up a postgres database with `docker-compose up` and use [sqlx](https://crates.io/crates/sqlx-cli) to run the migrations: `sqlx migrate run`.

Start the frontend dev server with `npm run dev` and run the API service with `cargo run`.

Happy hacking :)
