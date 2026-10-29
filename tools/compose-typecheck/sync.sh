#!/usr/bin/env bash
# 把手机端的 Kotlin 源码同步进本目录的临时工程，用仿制的 Compose 接口做类型检查。
# 用法：tools/compose-typecheck/sync.sh [最多显示多少行错误，默认 80]
# 没有输出 = 通过。需要 gradle 和 JDK 17+；依赖只有 Maven Central（沙箱里 Google Maven 不通也能跑）。
set -euo pipefail
here="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
repo="$(cd "$here/../.." && pwd)"
r="$repo/client/xiaoban-multiplatform"
t="$here/build-src/main/kotlin"
mkdir -p "$t/core" "$t/link" "$t/native" "$t/stubs"
for d in core link native stubs; do find "$t/$d" -name '*.kt' -delete; done
cp -r "$r/common/community-core/src/commonMain/kotlin/." "$t/core/"
cp -r "$r/common/link-core/src/commonMain/kotlin/." "$t/link/"
cp -r "$r/business/community-native/src/commonMain/kotlin/." "$t/native/"
cp "$here"/stubs/*.kt "$t/stubs/"
cd "$here"
gradle compileKotlin --no-daemon -q --offline 2>&1 | grep -v "JAVA_TOOL" | grep "^e:\|FAIL\|wrong" | sed "s#file://$here/build-src/main/kotlin/##" | head -"${1:-80}" || true
