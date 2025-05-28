FROM rust:alpine3.21 as builder
RUN apk add --no-cache musl-dev

WORKDIR /app
COPY . .
RUN cargo build --release --target x86_64-unknown-linux-musl


# Use a minimal base image for the final image
FROM alpine:3.21.3
WORKDIR /app
COPY --from=builder /app/target/x86_64-unknown-linux-musl/release/ip_webhook /app/ip_webhook
RUN chmod +x /app/ip_webhook
CMD ["/app/ip_webhook"]