package xyz.xiangban.xiaoban.business.communitynative

import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.material3.MaterialTheme
import androidx.compose.runtime.Composable
import androidx.compose.runtime.collectAsState
import androidx.compose.runtime.getValue
import androidx.compose.runtime.remember
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.ImageBitmap
import androidx.compose.ui.layout.ContentScale
import coil3.compose.AsyncImage
import kotlinx.coroutines.flow.MutableStateFlow
import xyz.xiangban.xiaoban.common.community.ApiException
import xyz.xiangban.xiaoban.common.community.CapabilityPackKinds
import xyz.xiangban.xiaoban.common.community.CommunityShare
import xyz.xiangban.xiaoban.common.community.CommunitySharingApi
import xyz.xiangban.xiaoban.common.community.Load
import xyz.xiangban.xiaoban.common.community.communityShareTarget
import xyz.xiangban.xiaoban.common.share.DetailShareSheet
import xyz.xiangban.xiaoban.common.share.ShareActionResult
import xyz.xiangban.xiaoban.common.share.ShareCardOptions
import xyz.xiangban.xiaoban.common.share.ShareContent
import xyz.xiangban.xiaoban.common.share.ShareDeleteAction
import xyz.xiangban.xiaoban.common.share.ShareExtraAction
import xyz.xiangban.xiaoban.common.share.ShareLoadState
import xyz.xiangban.xiaoban.common.share.SharePlatformBridge
import xyz.xiangban.xiaoban.common.share.ShareSheetPalette
import xyz.xiangban.xiaoban.common.share.ShareThumbKind

internal val EmptyCommunityLinks = MutableStateFlow<String?>(null)
enum class CommunityPosterResult { Saved, ChooserOpened, Cancelled, Unsupported, PermissionDenied, Failed }

private fun CommunityPosterResult.toShareActionResult(forCopy: Boolean = false): ShareActionResult = when (this) {
    CommunityPosterResult.Saved -> if (forCopy) ShareActionResult.Copied else ShareActionResult.Saved
    CommunityPosterResult.ChooserOpened -> ShareActionResult.Shared
    CommunityPosterResult.Cancelled -> ShareActionResult.Cancelled
    CommunityPosterResult.Unsupported -> ShareActionResult.Unsupported
    CommunityPosterResult.PermissionDenied -> ShareActionResult.PermissionDenied
    CommunityPosterResult.Failed -> ShareActionResult.Failed
}

private fun PlatformActions.asShareBridge(): SharePlatformBridge = object : SharePlatformBridge {
    override val systemShareSupported: Boolean get() = communitySystemShareSupported
    override val cardExportSupported: Boolean get() = communityPosterSupported
    override val copyCardSupported: Boolean get() = communityPosterCopySupported

    override fun copyText(text: String) = this@asShareBridge.copy(text)
    override fun systemShare(title: String, url: String) = this@asShareBridge.share(title, url)
    override fun openInBrowser(url: String) = this@asShareBridge.openInBrowser(url)
    override suspend fun encodeQrMatrix(text: String): List<BooleanArray>? = encodeCommunityQr(text)
    override suspend fun renderShareCard(content: ShareContent, options: ShareCardOptions): ImageBitmap? =
        renderCommunityShareCard(content, options)
    override suspend fun saveShareCard(image: ImageBitmap): ShareActionResult =
        saveCommunityPoster(image).toShareActionResult()
    override suspend fun shareCardImage(image: ImageBitmap): ShareActionResult =
        shareCommunityPoster(image).toShareActionResult()
    override suspend fun copyCardImage(image: ImageBitmap): ShareActionResult =
        copyCommunityPoster(image).toShareActionResult(forCopy = true)
}

@Composable
private fun rememberSharePalette(): ShareSheetPalette {
    val extra = LocalExtraColors.current
    val scheme = MaterialTheme.colorScheme
    val dark = ThemeState.dark
    val defaults = ShareSheetPalette.defaults(
        dark = dark,
        accent = Brand,
        accentSoft = extra.brandSoft,
        accentInk = extra.brandInk,
    )
    return defaults.copy(
        paper = scheme.background,
        surface = scheme.surface,
        ink = scheme.onSurface,
        ink2 = scheme.onSurfaceVariant,
        ink3 = extra.ink3,
        hairline = extra.hairline,
        fill = extra.fill,
        line = extra.line,
        danger = Danger,
        dangerSoft = DangerSoft,
    )
}

@Composable
private fun ShareCoverImage(url: String) {
    AsyncImage(
        model = url,
        contentDescription = null,
        imageLoader = rememberImageLoader(),
        contentScale = ContentScale.Crop,
        modifier = Modifier.fillMaxSize(),
    )
}

