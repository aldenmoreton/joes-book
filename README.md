<div align="center">
  
  # Joe's Book
  <b>A Place for Sports Predictions with Friends</b>

  [Visit Site Here](https://joes-book.shuttleapp.rs/)
</div>

## Dedication
This Project is dedicated to my Father-in-Law, Joe. Keep the picks coming.

## About
Joe's Book is yet another pick'em site.
The site currently centers around the college football season,
as members pick against the spread for some of the week's top football games.
The twist comes through the rating system, as players are required to rank their picks `1..=n`, either recieving that number of points if they pick correctly, or losing out on those points when they do not. With the addition of arbitrary extra-point questions, guest pickers, and league leaderboards, Joe's Book is a great choice for office pools or family groups.

## Stack
Joe's Book is a server-rendered site written in Rust. It uses the [Axum](https://github.com/tokio-rs/axum) web framework, [SQLx](https://github.com/launchbadge/sqlx) database library, and [HTMX](https://htmx.org/) for client reactivity. The production version of the app is currently hosted on [Shuttle](https://www.shuttle.rs/), but is also available in a [Docker Image](#exploration).

## Exploration
Joe's Book is not available for redistribution, but personal exploration is welcomed. A local instance can quickly be spun up using Docker:
```sh
docker compose up
```

Of course, those with Rust installed may use cargo:
```sh
cargo run --no-default-features
```

> Execution of this program depends on a valid Postgres connection string,
as well as Google OAUTH credentials. Both of which must be provided through environment variables.

## Disclaimer
Joe's Book is **NOT** a sports betting app.
Any stakes created in relation to the content of this app are not sanctioned by its creator.
By using this app, you agree to do so in such a way that follows the laws and regulations on sports betting in your area. Please pick responsibly!
