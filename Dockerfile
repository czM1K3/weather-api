FROM lukechannings/deno:v1.35.2 as builder

WORKDIR /app
COPY src/ src/
RUN deno compile --allow-read --allow-net --allow-env --output server ./src/index.ts

FROM debian:stable-slim
RUN adduser --uid 1002 deno
COPY --from=builder /app/server /weather
USER deno
EXPOSE 8080

CMD ["/weather"]