private fun communityShareKindLabel(info: CommunityShare): String = when (info.kind) {
    "topic" -> "社区讨论"
    "pack" -> when (info.packKind) {
        "game" -> "创意游戏"
        "app" -> "精选工具"
        in CapabilityPackKinds -> "社区能力"
        else -> "社区作品"
    }
    "scheme" -> "桌面方案"
    "project" -> "开源项目"
    "weekly" -> "社区周刊"
    else -> "社区内容"
}

private fun communityShareBadge(info: CommunityShare): String = when {
    info.kind == "pack" && info.packKind in setOf("game", "app") -> "可试玩"
    info.kind == "project" -> "开源"
    info.kind == "weekly" -> "周刊"
    else -> "网页直达"
}

/**
 * 站内公开内容分享入口：先匿名核验公开分享 DTO（避免未公开或已下架内容外泄），
 * 再把必要字段交给 `:common:share` 的 [DetailShareSheet] 渲染。
 */
@Composable
internal fun CommunityShareSheet(
    env: CommunityEnv,
    url: String,
    heading: String = "分享",
    hint: String? = null,
    onDismiss: () -> Unit,
    onCopied: () -> Unit = {},
    panelModifier: Modifier = Modifier,
    author: String = "",
    dateLabel: String = "",
    coverUrl: String? = null,
    badge: String? = null,
    thumbKind: ShareThumbKind? = null,
    extraActions: List<ShareExtraAction> = emptyList(),
    deleteAction: ShareDeleteAction? = null,
    onNotice: ((String) -> Unit)? = null,
) {
    val api = remember(env) { CommunitySharingApi(env.client, env.config) }
    val target = remember(url, env.config) { communityShareTarget(url, env.config) }
    val loader = rememberLoader(env, "share-$url") {
        api.publicInfo(target ?: throw ApiException(400, "不支持分享此链接"))
    }
    val state by loader.state.collectAsState()
    val bridge = remember(env.platform) { env.platform?.asShareBridge() }
    val palette = rememberSharePalette()

    val shareState: ShareLoadState = when (val current = state) {
        is Load.Ready -> {
            val info = current.value
            val resolvedThumb = thumbKind ?: when {
                !coverUrl.isNullOrBlank() -> ShareThumbKind.Cover
                info.kind == "project" -> ShareThumbKind.Repo
                info.kind == "weekly" -> ShareThumbKind.Daily
                else -> ShareThumbKind.Text
            }
            val meta = hint?.takeIf(String::isNotBlank) ?: listOfNotNull(
                author.takeIf(String::isNotBlank),
                "公开链接",
                if (info.kind == "pack" && info.packKind in setOf("game", "app")) "浏览器可查看与试玩" else "浏览器免登录可看",
            ).joinToString(" · ")
            ShareLoadState.Ready(
                ShareContent(
                    title = info.title,
                    url = info.url,
                    summary = info.summary,
                    author = author,
                    dateLabel = dateLabel,
                    kindLabel = communityShareKindLabel(info),
                    meta = meta,
                    badge = badge ?: communityShareBadge(info),
                    coverUrl = coverUrl,
                    thumbKind = resolvedThumb,
                ),
            )
        }

        is Load.Failed -> ShareLoadState.Unavailable(
            message = if (current.notFound) "该内容可能未公开或已被作者下架，公开后即可生成分享链接与卡片。" else "分享信息加载失败，请检查网络后重试。",
            canRetry = true,
        )

        else -> ShareLoadState.Loading
    }

    DetailShareSheet(
        state = shareState,
        bridge = bridge,
        onDismiss = onDismiss,
        palette = palette,
        extraActions = extraActions,
        deleteAction = deleteAction,
        onRetry = loader::reload,
        onCopied = onCopied,
        onNotice = onNotice,
        coverContent = { imageUrl -> ShareCoverImage(imageUrl) },
        panelModifier = panelModifier,
    )
}

/** 已有公开分享数据的详情页（开源精选、AI 日报、创意作品、软件等）直连入口。 */
@Composable
internal fun DirectShareSheet(
    env: CommunityEnv,
    content: ShareContent,
    onDismiss: () -> Unit,
    onCopied: () -> Unit = {},
    extraActions: List<ShareExtraAction> = emptyList(),
    deleteAction: ShareDeleteAction? = null,
    onNotice: ((String) -> Unit)? = null,
    panelModifier: Modifier = Modifier,
) {
    val bridge = remember(env.platform) { env.platform?.asShareBridge() }
    val palette = rememberSharePalette()
    DetailShareSheet(
        content = content,
        bridge = bridge,
        onDismiss = onDismiss,
        palette = palette,
        extraActions = extraActions,
        deleteAction = deleteAction,
        onCopied = onCopied,
        onNotice = onNotice,
        coverContent = { imageUrl -> ShareCoverImage(imageUrl) },
        panelModifier = panelModifier,
    )
}
