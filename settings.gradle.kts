pluginManagement {
    repositories {
        gradlePluginPortal()
        mavenCentral()
    }
}

plugins {
    id("org.gradle.toolchains.foojay-resolver-convention") version "1.0.0"
}

dependencyResolutionManagement {
    repositories {
        mavenCentral()
    }
}

rootProject.name = "xiaban"

include(":api-contracts", ":backend")
project(":api-contracts").projectDir = file("services/xiaban-api/api-contracts")
project(":backend").projectDir = file("services/xiaban-api/backend")
