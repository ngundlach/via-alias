ARG APP_NAME=via-alias
ARG DATA_DIR=/var/lib/via-alias

FROM rust:1.89-slim as builder

ARG APP_NAME
WORKDIR /app

COPY Cargo.toml Cargo.lock ./

# Create a dummy main.rs to cache dependencies
RUN mkdir src && \
  echo "fn main() {}" > src/main.rs && \
  cargo build --release && \
  rm -rf src

COPY src ./src

RUN cargo build --release

FROM debian:bookworm-slim
ARG APP_NAME
ARG DATA_DIR
ENV VIA_ALIAS_DB=${DATA_DIR}/via-alias.db

RUN apt-get update && \
  apt-get install -y ca-certificates && \
  rm -rf /var/lib/apt/lists/*

RUN useradd -m -u 1000 appuser

RUN mkdir -p ${DATA_DIR} && \
  chown -R appuser:appuser ${DATA_DIR}

WORKDIR /app

COPY --from=builder /app/target/release/${APP_NAME} ./app

RUN chown -R appuser:appuser /app

USER appuser

EXPOSE 6789

CMD ["./app"]
