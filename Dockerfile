FROM rust:1.81.0-slim-bookworm as builder

RUN apt update && apt install -y pkg-config libssl-dev
WORKDIR /app
ADD . ./
RUN cargo build --release


FROM debian:bookworm-slim
RUN apt update \
    && apt install -y ca-certificates tzdata \
    && rm -rf /var/lib/apt/lists/*

EXPOSE 8000

ENV TZ=Etc/Prague
ENV ROCKET_ADDRESS=0.0.0.0

RUN groupadd runner && useradd -g runner runner

COPY --from=builder /app/target/release/weather-api /app

RUN chown -R runner:runner /app

USER runner

CMD ["/app"]
