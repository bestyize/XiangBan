import org.jetbrains.kotlin.gradle.dsl.JvmTarget
import com.android.build.api.instrumentation.AsmClassVisitorFactory
import com.android.build.api.instrumentation.ClassContext
import com.android.build.api.instrumentation.ClassData
import com.android.build.api.instrumentation.InstrumentationParameters
import com.android.build.api.instrumentation.InstrumentationScope
import org.objectweb.asm.ClassVisitor
import org.objectweb.asm.MethodVisitor
import org.objectweb.asm.Opcodes

plugins {
    alias(libs.plugins.androidApplication)
    alias(libs.plugins.composeCompiler)
}

kotlin {
    compilerOptions {
        jvmTarget = JvmTarget.JVM_11
    }
}
// 发布签名：设置了下面四个环境变量（CI 里来自仓库密钥）就用发布密钥签名，否则 release 包是未签名的。
//   KEYSTORE_FILE、KEYSTORE_PASSWORD、KEY_ALIAS、KEY_PASSWORD
val releaseKeystore: String? = System.getenv("KEYSTORE_FILE")?.takeIf { it.isNotBlank() && File(it).exists() }
val releaseStorePassword: String? = System.getenv("KEYSTORE_PASSWORD")?.takeIf { it.isNotBlank() }
val hasReleaseSigning = releaseKeystore != null && releaseStorePassword != null

dependencies {
    implementation(project(":shell"))

    implementation(libs.androidx.activity.compose)
    implementation(libs.androidx.core.ktx)
    implementation("com.google.zxing:core:3.5.4")
    implementation(libs.onnxruntime.android)

    implementation(libs.compose.uiToolingPreview)
    debugImplementation(libs.compose.uiTooling)
}

android {
    namespace = "xyz.xiangban.xiaoban"
    compileSdk = libs.versions.android.compileSdk.get().toInt()

    defaultConfig {
        applicationId = "xyz.xiangban.xiaoban"
        minSdk = libs.versions.android.minSdk.get().toInt()
        targetSdk = libs.versions.android.targetSdk.get().toInt()
        versionCode = 1
        versionName = "1.0.0"
        // 手机只支持 64 位 ARM（arm64-v8a）：原生库只打这一种，其他 CPU 的包不装。
        ndk { abiFilters += "arm64-v8a" }
    }
    packaging {
        // Java/JNI 接口保留在 APK；四个语音原生库从带校验的组件清单按需下载。
        jniLibs.excludes += setOf("**/libonnxruntime.so", "**/libonnxruntime4j_jni.so", "**/libort_sherpa.so", "**/libsherpa-onnx-jni.so")
        resources {
            excludes += "/META-INF/{AL2.0,LGPL2.1}"
        }
    }
    signingConfigs {
        if (hasReleaseSigning) {
            create("release") {
                storeFile = File(releaseKeystore!!)
                storePassword = releaseStorePassword
                keyAlias = System.getenv("KEY_ALIAS")?.takeIf { it.isNotBlank() } ?: "release-key"
                keyPassword = System.getenv("KEY_PASSWORD")?.takeIf { it.isNotBlank() } ?: releaseStorePassword
            }
        }
    }
    buildTypes {
        release {
            if (hasReleaseSigning) signingConfig = signingConfigs.getByName("release")
            isMinifyEnabled = false
            proguardFiles(
                getDefaultProguardFile("proguard-android-optimize.txt"),
                "proguard-rules.pro"
            )
        }
    }
    compileOptions {
        sourceCompatibility = JavaVersion.VERSION_11
        targetCompatibility = JavaVersion.VERSION_11
    }
    buildFeatures {
        compose = true
    }
}

