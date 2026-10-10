# 只含运行时的镜像：二进制与管理后台前端由 CI 在 ubuntu x64 上构建，再放进构建上下文。
# 上下文目录结构：./server-account（release 二进制）与 ./web（管理后台 dist）。
# 构建机必须是 ubuntu 24.04：二进制动态链接 glibc，运行时基础镜像要不低于构建机的 glibc 版本。
FROM ubuntu:24.04

RUN apt-get update \
    && apt-get install -y --no-install-recommends ca-certificates curl \
    && rm -rf /var/lib/apt/lists/* \
    && useradd --uid 10001 --no-create-home --shell /usr/sbin/nologin account

WORKDIR /app
COPY server-account /app/server-account
COPY web /app/web

ENV ACCOUNT_HOST=0.0.0.0 \
    ACCOUNT_PORT=55009 \
    ACCOUNT_WEB_DIR=/app/web

USER account
EXPOSE 55009
HEALTHCHECK --interval=15s --timeout=3s --start-period=20s --retries=5 \
    CMD curl -fsS "http://127.0.0.1:${ACCOUNT_PORT}/api/health" || exit 1

ENTRYPOINT ["/app/server-account"]
