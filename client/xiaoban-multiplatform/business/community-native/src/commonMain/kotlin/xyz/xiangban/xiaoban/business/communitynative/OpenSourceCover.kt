package xyz.xiangban.xiaoban.business.communitynative

import androidx.compose.runtime.Composable
import androidx.compose.runtime.remember
import coil3.ImageLoader
import coil3.compose.LocalPlatformContext
import coil3.network.ktor3.KtorNetworkFetcherFactory
import coil3.svg.SvgDecoder

@Composable
internal fun rememberSoftwareImageLoader(): ImageLoader {
    val context = LocalPlatformContext.current
    return remember(context) {
        ImageLoader.Builder(context).components {
            add(KtorNetworkFetcherFactory())
            add(SvgDecoder.Factory())
        }.build()
    }
}
