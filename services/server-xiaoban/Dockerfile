# 壁纸采集服务的运行时镜像（由 infra/images/gen-dockerfiles.py 按 infra/images/services.json 生成，不要手改）。
# 二进制由构建机（ubuntu 24.04 x64）编译后放进构建上下文，布局：./wallpaper_collect。
# 二进制动态链接 glibc，运行时基础镜像要不低于构建机的 glibc 版本。
FROM ubuntu:24.04

RUN apt-get update \
    && apt-get install -y --no-install-recommends ca-certificates curl \
    && rm -rf /var/lib/apt/lists/* \
    && useradd --uid 10007 --no-create-home --shell /usr/sbin/nologin wallpaper

WORKDIR /app
COPY wallpaper_collect /app/wallpaper_collect

ENV SERVER_HOST=0.0.0.0 \
    SERVER_PORT=8080

USER wallpaper
EXPOSE 8080
HEALTHCHECK --interval=15s --timeout=3s --start-period=20s --retries=5 \
    CMD curl -fsS "http://127.0.0.1:${SERVER_PORT}/health" || exit 1

ENTRYPOINT ["/app/wallpaper_collect"]
