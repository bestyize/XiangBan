# Build context: repository root (the server depends on the shared module).
FROM eclipse-temurin:21-jdk AS build
WORKDIR /src
COPY gradlew settings.gradle.kts build.gradle.kts gradle.properties ./
COPY gradle gradle
COPY shared/build.gradle.kts shared/
COPY server/build.gradle.kts server/
# Resolve Gradle and plugins in their own layer so code changes do not re-download them.
RUN ./gradlew --no-daemon -q help
COPY shared shared
COPY server server
RUN ./gradlew --no-daemon -q :server:installDist -x test

FROM eclipse-temurin:21-jre
RUN useradd --system --uid 10001 --no-create-home xiaban
WORKDIR /app
COPY --from=build /src/server/build/install/server /app
USER xiaban
# Sized for the 1 vCPU / 2 GB app host of phase one (PRD 9.2).
ENV JAVA_OPTS="-XX:MaxRAMPercentage=60 -XX:+ExitOnOutOfMemoryError" \
    XIABAN_ENV=prod \
    PORT=8080
EXPOSE 8080
ENTRYPOINT ["/app/bin/server"]
