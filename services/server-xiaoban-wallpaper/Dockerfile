FROM node:24-bookworm-slim AS frontend
WORKDIR /build/frontend
COPY frontend/package*.json ./
RUN npm ci
COPY frontend/ ./
RUN npm run build

FROM rust:1-bookworm AS backend
WORKDIR /build/backend
RUN apt-get update && apt-get install -y --no-install-recommends pkg-config libssl-dev && rm -rf /var/lib/apt/lists/*
COPY backend/ ./
RUN cargo build --release --locked

FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y --no-install-recommends ca-certificates libssl3 && rm -rf /var/lib/apt/lists/* && useradd --system --uid 10001 xiaoban
COPY --from=backend /build/backend/target/release/xiaoban-wallpaper-server /usr/local/bin/xiaoban-wallpaper-server
COPY --from=frontend /build/frontend/dist /app/frontend
ENV HOST=0.0.0.0 PORT=55006 FRONTEND_DIST=/app/frontend
USER xiaoban
EXPOSE 55006
ENTRYPOINT ["xiaoban-wallpaper-server"]
