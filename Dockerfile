ARG APP_NAME=via-alias
ARG DATA_DIR=/via_data/via-alias

FROM rust:1.93-alpine as builder

ARG APP_NAME
WORKDIR /app

COPY Cargo.toml Cargo.lock ./

COPY src ./src
COPY migrations ./migrations

RUN cargo build --release

FROM alpine:3.23.3

ARG APP_NAME
ARG DATA_DIR
ENV VIA_ALIAS_DB=${DATA_DIR}/via-alias.db

RUN addgroup -g 1000 appuser && \
  adduser -D -u 1000 -G appuser appuser

RUN mkdir -p ${DATA_DIR} && \
  chown -R appuser:appuser ${DATA_DIR}

WORKDIR /app

COPY --from=builder /app/target/release/${APP_NAME} ./app

RUN chown -R appuser:appuser /app

USER appuser

EXPOSE 6789

VOLUME ${DATA_DIR}

CMD ["./app"]
