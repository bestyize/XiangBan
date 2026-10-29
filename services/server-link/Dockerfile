# 手机与桌面互联中转的运行时镜像（由 infra/images/gen-dockerfiles.py 按 infra/images/services.json 生成，不要手改）。
# 二进制由构建机（ubuntu 24.04 x64）编译后放进构建上下文，布局：./server-link。
# 二进制动态链接 glibc，运行时基础镜像要不低于构建机的 glibc 版本。
FROM ubuntu:24.04

RUN apt-get update \
    && apt-get install -y --no-install-recommends ca-certificates curl \
    && rm -rf /var/lib/apt/lists/* \
    && useradd --uid 10002 --no-create-home --shell /usr/sbin/nologin link

WORKDIR /app
COPY server-link /app/server-link

ENV LINK_HOST=0.0.0.0 \
    LINK_PORT=55030

USER link
EXPOSE 55030
HEALTHCHECK --interval=15s --timeout=3s --start-period=20s --retries=5 \
    CMD curl -fsS "http://127.0.0.1:${LINK_PORT}/api/link/health" || exit 1

ENTRYPOINT ["/app/server-link"]
