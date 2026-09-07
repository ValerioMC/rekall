#!/usr/bin/env bash
# Builds the GraalVM native image for rekall-app. Cross-platform: tested on macOS (Apple
# Silicon) and Windows (via Git Bash, e.g. GitHub Actions' windows-latest runner) - Windows
# needs `zip` on PATH (not bundled with Git Bash; `choco install zip -y` in CI) and a GraalVM
# JDK with a working C toolchain (MSVC on Windows, set up via a Developer Command Prompt or
# ilammy/msvc-dev-cmd in CI).
#
# `mvn -Pnative package` alone only prepares the build (AOT processing, reachability
# metadata) - see the comment on the native profile in rekall-app/pom.xml for why the actual
# `native-image` invocation lives here instead: every in-pom way tried to feed native-image a
# patched copy of hibernate-core.jar (with the ByteBuddy BytecodeProvider service registration
# stripped, required because native image can never allow ByteBuddy's runtime class generation)
# turned out unreliable across several attempts, and patching the shared ~/.m2 cache copy
# instead breaks hibernate-maven-plugin's own build-time enhancement, which needs ByteBuddy
# working normally on the JVM.
set -euo pipefail

cd "$(dirname "$0")/.."

case "$(uname -s)" in
    MINGW*|MSYS*|CYGWIN*) SEP=';' ;;
    *) SEP=':' ;;
esac

echo "==> mvn -Pnative install (AOT + reachability metadata)"
# install, not package: the classpath resolution below runs as its own, separate mvn
# invocation (not part of this reactor session), so it resolves rekall-domain/rekall-api/
# rekall-mcp from ~/.m2 - without installing here first, it would silently pick up
# whatever was last installed there (in one case, a stale jar from a full day earlier),
# not the jar this run just built.
mvn -Pnative -pl rekall-app -am clean install -DskipTests

CP_FILE="rekall-app/target/native-classpath.txt"
echo "==> Resolving full runtime classpath"
mvn -q -pl rekall-app dependency:build-classpath -DincludeScope=runtime -Dmdep.outputFile="$(pwd)/$CP_FILE"

HIBERNATE_VERSION=$(mvn -q -pl rekall-app help:evaluate -Dexpression=hibernate.version -DforceStdout)
LOCAL_REPO=$(mvn -q -pl rekall-app help:evaluate -Dexpression=settings.localRepository -DforceStdout)
ORIGINAL_JAR="${LOCAL_REPO}/org/hibernate/orm/hibernate-core/${HIBERNATE_VERSION}/hibernate-core-${HIBERNATE_VERSION}.jar"
if [ ! -f "$ORIGINAL_JAR" ]; then
    echo "Expected hibernate-core at $ORIGINAL_JAR, not found" >&2
    exit 1
fi

NATIVE_LIBS="rekall-app/target/native-libs"
mkdir -p "$NATIVE_LIBS"
PATCHED_JAR="$NATIVE_LIBS/hibernate-core-${HIBERNATE_VERSION}.jar"
cp "$ORIGINAL_JAR" "$PATCHED_JAR"

echo "==> Stripping ByteBuddy's BytecodeProvider registration from a project-local copy"
echo "    (never the shared ~/.m2 cache: hibernate-maven-plugin's own enhance goal needs it)"
zip -q -d "$PATCHED_JAR" "META-INF/services/org.hibernate.bytecode.spi.BytecodeProvider"

# Rebuild the classpath as: this module's classes, the patched jar, then everything
# build-classpath resolved except the original (unpatched) hibernate-core entry - split/joined
# on the OS path separator rather than via sed/grep, since a Windows path's drive-letter colon
# would otherwise collide with the classpath separator itself.
CLASSPATH="rekall-app/target/classes${SEP}${PATCHED_JAR}"
# <<< (not < "$CP_FILE" directly) because build-classpath's output has no trailing newline,
# which makes `read` return non-zero at EOF even after reading the line successfully - and
# that would otherwise kill the script here under `set -e`.
IFS="$SEP" read -ra ENTRIES <<< "$(cat "$CP_FILE")"
for entry in "${ENTRIES[@]}"; do
    if [ "$entry" != "$ORIGINAL_JAR" ]; then
        CLASSPATH="${CLASSPATH}${SEP}${entry}"
    fi
done

# GraalVM ships native-image as a .cmd shim on Windows, which Git Bash does not resolve from a
# bare `native-image` even with the distribution on PATH. Prefer whatever is on PATH, and fall
# back to the one inside the JDK, .cmd included.
GRAAL_HOME="${GRAALVM_HOME:-${JAVA_HOME:-}}"
if command -v native-image >/dev/null 2>&1; then
    NATIVE_IMAGE="native-image"
elif [ -n "$GRAAL_HOME" ] && [ -x "$GRAAL_HOME/bin/native-image" ]; then
    NATIVE_IMAGE="$GRAAL_HOME/bin/native-image"
elif [ -n "$GRAAL_HOME" ] && [ -f "$GRAAL_HOME/bin/native-image.cmd" ]; then
    NATIVE_IMAGE="$GRAAL_HOME/bin/native-image.cmd"
else
    echo "native-image not found. Set GRAALVM_HOME or JAVA_HOME to a GraalVM 25." >&2
    exit 1
fi

echo "==> native-image ($NATIVE_IMAGE)"
"$NATIVE_IMAGE" \
    -cp "$CLASSPATH" \
    --no-fallback \
    -o rekall-app/target/rekall-app \
    dev.rekall.RekallApplication

echo "==> Built rekall-app/target/rekall-app"
