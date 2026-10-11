import java.io.File
import java.util.Properties

plugins {
    alias(libs.plugins.android.application)
    alias(libs.plugins.kotlin.compose)
}

val localProperties = Properties().apply {
    val localFile = rootProject.file("local.properties")
    if (localFile.exists()) {
        localFile.inputStream().use { load(it) }
    }
    val rootLocalFile = rootProject.rootDir.resolve("../../local.properties")
    if (rootLocalFile.exists()) {
        rootLocalFile.inputStream().use { load(it) }
    }
}

fun getProp(vararg keys: String): String? {
    for (key in keys) {
        val v = localProperties.getProperty(key) ?: System.getenv(key)
        if (!v.isNullOrBlank()) return v
    }
    return null
}

val userHomeKeystore = File(System.getProperty("user.home"), "xiangbanmobile.jks")
val keystoreFilePath = getProp("keystore.file", "KEYSTORE_FILE", "storeFile")
val targetKeystoreFile = if (!keystoreFilePath.isNullOrBlank()) File(keystoreFilePath) else userHomeKeystore
val targetStorePassword = getProp("keystore.password", "KEYSTORE_PASSWORD", "keystorePassword", "storePassword")
val targetKeyAlias = getProp("keystore.alias", "key.alias", "KEY_ALIAS", "keyAlias") ?: "release-key"
val targetKeyPassword = getProp("key.password", "KEY_PASSWORD", "keyPassword") ?: targetStorePassword

val hasCustomSigning = targetKeystoreFile.exists() && !targetStorePassword.isNullOrBlank()

android {
    namespace = "xyz.xiaban.wallpaper"
    compileSdk {
        version = release(37)
    }

    defaultConfig {
        applicationId = "xyz.xiaban.wallpaper"
        // Saving into the gallery through MediaStore needs no storage permission from Android 10.
        minSdk = 29
        targetSdk = 37
        versionCode = 4
        versionName = "1.0.0"
        testInstrumentationRunner = "androidx.test.runner.AndroidJUnitRunner"
    }

    signingConfigs {
        create("release") {
            if (hasCustomSigning) {
                storeFile = targetKeystoreFile
                storePassword = targetStorePassword
                keyAlias = targetKeyAlias
                keyPassword = targetKeyPassword
            }
        }
        if (hasCustomSigning) {
            getByName("debug") {
                storeFile = targetKeystoreFile
                storePassword = targetStorePassword
                keyAlias = targetKeyAlias
                keyPassword = targetKeyPassword
            }
        }
    }

    buildTypes {
        debug {
            if (hasCustomSigning) {
                signingConfig = signingConfigs.getByName("debug")
            }
        }
        release {
            if (hasCustomSigning) {
                signingConfig = signingConfigs.getByName("release")
            }
            // Release builds ship for 64-bit ARM phones only; debug stays universal for x86_64 emulators.
            ndk {
                abiFilters += "arm64-v8a"
            }
            optimization {
                enable = true
            }
        }
    }
    compileOptions {
        sourceCompatibility = JavaVersion.VERSION_17
        targetCompatibility = JavaVersion.VERSION_17
    }
    buildFeatures {
        compose = true
    }
}

dependencies {
    implementation(platform(libs.androidx.compose.bom))
    implementation(libs.androidx.compose.ui)
    implementation(libs.androidx.compose.foundation)
    implementation(libs.androidx.compose.material3)
    implementation(libs.androidx.compose.icons)
    implementation(libs.androidx.activity.compose)
    implementation(libs.androidx.lifecycle.viewmodel.compose)
    implementation(libs.androidx.lifecycle.runtime.compose)
    implementation(libs.coil.compose)
    implementation(libs.coil.network.okhttp)
    implementation(libs.androidx.media3.exoplayer)
    implementation(libs.androidx.media3.ui)
    debugImplementation(libs.androidx.compose.ui.tooling)

    testImplementation(libs.junit)
    testImplementation(libs.org.json)

    androidTestImplementation(platform(libs.androidx.compose.bom))
    androidTestImplementation(libs.androidx.compose.ui.test.junit4)
    androidTestImplementation(libs.androidx.test.ext.junit)
    androidTestImplementation(libs.androidx.test.runner)
    debugImplementation(libs.androidx.compose.ui.test.manifest)
    // The instrumented-test libraries need a newer concurrent-futures than the app resolves on its own.
    debugImplementation(libs.androidx.concurrent.futures)
}
