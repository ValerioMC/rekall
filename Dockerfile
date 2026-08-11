# Three stages: the UI is built by node, the jar by maven, and neither toolchain ships in the
# runtime image. The UI output lands in the Spring Boot module's static resources, so a single
# jar serves the API, the MCP endpoint and the frontend.

FROM node:22-alpine AS ui
WORKDIR /ui
COPY rekall-ui/package.json rekall-ui/pnpm-lock.yaml ./
RUN corepack enable && pnpm install --frozen-lockfile
COPY rekall-ui/ ./
# Overridden so the build stays inside this stage rather than writing to the Java module.
RUN pnpm exec vite build --outDir /ui/dist --emptyOutDir

FROM maven:3.9-eclipse-temurin-25 AS backend
WORKDIR /build
# Poms first: dependency resolution is cached unless a pom actually changes.
COPY pom.xml ./
COPY rekall-meta/pom.xml rekall-meta/
COPY rekall-engine/pom.xml rekall-engine/
COPY rekall-content/pom.xml rekall-content/
COPY rekall-api/pom.xml rekall-api/
COPY rekall-mcp/pom.xml rekall-mcp/
COPY rekall-app/pom.xml rekall-app/
RUN mvn -B -ntp dependency:go-offline -DskipTests || true

COPY rekall-meta/src rekall-meta/src
COPY rekall-engine/src rekall-engine/src
COPY rekall-content/src rekall-content/src
COPY rekall-api/src rekall-api/src
COPY rekall-mcp/src rekall-mcp/src
COPY rekall-app/src rekall-app/src
COPY --from=ui /ui/dist rekall-app/src/main/resources/static

# -DskipTests rather than -Dmaven.test.skip: the latter would skip test compilation too, and
# rekall-meta publishes a test-jar the other modules depend on.
RUN mvn -B -ntp -DskipTests package

FROM eclipse-temurin:25-jre
WORKDIR /app

# Non-root: nothing here needs to write outside the JVM's own temp space.
RUN groupadd --system rekall && useradd --system --gid rekall --home /app rekall
COPY --from=backend /build/rekall-app/target/rekall-app-*.jar app.jar
USER rekall

EXPOSE 8080
ENV JAVA_OPTS="-XX:MaxRAMPercentage=70"
ENTRYPOINT ["sh", "-c", "exec java $JAVA_OPTS -jar app.jar"]
