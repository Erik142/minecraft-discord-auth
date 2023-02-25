# FROM rust:1.64 as build
# WORKDIR /app/
# RUN USER=root cargo new --bin minecraft-discord-auth
# WORKDIR /app/minecraft-discord-auth
# COPY ./Cargo.lock ./Cargo.lock
# COPY ./Cargo.toml ./Cargo.toml
# RUN cargo build --release
# RUN rm -rf ./src
# COPY ./src ./src
# RUN [ $(ls -l ./target/release/deps/minecraft-discord-auth | wc -l 2>/dev/null) -eq 0 ] || rm ./target/release/deps/minecraft-discord-auth*
# RUN [ -f ./target/release/minecraft-discord-auth ] && rm ./target/release/minecraft-discord-auth
# RUN cargo build --release
# RUN cargo install --path . --root ./out

FROM rust:1.64 as planner
WORKDIR /app
# We only pay the installation cost once,
# it will be cached from the second build onwards
RUN cargo install cargo-chef
COPY . .
RUN cargo chef prepare  --recipe-path recipe.json

FROM rust:1.64 as cacher
WORKDIR /app
RUN cargo install cargo-chef
COPY --from=planner /app/recipe.json recipe.json
RUN cargo chef cook --release --recipe-path recipe.json

FROM rust:1.64 as builder
WORKDIR /app
COPY . .
# Copy over the cached dependencies
COPY --from=cacher /app/target target
RUN cargo build --release --bin minecraft-discord-auth

FROM debian:buster-slim
WORKDIR /app/
COPY --from=builder /app/target/release/minecraft-discord-auth /app/
CMD ["/app/minecraft-discord-auth"]
