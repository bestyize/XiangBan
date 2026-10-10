package xyz.xiangban.xiaoban

interface Platform {
    val name: String
}

expect fun getPlatform(): Platform