# 论坛（超级小伴）的运行时镜像（由 infra/images/gen-dockerfiles.py 按 infra/images/services.json 生成，不要手改）。
# 二进制与前端静态资源由构建机（ubuntu 24.04 x64）编译后放进构建上下文，布局：./server-forum、./web。
# 二进制动态链接 glibc，运行时基础镜像要不低于构建机的 glibc 版本。
FROM ubuntu:24.04

RUN apt-get update \
    && apt-get install -y --no-install-recommends ca-certificates curl \
    && rm -rf /var/lib/apt/lists/* \
    && useradd --uid 10004 --no-create-home --shell /usr/sbin/nologin forum \
    && mkdir /data && chown forum /data

WORKDIR /app
COPY server-forum /app/server-forum
COPY web /app/web

ENV FORUM_HOST=0.0.0.0 \
    FORUM_PORT=55011 \
    FORUM_UPLOAD_DIR=/data/uploads \
    FORUM_WEB_DIR=/app/web

USER forum
EXPOSE 55011
VOLUME /data
HEALTHCHECK --interval=15s --timeout=3s --start-period=20s --retries=5 \
    CMD curl -fsS "http://127.0.0.1:${FORUM_PORT}/api/health" || exit 1

ENTRYPOINT ["/app/server-forum"]
