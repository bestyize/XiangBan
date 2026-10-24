# 只含运行时的镜像：二进制与前端静态资源由 CI 在 ubuntu x64 上构建，再放进构建上下文。
# 上下文目录结构：./server-xiaoban-market（release 二进制）与 ./web（前端 dist）。
# 构建机必须是 ubuntu 24.04：二进制动态链接 glibc，运行时基础镜像要不低于构建机的 glibc 版本。
FROM ubuntu:24.04

RUN apt-get update \
    && apt-get install -y --no-install-recommends ca-certificates curl \
    && rm -rf /var/lib/apt/lists/* \
    && useradd --uid 10001 --no-create-home --shell /usr/sbin/nologin market \
    && mkdir /data && chown market /data

WORKDIR /app
COPY server-xiaoban-market /app/server-xiaoban-market
COPY web /app/web

# 安装包按内容哈希存放在 /data，部署时挂命名卷；根文件系统可以只读。
ENV MARKET_HOST=0.0.0.0 \
    MARKET_PORT=55010 \
    MARKET_WEB_DIR=/app/web \
    MARKET_DATA_DIR=/data

USER market
EXPOSE 55010
VOLUME /data
HEALTHCHECK --interval=15s --timeout=3s --start-period=20s --retries=5 \
    CMD curl -fsS "http://127.0.0.1:${MARKET_PORT}/api/health" || exit 1

ENTRYPOINT ["/app/server-xiaoban-market"]
