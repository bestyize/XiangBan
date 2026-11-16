import org.jetbrains.kotlin.gradle.dsl.JvmTarget
import groovy.json.JsonSlurper
import java.security.MessageDigest
import java.nio.file.Files
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
    implementation(libs.androidx.webkit)
    implementation("com.google.zxing:core:3.5.4")
    implementation(libs.onnxruntime.android)
    implementation("org.apache.commons:commons-compress:1.27.1")
    implementation("org.tukaani:xz:1.10")

    implementation(libs.compose.uiToolingPreview)
    implementation(libs.compose.foundation)
    implementation(libs.compose.material3)
    implementation(libs.compose.animation)
    debugImplementation(libs.compose.uiTooling)
    testDebugImplementation(libs.junit)
}

android {
    buildFeatures { buildConfig = true }
    namespace = "xyz.xiangban.xiaoban"
    compileSdk = libs.versions.android.compileSdk.get().toInt()

    defaultConfig {
        applicationId = "xyz.xiangban.xiaoban"
        minSdk = libs.versions.android.minSdk.get().toInt()
        targetSdk = libs.versions.android.targetSdk.get().toInt()
        versionName = "3.1.8"
        val (major, minor, patch) = requireNotNull(versionName).split('.').map(String::toInt)
        require(major in 0..2099 && minor in 0..999 && patch in 0..999)
        versionCode = (major * 1_000_000 + minor * 1_000 + patch).also { require(it > 0) }
        // 手机只支持 64 位 ARM（arm64-v8a）：原生库只打这一种，其他 CPU 的包不装。
        ndk { abiFilters += "arm64-v8a" }
    }
    sourceSets.getByName("main") {
        assets.srcDir(layout.buildDirectory.dir("tool-environment/assets").get().asFile)
        jniLibs.srcDir(layout.buildDirectory.dir("tool-environment/jni").get().asFile)
    }
    packaging {
        // PRoot is launched as an executable from Android's extracted, read-only native library directory.
        jniLibs.useLegacyPackaging = true
        jniLibs.keepDebugSymbols += setOf("**/libproot*.so", "**/libtalloc.so", "**/libandroid-shmem.so")
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
        debug {
            // Keep locally signed builds beside an installed release or another machine's debug build.
            applicationIdSuffix = ".debug"
        }
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
    inputs.file(script)
    inputs.file(rootProject.layout.projectDirectory.file("gradle/libs.versions.toml"))
    outputs.dir(voiceRuntimeDirectory)
    val python = System.getenv("PYTHON")?.takeIf { it.isNotBlank() }
        ?: if (System.getProperty("os.name").startsWith("Windows")) "python" else "python3"
    commandLine(python, script.asFile.absolutePath, "--output", voiceRuntimeDirectory.get().asFile.absolutePath)
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

// Copy the reviewed protocol 2 packages and add the Android host adapter before gameplay scripts.
abstract class SyncMobileGameAssets : DefaultTask() {
    @get:InputDirectory abstract val sourceDirectory: org.gradle.api.file.DirectoryProperty
    @get:InputDirectory abstract val runtimeDirectory: org.gradle.api.file.DirectoryProperty
    @get:InputDirectory abstract val previewDirectory: org.gradle.api.file.DirectoryProperty
    @get:InputFile abstract val distributionFile: org.gradle.api.file.RegularFileProperty
    @get:OutputDirectory abstract val outputDirectory: org.gradle.api.file.DirectoryProperty
    @get:javax.inject.Inject abstract val files: org.gradle.api.file.FileSystemOperations
    @TaskAction fun copyGames() {
        val source = sourceDirectory.get().asFile
        val runtime = runtimeDirectory.get().asFile
        val output = outputDirectory.get().asFile
        val categories = (JsonSlurper().parse(runtime.resolve("catalog.json")) as Map<*, *>).map { (id, category) ->
            (id as String) to (category as String)
        }.toMap()
        val reviewedIds = source.listFiles().orEmpty().filter { it.resolve("manifest.json").isFile }.map { it.name }.toSet()
        check(categories.keys == reviewedIds) { "Android 内置作品的协议 2 分类与资源目录不一致" }
        val distribution = JsonSlurper().parse(distributionFile.get().asFile) as Map<*, *>
        val entries = (distribution["games"] as List<*>).map { it as Map<*, *> }.associateBy { it["id"] as String }
        check(distribution["version"] == 1 && entries.keys == reviewedIds) { "内置游戏分发清单与资源目录不一致" }
        val allowed = setOf("html", "htm", "js", "mjs", "css", "json", "svg", "png", "jpg", "jpeg", "webp", "gif", "mp3", "ogg", "wav", "woff2", "md", "txt")
        for (id in categories.keys) {
            val input = source.resolve(id)
            val digest = MessageDigest.getInstance("SHA-256")
            val sourceFiles = input.walkTopDown().filter { it.isFile && !it.name.contains(".test.") }.sortedBy { it.relativeTo(input).invariantSeparatorsPath }
            for (file in sourceFiles) {
                check(!Files.isSymbolicLink(file.toPath()) && file.extension.lowercase() in allowed) { "内置游戏包含不支持的文件：$file" }
                digest.update(file.relativeTo(input).invariantSeparatorsPath.toByteArray(Charsets.UTF_8))
                digest.update(0.toByte())
                digest.update(MessageDigest.getInstance("SHA-256").digest(file.readBytes()))
            }
            val checksum = digest.digest().joinToString("") { "%02x".format(it) }
            check(checksum == entries.getValue(id)["sourceSha256"]) { "内置游戏 $id 已更改，请重新生成并部署资源包" }
        }
        files.sync {
            for (id in categories.keys) {
                val target = "game-packs-v2/$id"
                val manifest = JsonSlurper().parse(source.resolve("$id/manifest.json")) as Map<*, *>
                from(source.resolve(id)) {
                    include("manifest.json", manifest["icon"] as String, manifest["cover"] as String)
                    if (id == "pocket-sprites") exclude(manifest["icon"] as String, manifest["cover"] as String)
                    into(target)
                }
                if (id == "pocket-sprites") from(previewDirectory) { into("$target/assets") }
            }
            from(listOf(runtime.resolve("builtin-runtime.js"), runtime.resolve("builtin-runtime.css"))) { into("builtin-runtime-v2") }
            from(distributionFile) { into("builtin-games") }
            into(output)
        }
        for ((id, category) in categories) {
            val input = source.resolve(id)
            val manifest = JsonSlurper().parse(input.resolve("manifest.json")) as Map<*, *>
            check(manifest["schemaVersion"] == 3 && manifest["runtimeProtocol"] == 2 && manifest["id"] == id && manifest["entry"] == "index.html" && manifest["category"] == category) {
                "$id 的协议 2 清单无效"
            }

            val entry = input.resolve("index.html").readText()
            check("</head>" in entry && "<script" in entry) { "$id 的入口缺少脚本或 head 结束标签" }
        }
    }
}
val mobileGameAssets = tasks.register<SyncMobileGameAssets>("syncMobileGameAssets") {
    sourceDirectory.set(rootProject.layout.projectDirectory.dir("../xiaoban-desktop/public/game-packs"))
    runtimeDirectory.set(layout.projectDirectory.dir("src/main/builtin-runtime-v2"))
    previewDirectory.set(rootProject.layout.projectDirectory.dir("../../assets/game-packs/previews/pocket-sprites"))
    distributionFile.set(rootProject.layout.projectDirectory.file("../../assets/game-packs/distribution.json"))
    outputDirectory.set(layout.buildDirectory.dir("generated/mobile-games"))
}
androidComponents.onVariants { variant ->
    variant.sources.assets?.addGeneratedSourceDirectory(mobileGameAssets) { it.outputDirectory }
    variant.sources.assets?.addGeneratedSourceDirectory(mobileVoiceAssets) { it.outputDirectory }
    variant.instrumentation.transformClassesWith(SpeechOrtLoaderTransform::class.java, InstrumentationScope.ALL) {}
}
