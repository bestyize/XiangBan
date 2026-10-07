# Build context: repository root; backend and contracts are Gradle subprojects.
FROM eclipse-temurin:21-jdk AS build
WORKDIR /src
COPY gradlew settings.gradle.kts build.gradle.kts gradle.properties ./
COPY gradle gradle
COPY services/xiaban-api/api-contracts/build.gradle.kts services/xiaban-api/api-contracts/
COPY services/xiaban-api/backend/build.gradle.kts services/xiaban-api/backend/
# Resolve Gradle and plugins in their own layer so code changes do not re-download them.
RUN chmod +x gradlew && ./gradlew --no-daemon -q help
COPY services/xiaban-api/api-contracts services/xiaban-api/api-contracts
COPY services/xiaban-api/backend services/xiaban-api/backend
RUN ./gradlew --no-daemon -q :backend:installDist -x test

FROM eclipse-temurin:21-jre
RUN useradd --system --uid 10001 --no-create-home xiaban
WORKDIR /app
COPY --from=build /src/services/xiaban-api/backend/build/install/backend /app
USER xiaban
# Sized for the 1 vCPU / 2 GB app host of phase one (PRD 9.2).
ENV JAVA_OPTS="-XX:MaxRAMPercentage=60 -XX:+ExitOnOutOfMemoryError" \
    XIABAN_ENV=prod \
    PORT=8080
EXPOSE 8080
ENTRYPOINT ["/app/bin/backend"]
