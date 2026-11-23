# Koma server — runtime-only image (build-once-repackage-many).
# The static aarch64-musl binary is cross-built by scripts/build-release.sh (zig recipe);
# nothing is compiled inside this container — the pinned device has 512MB RAM.
# Context layout produced by the script: docker/koma-server, docker/web/.

# CA bundle for outbound TLS (MangaDex via rustls needs certs at runtime).
# bookworm-slim does NOT ship ca-certificates by default — install it in the stage.
FROM debian:bookworm-slim AS certs
RUN apt-get update     && apt-get install -y --no-install-recommends ca-certificates     && rm -rf /var/lib/apt/lists/*

FROM scratch
COPY --from=certs /etc/ssl/certs/ca-certificates.crt /etc/ssl/certs/ca-certificates.crt
COPY --chmod=755 docker/koma-server /koma-server

# Separate-origins posture (owner decision 2): the server NEVER serves the web UI.
# The SPA rides its own container (docker-compose: koma-web, nginx).
ENV KOMA_DB_URL="sqlite:///data/koma.db?mode=rwc" \
    KOMA_STORAGE="/data/storage" \
    KOMA_ADDR="0.0.0.0:8080" \
    SSL_CERT_FILE="/etc/ssl/certs/ca-certificates.crt"
VOLUME /data
EXPOSE 8080
ENTRYPOINT ["/koma-server"]
