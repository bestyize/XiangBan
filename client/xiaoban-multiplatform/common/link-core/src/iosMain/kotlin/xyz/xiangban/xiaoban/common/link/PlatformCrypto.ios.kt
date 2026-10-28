package xyz.xiangban.xiaoban.common.link

/** iOS 暂缓（互联计划已定仅 Android）。保留这个桩只是为了让共享框架能编译。 */
actual fun platformCrypto(): PlatformCrypto = throw UnsupportedOperationException("互联功能暂不支持 iOS")
