// 类型检查用的临时工程：源码由 sync.sh 复制到 build-src/，Compose 接口用 stubs/ 里的仿制版。
plugins {
    kotlin("jvm") version "2.2.20"
    kotlin("plugin.serialization") version "2.2.20"
}
dependencies {
    implementation("org.jetbrains.kotlinx:kotlinx-serialization-json:1.9.0")
    implementation("org.jetbrains.kotlinx:kotlinx-coroutines-core:1.10.2")
    implementation("io.ktor:ktor-client-core:3.3.0")
    implementation("io.ktor:ktor-client-websockets:3.3.0")
}
kotlin { compilerOptions { freeCompilerArgs.addAll("-opt-in=kotlin.time.ExperimentalTime") } }
sourceSets { main { kotlin.setSrcDirs(listOf("build-src/main/kotlin")) } }
