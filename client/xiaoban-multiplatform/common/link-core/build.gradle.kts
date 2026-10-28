import org.jetbrains.kotlin.gradle.dsl.JvmTarget

plugins {
    alias(libs.plugins.kotlinMultiplatform)
    alias(libs.plugins.androidMultiplatformLibrary)
    alias(libs.plugins.kotlinSerialization)
}

// 手机与桌面互联的协议实现（对应 docs/agent/设备互联协议.md）：配对、密钥派生、端到端加密信封、重放防护。
// 不依赖 Compose，可在 JVM 上直接跑单元测试，必须通过 libs/link-protocol/vectors/v1.json：
//   ./gradlew :common:link-core:jvmTest
kotlin {
    jvm()
    listOf(
        iosArm64(),
        iosSimulatorArm64()
    ).forEach { iosTarget ->
        iosTarget.binaries.framework {
            baseName = "CommonLinkCore"
            isStatic = true
        }
    }

    android {
        namespace = "xyz.xiangban.xiaoban.common.link"
        compileSdk = libs.versions.android.compileSdk.get().toInt()
        minSdk = libs.versions.android.minSdk.get().toInt()

        compilerOptions {
            jvmTarget = JvmTarget.JVM_11
        }
    }

    sourceSets {
        commonMain.dependencies {
            api(project(":common:community-core"))
            api(libs.kotlinx.coroutines.core)
            api(libs.kotlinx.serialization.json)
            api(libs.ktor.client.core)
            api(libs.ktor.client.websockets)
        }
        // Android 与 JVM 都用 Java 密码学接口（javax.crypto / java.security），同一份实现编进两边。
        jvmMain { kotlin.srcDir("src/jvmCommon/kotlin") }
        androidMain {
            kotlin.srcDir("src/jvmCommon/kotlin")
            kotlin.srcDir("src/androidMain/kotlin")
        }
        commonTest.dependencies {
            implementation(libs.kotlin.test)
            implementation(libs.kotlinx.coroutines.test)
            implementation(libs.ktor.client.mock)
        }
        jvmTest.dependencies {
            implementation(libs.ktor.client.cio)
        }
    }
}
