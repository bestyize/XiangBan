package xyz.xiangban.xiaoban.common.community

import io.ktor.client.HttpClient
import io.ktor.client.plugins.HttpTimeout

/** 应用里用的 HTTP 客户端：引擎由各平台依赖提供（Android 用 OkHttp，iOS 用 Darwin），这里只统一超时。 */
fun defaultHttpClient(): HttpClient = HttpClient {
    install(HttpTimeout) {
        connectTimeoutMillis = 10_000
        requestTimeoutMillis = 30_000
    }
    expectSuccess = false
}
