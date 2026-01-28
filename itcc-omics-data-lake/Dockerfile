FROM lukemathwalker/cargo-chef:latest-rust-bookworm AS chef
WORKDIR /app

FROM chef AS planner
COPY . .
RUN cargo chef prepare --recipe-path recipe.json

FROM chef AS builder
WORKDIR /app

COPY --from=planner /app/recipe.json recipe.json
RUN cargo chef cook --release --recipe-path recipe.json

COPY . .
RUN cargo build --release --bin itcc-omics-data-lake

FROM gcr.io/distroless/cc-debian12 AS runtime

COPY --from=builder /app/target/release/itcc-omics-data-lake /usr/local/bin/itcc-omics-data-lake

ENTRYPOINT ["/usr/local/bin/itcc-omics-data-lake"]
