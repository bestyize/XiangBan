# 小伴市场的运行时镜像（由 infra/images/gen-dockerfiles.py 按 infra/images/services.json 生成，不要手改）。
# 二进制与前端静态资源由构建机（ubuntu 24.04 x64）编译后放进构建上下文，布局：./server-xiaoban-market、./web。
# 二进制动态链接 glibc，运行时基础镜像要不低于构建机的 glibc 版本。
FROM ubuntu:24.04

RUN apt-get update \
    && apt-get install -y --no-install-recommends ca-certificates curl \
    && rm -rf /var/lib/apt/lists/* \
    && useradd --uid 10001 --no-create-home --shell /usr/sbin/nologin market \
    && mkdir /data && chown market /data

WORKDIR /app
COPY server-xiaoban-market /app/server-xiaoban-market
COPY web /app/web

ENV MARKET_HOST=0.0.0.0 \
    MARKET_PORT=55010 \
    MARKET_DATA_DIR=/data \
    MARKET_WEB_DIR=/app/web

USER market
EXPOSE 55010
VOLUME /data
HEALTHCHECK --interval=15s --timeout=3s --start-period=20s --retries=5 \
    CMD curl -fsS "http://127.0.0.1:${MARKET_PORT}/api/health" || exit 1

ENTRYPOINT ["/app/server-xiaoban-market"]
