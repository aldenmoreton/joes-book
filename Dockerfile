FROM rust:alpine3.18 AS builder

WORKDIR /build

RUN apk update && \
	apk upgrade --no-cache && \
	apk add pkgconfig libressl-dev musl-dev

# RUN rustup default nightly
RUN rustup target add wasm32-unknown-unknown

# RUN curl --proto '=https' --tlsv1.2 -LsSf https://github.com/leptos-rs/cargo-leptos/releases/download/0.2.0/cargo-leptos-installer.sh | sh
RUN cargo install --locked cargo-leptos

COPY . .

RUN cargo leptos build --release -vv


FROM alpine:3.18 AS runner

WORKDIR /var/www/app

RUN addgroup -S server && \
	adduser -S www-data -G server && \
	chown -R www-data:server /var/www/app

COPY --chown=www-data:server --from=builder /build/target/server/release/joes_book ./server/joes_book
COPY --chown=www-data:server --from=builder /build/target/front/wasm32-unknown-unknown/release/joes_book.wasm ./front/joes_book.wasm
COPY --chown=www-data:server --from=builder /build/target/site ./site

USER www-data

ENV LEPTOS_OUTPUT_NAME "joes_book"
ENV LEPTOS_SITE_ROOT "/var/www/app/site"
ENV LEPTOS_ENV "PROD"
ENV LEPTOS_SITE_ADDR "0.0.0.0:3000"

EXPOSE 3000

CMD ["./server/joes_book"]