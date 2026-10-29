# AI 网关的运行时镜像（由 infra/images/gen-dockerfiles.py 按 infra/images/services.json 生成，不要手改）。
# 二进制与前端静态资源由构建机（ubuntu 24.04 x64）编译后放进构建上下文，布局：./server-ai、./web。
# 二进制动态链接 glibc，运行时基础镜像要不低于构建机的 glibc 版本。
FROM ubuntu:24.04

RUN apt-get update \
    && apt-get install -y --no-install-recommends ca-certificates curl \
    && rm -rf /var/lib/apt/lists/* \
    && useradd --uid 10003 --no-create-home --shell /usr/sbin/nologin ai \
    && mkdir /data && chown ai /data

WORKDIR /app
COPY server-ai /app/server-ai
COPY web /app/web

ENV AI_HOST=0.0.0.0 \
    AI_PORT=55040 \
    AI_DATA_DIR=/data \
    AI_WEB_DIR=/app/web

USER ai
EXPOSE 55040
VOLUME /data
HEALTHCHECK --interval=15s --timeout=3s --start-period=20s --retries=5 \
    CMD curl -fsS "http://127.0.0.1:${AI_PORT}/api/health" || exit 1

ENTRYPOINT ["/app/server-ai"]
