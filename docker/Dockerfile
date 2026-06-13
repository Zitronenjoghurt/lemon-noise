FROM lukemathwalker/cargo-chef:latest-rust-1 AS chef
WORKDIR /lemon-noise
RUN rustup target add wasm32-unknown-unknown
RUN curl -L --proto '=https' --tlsv1.2 -sSf https://raw.githubusercontent.com/cargo-bins/cargo-binstall/main/install-from-binstall-release.sh | bash
RUN cargo binstall -y trunk

FROM chef AS planner
COPY . .
RUN cargo chef prepare --recipe-path recipe.json

FROM chef AS builder
COPY --from=planner /lemon-noise/recipe.json recipe.json
RUN cargo chef cook --release --recipe-path recipe.json
RUN cargo chef cook --release --target wasm32-unknown-unknown -p lemon-noise-app --recipe-path recipe.json
COPY . .
RUN cargo build --release --bin lemon-noise-server

WORKDIR /lemon-noise/app
RUN trunk build --release

FROM gcr.io/distroless/cc-debian13:nonroot AS runtime
WORKDIR /lemon-noise
COPY --from=builder /lemon-noise/target/release/lemon-noise-server /usr/local/bin/server
COPY --from=builder /lemon-noise/app/dist /lemon-noise/static
ENV STATIC_DIR=/lemon-noise/static
CMD ["/usr/local/bin/server"]
