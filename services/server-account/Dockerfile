# 统一账号中心的运行时镜像（由 infra/images/gen-dockerfiles.py 按 infra/images/services.json 生成，不要手改）。
# 二进制与前端静态资源由构建机（ubuntu 24.04 x64）编译后放进构建上下文，布局：./server-account、./web。
# 二进制动态链接 glibc，运行时基础镜像要不低于构建机的 glibc 版本。
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