// ORT 的 Android 分支忽略 onnxruntime.native.path，必须重定向其硬编码的 loadLibrary。
// 仅修改这个依赖类的加载调用，其余 JNI 类和推理代码保持上游版本。
abstract class SpeechOrtLoaderTransform : AsmClassVisitorFactory<InstrumentationParameters.None> {
    override fun isInstrumentable(classData: ClassData) = classData.className == "ai.onnxruntime.OnnxRuntime"
    override fun createClassVisitor(classContext: ClassContext, nextClassVisitor: ClassVisitor): ClassVisitor =
        object : ClassVisitor(Opcodes.ASM9, nextClassVisitor) {
            override fun visitMethod(access: Int, name: String?, descriptor: String?, signature: String?, exceptions: Array<out String>?): MethodVisitor =
                object : MethodVisitor(Opcodes.ASM9, super.visitMethod(access, name, descriptor, signature, exceptions)) {
                    override fun visitMethodInsn(opcode: Int, owner: String, name: String, descriptor: String, isInterface: Boolean) {
                        if (opcode == Opcodes.INVOKESTATIC && owner == "java/lang/System" && name == "loadLibrary" && descriptor == "(Ljava/lang/String;)V") {
                            super.visitMethodInsn(opcode, "xyz/xiangban/xiaoban/SpeechNativeLoader", "load", descriptor, false)
                        } else super.visitMethodInsn(opcode, owner, name, descriptor, isInterface)
                    }
                }
        }
}

val voiceRuntimeDirectory = layout.buildDirectory.dir("voice-runtime")
val prepareVoiceRuntime = tasks.register<Exec>("prepareVoiceRuntime") {
    val script = layout.projectDirectory.file("scripts/prepare-voice-runtime.py")
    val releaseTag = providers.environmentVariable("XIAOBAN_VOICE_RELEASE_TAG").getOrElse("xiaoban-wallpaper-v3.0.0")
    inputs.file(script)
    inputs.file(rootProject.layout.projectDirectory.file("gradle/libs.versions.toml"))
    inputs.property("releaseTag", releaseTag)
    outputs.dir(voiceRuntimeDirectory)
    commandLine("python3", script.asFile.absolutePath, "--output", voiceRuntimeDirectory.get().asFile.absolutePath, "--release-tag", releaseTag)
}

abstract class SyncMobileVoiceAssets : DefaultTask() {
    @get:InputDirectory abstract val sourceDirectory: org.gradle.api.file.DirectoryProperty
    @get:OutputDirectory abstract val outputDirectory: org.gradle.api.file.DirectoryProperty
    @get:javax.inject.Inject abstract val files: org.gradle.api.file.FileSystemOperations
    @TaskAction fun copyManifest() {
        files.sync { from(sourceDirectory); into(outputDirectory) }
    }
}
val mobileVoiceAssets = tasks.register<SyncMobileVoiceAssets>("syncMobileVoiceAssets") {
    dependsOn(prepareVoiceRuntime)
    sourceDirectory.set(voiceRuntimeDirectory.map { it.dir("assets") })
    outputDirectory.set(layout.buildDirectory.dir("generated/mobile-voice"))
}

// Package the same reviewed offline games as desktop; never copy the rest of its public directory.
abstract class SyncMobileGameAssets : DefaultTask() {
    @get:InputDirectory abstract val sourceDirectory: org.gradle.api.file.DirectoryProperty
    @get:OutputDirectory abstract val outputDirectory: org.gradle.api.file.DirectoryProperty
    @get:javax.inject.Inject abstract val files: org.gradle.api.file.FileSystemOperations
    @TaskAction fun copyGames() {
        files.sync {
            from(sourceDirectory) {
                include("**/*.html", "**/*.js", "**/*.css", "**/*.json", "**/*.svg")
                exclude("**/*.test.*")
            }
            into(outputDirectory.dir("game-packs"))
        }
    }
}
val mobileGameAssets = tasks.register<SyncMobileGameAssets>("syncMobileGameAssets") {
    sourceDirectory.set(rootProject.layout.projectDirectory.dir("../xiaoban-desktop/public/game-packs"))
    outputDirectory.set(layout.buildDirectory.dir("generated/mobile-games"))
}
androidComponents.onVariants { variant ->
    variant.sources.assets?.addGeneratedSourceDirectory(mobileGameAssets) { it.outputDirectory }
    variant.sources.assets?.addGeneratedSourceDirectory(mobileVoiceAssets) { it.outputDirectory }
    variant.instrumentation.transformClassesWith(SpeechOrtLoaderTransform::class.java, InstrumentationScope.ALL) {}
}
