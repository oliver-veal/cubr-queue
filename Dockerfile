FROM alpine:3.16 as builder

RUN apk update && apk add --no-cache ca-certificates

FROM scratch

COPY --from=builder /etc/ssl/certs/ca-certificates.crt /etc/ssl/certs/
COPY target/x86_64-unknown-linux-musl/release/queue /queue

ENTRYPOINT ["/queue"]