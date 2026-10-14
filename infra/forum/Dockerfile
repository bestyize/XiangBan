# 只含运行时的镜像：二进制与前端产物由 CI 在 ubuntu-24.04 x64 上构建，再放进构建上下文。
# 上下文目录结构：./server-forum（release 二进制）与 ./web（前端 dist）。
# 构建机必须是 ubuntu 24.04：二进制动态链接 glibc，运行时基础镜像要不低于构建机的 glibc 版本。
FROM ubuntu:24.04

RUN apt-get update \
    && apt-get install -y --no-install-recommends ca-certificates curl \
    && rm -rf /var/lib/apt/lists/* \
    && useradd --uid 10001 --no-create-home --shell /usr/sbin/nologin forum \
    && mkdir -p /app/data/uploads \
    && chown -R forum:forum /app/data

WORKDIR /app
COPY server-forum /app/server-forum
COPY web /app/web

ENV FORUM_HOST=0.0.0.0 \
    FORUM_PORT=55010 \
    FORUM_WEB_DIR=/app/web \
    FORUM_UPLOAD_DIR=/app/data/uploads

USER forum
EXPOSE 55010
# 论坛没有专用健康接口，用首页（SPA index.html）探活。
HEALTHCHECK --interval=15s --timeout=3s --start-period=20s --retries=5 \
    CMD curl -fsS "http://127.0.0.1:${FORUM_PORT}/" || exit 1

ENTRYPOINT ["/app/server-forum"]
