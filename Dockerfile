ARG RUST_VERSION=1.80.1
ARG APP_NAME=joes-book

# Build executable
FROM rust:${RUST_VERSION}-alpine AS build
ARG APP_NAME
WORKDIR /app

RUN apk add --no-cache clang lld musl-dev git pkgconfig openssl-dev openssl-libs-static

RUN --mount=type=bind,source=.sqlx,target=.sqlx \
    --mount=type=bind,source=src,target=src \
    --mount=type=bind,source=Cargo.toml,target=Cargo.toml \
    --mount=type=bind,source=Cargo.lock,target=Cargo.lock \
    --mount=type=cache,target=/app/target/ \
    --mount=type=cache,target=/usr/local/cargo/git/db \
    --mount=type=cache,target=/usr/local/cargo/registry/ \
cargo build --locked --release --no-default-features && \
cp ./target/release/$APP_NAME /bin/server

FROM node:18.13.0 AS styles

RUN --mount=type=bind,source=package.json,target=package.json \
    --mount=type=bind,source=tailwind.config.js,target=tailwind.config.js \
    --mount=type=bind,source=style,target=style \
    --mount=type=bind,source=src,target=src \
    --mount=type=cache,target=node_modules \
    npx tailwindcss -i style/input.css -o /bookie.css

FROM alpine:3.18 AS final

ARG UID=10001
RUN adduser \
    --disabled-password \
    --gecos "" \
    --home "/nonexistent" \
    --shell "/sbin/nologin" \
    --no-create-home \
    --uid "${UID}" \
    appuser
USER appuser

# Copy the public folder
COPY ./public /public

# Copy the executable from the "build" stage.
COPY --from=build /bin/server /bin/server/

# Copy styles
COPY --from=styles /bookie.css /public/styles/bookie.css

EXPOSE 8000

CMD ["/bin/server/server"]
