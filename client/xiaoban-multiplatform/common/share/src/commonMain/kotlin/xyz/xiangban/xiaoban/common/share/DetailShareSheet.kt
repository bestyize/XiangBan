package xyz.xiangban.xiaoban.common.share

import androidx.compose.animation.AnimatedContent
import androidx.compose.animation.fadeIn
import androidx.compose.animation.fadeOut
import androidx.compose.animation.togetherWith
import androidx.compose.foundation.Canvas
import androidx.compose.foundation.background
import androidx.compose.foundation.border
import androidx.compose.foundation.clickable
import androidx.compose.foundation.horizontalScroll
import androidx.compose.foundation.layout.*
import androidx.compose.foundation.rememberScrollState
import androidx.compose.foundation.shape.CircleShape
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.foundation.verticalScroll
import androidx.compose.material3.CircularProgressIndicator
import androidx.compose.material3.Icon
import androidx.compose.material3.Text
import androidx.compose.runtime.*
import androidx.compose.ui.Alignment
import androidx.compose.ui.ExperimentalComposeUiApi
import androidx.compose.ui.Modifier
import androidx.compose.ui.backhandler.BackHandler
import androidx.compose.ui.draw.alpha
import androidx.compose.ui.draw.clip
import androidx.compose.ui.draw.shadow
import androidx.compose.ui.geometry.Offset
import androidx.compose.ui.geometry.Size
import androidx.compose.ui.graphics.Brush
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.graphics.PathEffect
import androidx.compose.ui.semantics.Role
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.style.TextAlign
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import kotlinx.coroutines.CancellationException
import kotlinx.coroutines.delay
import kotlinx.coroutines.launch

private enum class ShareStage { Sheet, ConfirmDelete, CardStudio }

/** 直接传入已准备好的 [ShareContent] 时使用的便捷入口。 */
@Composable
fun DetailShareSheet(
    content: ShareContent,
    bridge: SharePlatformBridge?,
    onDismiss: () -> Unit,
    palette: ShareSheetPalette = ShareSheetPalette.defaults(dark = false),
    extraActions: List<ShareExtraAction> = emptyList(),
    deleteAction: ShareDeleteAction? = null,
    onCopied: (() -> Unit)? = null,
    onNotice: ((String) -> Unit)? = null,
    coverContent: (@Composable (String) -> Unit)? = null,
    panelModifier: Modifier = Modifier,
) {
    DetailShareSheet(
        state = ShareLoadState.Ready(content),
        bridge = bridge,
        onDismiss = onDismiss,
        palette = palette,
        extraActions = extraActions,
        deleteAction = deleteAction,
        onRetry = null,
        onCopied = onCopied,
        onNotice = onNotice,
        coverContent = coverContent,
        panelModifier = panelModifier,
    )
}

/**
 * 通用详情页分享面板组件：
 * - 单行紧凑预览条 + 单行可横滑图标轨（右侧渐隐提示）+ 底部通栏扁平「取消」
 * - 支持主态末尾「删除」与内联单次确认
 * - 内置沉浸式「分享卡片」预览与导出工作台（纯白 / 暖纸笺 / 墨夜 + 摘录 / 二维码开关）
 */
@OptIn(ExperimentalComposeUiApi::class)
@Composable
fun DetailShareSheet(
    state: ShareLoadState,
    bridge: SharePlatformBridge?,
    onDismiss: () -> Unit,
    palette: ShareSheetPalette = ShareSheetPalette.defaults(dark = false),
    extraActions: List<ShareExtraAction> = emptyList(),
    deleteAction: ShareDeleteAction? = null,
    onRetry: (() -> Unit)? = null,
    onCopied: (() -> Unit)? = null,
    onNotice: ((String) -> Unit)? = null,
    coverContent: (@Composable (String) -> Unit)? = null,
    panelModifier: Modifier = Modifier,
) {
    val scope = rememberCoroutineScope()
    var stage by remember { mutableStateOf(ShareStage.Sheet) }
    var copied by remember { mutableStateOf(false) }
    var busy by remember { mutableStateOf(false) }
    var localToast by remember { mutableStateOf<String?>(null) }
    var cardOptions by remember { mutableStateOf(ShareCardOptions()) }

    val readyContent = (state as? ShareLoadState.Ready)?.content
    var qrMatrix by remember(readyContent?.url) { mutableStateOf<List<BooleanArray>?>(null) }
    LaunchedEffect(readyContent?.url, bridge) {
        val url = readyContent?.url
        qrMatrix = if (url != null && bridge != null) runCatching { bridge.encodeQrMatrix(url) }.getOrNull() else null
    }
    LaunchedEffect(copied) {
        if (copied) {
            delay(2000)
            copied = false
        }
    }
    LaunchedEffect(localToast) {
        if (localToast != null) {
            delay(1900)
            localToast = null
        }
    }

    fun showNotice(message: String?) {
        if (message.isNullOrBlank()) return
        localToast = message
        onNotice?.invoke(message)
    }

    fun runAsync(block: suspend () -> String?) {
        if (busy) return
        busy = true
        scope.launch {
            try {
                showNotice(block())
            } catch (e: CancellationException) {
                throw e
            } catch (e: Exception) {
                showNotice(e.message?.takeIf(String::isNotBlank) ?: "操作未完成，请重试")
            } finally {
                busy = false
            }
        }
    }

    BackHandler {
        when (stage) {
            ShareStage.Sheet -> onDismiss()
            ShareStage.ConfirmDelete, ShareStage.CardStudio -> stage = ShareStage.Sheet
        }
    }

    Box(Modifier.fillMaxSize()) {
        if (stage == ShareStage.CardStudio && readyContent != null) {
            ShareCardStudioOverlay(
                content = readyContent,
                options = cardOptions,
                qrMatrix = qrMatrix,
                palette = palette,
                busy = busy,
                copyCardSupported = bridge?.copyCardSupported == true,
                coverContent = coverContent,
                onOptionsChange = { cardOptions = it },
                onBack = { stage = ShareStage.Sheet },
                onDismiss = onDismiss,
                onSave = {
                    runAsync {
                        if (bridge == null || !bridge.cardExportSupported) return@runAsync shareActionNotice(ShareActionResult.Unsupported)
                        val image = bridge.renderShareCard(readyContent, cardOptions) ?: return@runAsync "卡片生成失败，请重试"
                        shareActionNotice(bridge.saveShareCard(image))
                    }
                },
                onSend = {
                    runAsync {
                        if (bridge == null || !bridge.cardExportSupported) return@runAsync shareActionNotice(ShareActionResult.Unsupported)
                        val image = bridge.renderShareCard(readyContent, cardOptions) ?: return@runAsync "卡片生成失败，请重试"
                        shareActionNotice(bridge.shareCardImage(image))
                    }
                },
                onCopyImage = {
                    runAsync {
                        if (bridge == null || !bridge.copyCardSupported) return@runAsync "当前平台暂不支持复制图片，可保存或发送"
                        val image = bridge.renderShareCard(readyContent, cardOptions) ?: return@runAsync "卡片生成失败，请重试"
                        shareActionNotice(bridge.copyCardImage(image))
                    }
                },
            )
        } else {
            val scrimColor = Color.Black.copy(alpha = if (palette.dark) 0.62f else 0.44f)
            Box(
                Modifier
                    .fillMaxSize()
                    .background(scrimColor)
                    .clickable(indication = null, interactionSource = null, onClick = onDismiss)
                    .imePadding(),
                contentAlignment = Alignment.BottomCenter,
            ) {
                Column(
                    panelModifier
                        .fillMaxWidth()
                        .shadow(16.dp, RoundedCornerShape(topStart = 18.dp, topEnd = 18.dp), clip = false)
                        .clip(RoundedCornerShape(topStart = 18.dp, topEnd = 18.dp))
                        .background(palette.surface)
                        .clickable(indication = null, interactionSource = null) {},
                ) {
                    // 顶部拖动把手
                    Box(
                        Modifier
                            .fillMaxWidth()
                            .padding(top = 8.dp, bottom = 12.dp),
                        contentAlignment = Alignment.Center,
                    ) {
                        Box(
                            Modifier
                                .size(36.dp, 4.dp)
                                .clip(CircleShape)
                                .background(palette.line),
                        )
                    }

                    when {
                        stage == ShareStage.ConfirmDelete && deleteAction != null -> {
                            DeleteConfirmPanel(
                                action = deleteAction,
                                palette = palette,
                                busy = busy,
                                onBack = { if (!busy) stage = ShareStage.Sheet },
                                onConfirm = {
                                    runAsync {
                                        val msg = deleteAction.onConfirm()
                                        onDismiss()
                                        msg
                                    }
                                },
                            )
                        }

                        state is ShareLoadState.Loading -> {
                            LoadingPanel(palette)
                        }

                        state is ShareLoadState.Unavailable -> {
                            UnavailablePanel(
                                message = state.message,
                                canRetry = state.canRetry && onRetry != null,
                                palette = palette,
                                onRetry = { onRetry?.invoke() },
                            )
                        }

                        readyContent != null -> {
                            ShareContextStrip(readyContent, palette, coverContent)
                            Spacer(Modifier.height(16.dp))
                            val railItems = remember(bridge?.systemShareSupported, copied, extraActions, deleteAction) {
                                buildShareRailItems(
                                    systemShareEnabled = bridge?.systemShareSupported == true,
                                    copied = copied,
                                    extraActions = extraActions,
                                    deleteAction = deleteAction,
                                )
                            }
                            ShareHorizontalRail(
                                items = railItems,
                                palette = palette,
                                busy = busy,
                                onSelect = { item ->
                                    when {
                                        item.id == "system" -> runAsync {
                                            bridge?.systemShare(readyContent.title, readyContent.url)
                                            "已唤起系统分享"
                                        }

                                        item.id == "copyLink" -> {
                                            if (bridge != null) {
                                                bridge.copyText(readyContent.url)
                                                copied = true
                                                onCopied?.invoke()
                                                showNotice("公开网页链接已复制")
                                            }
                                        }

                                        item.id == "openCard" -> {
                                            stage = ShareStage.CardStudio
                                        }

                                        item.id == "browser" -> runAsync {
                                            bridge?.openInBrowser(readyContent.url)
                                            null
                                        }

                                        item.id == "delete" && deleteAction != null -> {
                                            stage = ShareStage.ConfirmDelete
                                        }

                                        item.id.startsWith("extra:") -> {
                                            val extraId = item.id.removePrefix("extra:")
                                            val target = extraActions.firstOrNull { it.id == extraId }
                                            if (target != null && target.enabled) {
                                                runAsync { target.onClick() }
                                            }
                                        }
                                    }
                                },
                            )
                        }
                    }

                    // 底部通栏扁平「取消」条（删除确认态自带返回/确认双键，不重复展示取消）
                    if (stage != ShareStage.ConfirmDelete) {
                        Spacer(Modifier.height(12.dp))
                        Box(Modifier.fillMaxWidth().height(6.dp).background(palette.paper))
                        Box(
                            Modifier
                                .fillMaxWidth()
                                .height(50.dp)
                                .clickable(role = Role.Button, onClick = onDismiss),
                            contentAlignment = Alignment.Center,
                        ) {
                            Text(
                                text = "取消",
                                fontSize = 15.sp,
                                lineHeight = 22.sp,
                                fontWeight = FontWeight.Medium,
                                color = palette.ink2,
                            )
                        }
                    }
                    Spacer(Modifier.navigationBarsPadding())
                }
            }
        }

        // 面板内轻提示胶囊
        localToast?.let { text ->
            Box(
                Modifier
                    .fillMaxWidth()
                    .statusBarsPadding()
                    .padding(top = 60.dp, start = 24.dp, end = 24.dp),
                contentAlignment = Alignment.TopCenter,
            ) {
                Text(
                    text = text,
                    modifier = Modifier
                        .clip(CircleShape)
                        .background(if (palette.dark) Color(0xF0F0F0F4) else Color(0xEA18191E))
                        .padding(horizontal = 14.dp, vertical = 8.dp),
                    color = if (palette.dark) Color(0xFF16171A) else Color.White,
                    fontSize = 12.5.sp,
                    lineHeight = 18.sp,
                    fontWeight = FontWeight.Medium,
                    maxLines = 1,
                    overflow = TextOverflow.Ellipsis,
                )
            }
        }
    }
}

@Composable
private fun ShareContextStrip(
    content: ShareContent,
    palette: ShareSheetPalette,
    coverContent: (@Composable (String) -> Unit)?,
) {
    Row(
        modifier = Modifier
            .fillMaxWidth()
            .padding(horizontal = 16.dp)
            .clip(RoundedCornerShape(12.dp))
            .background(palette.paper)
            .border(1.dp, palette.hairline, RoundedCornerShape(12.dp))
            .padding(horizontal = 12.dp, vertical = 10.dp),
        verticalAlignment = Alignment.CenterVertically,
        horizontalArrangement = Arrangement.spacedBy(10.dp),
    ) {
        ShareContextThumb(content, palette, coverContent)
        Column(Modifier.weight(1f)) {
            Text(
                text = content.title,
                fontSize = 14.5.sp,
                lineHeight = 20.sp,
                fontWeight = FontWeight.SemiBold,
                color = palette.ink,
                maxLines = 1,
                overflow = TextOverflow.Ellipsis,
            )
            Spacer(Modifier.height(2.dp))
            Text(
                text = content.displayMeta,
                fontSize = 12.sp,
                lineHeight = 16.sp,
                color = palette.ink3,
                maxLines = 1,
                overflow = TextOverflow.Ellipsis,
            )
        }
        if (content.badge.isNotBlank()) {
            Text(
                text = content.badge,
                modifier = Modifier
                    .clip(CircleShape)
                    .background(palette.fill)
                    .padding(horizontal = 8.dp, vertical = 3.dp),
                fontSize = 11.sp,
                lineHeight = 15.sp,
                fontWeight = FontWeight.Medium,
                color = palette.ink2,
                maxLines = 1,
            )
        }
    }
}

@Composable
private fun ShareContextThumb(
    content: ShareContent,
    palette: ShareSheetPalette,
    coverContent: (@Composable (String) -> Unit)?,
) {
    val shape = RoundedCornerShape(10.dp)
    if (!content.coverUrl.isNullOrBlank() && coverContent != null) {
        Box(Modifier.size(42.dp).clip(shape).background(palette.fill)) {
            coverContent(content.coverUrl)
        }
        return
    }
    when (content.thumbKind) {
        ShareThumbKind.Repo -> {
            val monogram = content.thumbMonogram?.take(1)?.uppercase()
                ?: content.title.firstOrNull()?.uppercaseChar()?.toString()
                ?: "R"
            Box(
                Modifier.size(42.dp).clip(shape).background(Color(0xFF6965DB)),
                contentAlignment = Alignment.Center,
            ) {
                Text(monogram, color = Color.White, fontSize = 17.sp, fontWeight = FontWeight.Bold)
            }
        }

        ShareThumbKind.Daily -> {
            Box(
                Modifier
                    .size(42.dp)
                    .clip(shape)
                    .background(Brush.linearGradient(listOf(Color(0xFF1F2430), Color(0xFF3A4258)))),
                contentAlignment = Alignment.Center,
            ) {
                Icon(ShareIcons.Newspaper, contentDescription = null, tint = Color.White, modifier = Modifier.size(19.dp))
            }
        }

        ShareThumbKind.Text, ShareThumbKind.Cover -> {
            Box(
                Modifier
                    .size(42.dp)
                    .clip(shape)
                    .background(Brush.linearGradient(listOf(palette.accentSoft, palette.fill)))
                    .border(1.dp, palette.hairline, shape),
                contentAlignment = Alignment.Center,
            ) {
                Icon(ShareIcons.FileText, contentDescription = null, tint = palette.accentInk, modifier = Modifier.size(19.dp))
            }
        }
    }
}

@Composable
private fun ShareHorizontalRail(
    items: List<ShareRailItem>,
    palette: ShareSheetPalette,
    busy: Boolean,
    onSelect: (ShareRailItem) -> Unit,
) {
    val scroll = rememberScrollState()
    val showRightFade by remember { derivedStateOf { scroll.maxValue > 0 && scroll.value < scroll.maxValue - 4 } }

    Box(Modifier.fillMaxWidth()) {
        Row(
            modifier = Modifier
                .fillMaxWidth()
                .horizontalScroll(scroll)
                .padding(horizontal = 16.dp, vertical = 2.dp),
            horizontalArrangement = Arrangement.spacedBy(10.dp),
        ) {
            items.forEach { item ->
                ShareRailButton(
                    item = item,
                    palette = palette,
                    enabled = item.enabled && !busy,
                    onClick = { onSelect(item) },
                )
            }
        }
        if (showRightFade) {
            Box(
                Modifier
                    .align(Alignment.CenterEnd)
                    .width(28.dp)
                    .height(80.dp)
                    .background(
                        Brush.horizontalGradient(
                            colors = listOf(palette.surface.copy(alpha = 0f), palette.surface.copy(alpha = 0.94f)),
                        ),
                    ),
            )
        }
    }
}

@Composable
private fun ShareRailButton(
    item: ShareRailItem,
    palette: ShareSheetPalette,
    enabled: Boolean,
    onClick: () -> Unit,
) {
    val tileBg = when (item.tone) {
        ShareTileTone.Primary -> palette.accentSoft
        ShareTileTone.Copied -> palette.successSoft
        ShareTileTone.Danger -> palette.dangerSoft
        ShareTileTone.Default -> palette.fill
    }
    val tileBorder = when (item.tone) {
        ShareTileTone.Primary -> palette.accentSoftLine
        ShareTileTone.Copied -> palette.successLine
        else -> null
    }
    val iconTint = when (item.tone) {
        ShareTileTone.Primary -> palette.accentInk
        ShareTileTone.Copied -> palette.success
        ShareTileTone.Danger -> palette.danger
        ShareTileTone.Default -> palette.ink
    }
    val labelColor = when (item.tone) {
        ShareTileTone.Primary -> palette.ink
        ShareTileTone.Copied -> palette.success
        ShareTileTone.Danger -> palette.danger
        ShareTileTone.Default -> palette.ink2
    }
    val labelWeight = when (item.tone) {
        ShareTileTone.Copied -> FontWeight.SemiBold
        ShareTileTone.Primary, ShareTileTone.Danger -> FontWeight.Medium
        ShareTileTone.Default -> FontWeight.Normal
    }

    Column(
        modifier = Modifier
            .width(66.dp)
            .alpha(if (enabled) 1f else 0.42f)
            .clip(RoundedCornerShape(12.dp))
            .clickable(enabled = enabled, role = Role.Button, onClick = onClick)
            .padding(vertical = 2.dp),
        horizontalAlignment = Alignment.CenterHorizontally,
        verticalArrangement = Arrangement.spacedBy(7.dp),
    ) {
        val tileShape = RoundedCornerShape(16.dp)
        Box(
            modifier = Modifier
                .size(52.dp)
                .clip(tileShape)
                .background(tileBg)
                .then(if (tileBorder != null) Modifier.border(1.dp, tileBorder, tileShape) else Modifier),
            contentAlignment = Alignment.Center,
        ) {
            Icon(
                imageVector = ShareIcons.of(item.icon),
                contentDescription = item.label,
                tint = iconTint,
                modifier = Modifier.size(22.dp),
            )
            if (item.accentDot) {
                Box(
                    Modifier
                        .align(Alignment.TopEnd)
                        .padding(top = 7.dp, end = 7.dp)
                        .size(6.dp)
                        .clip(CircleShape)
                        .background(palette.accent),
                )
            }
        }
        Text(
            text = item.label,
            fontSize = 11.5.sp,
            lineHeight = 16.sp,
            fontWeight = labelWeight,
            color = labelColor,
            maxLines = 1,
            textAlign = TextAlign.Center,
        )
    }
}

@Composable
private fun LoadingPanel(palette: ShareSheetPalette) {
    Row(
        modifier = Modifier
            .fillMaxWidth()
            .padding(horizontal = 20.dp, vertical = 28.dp),
        horizontalArrangement = Arrangement.Center,
        verticalAlignment = Alignment.CenterVertically,
    ) {
        CircularProgressIndicator(
            modifier = Modifier.size(18.dp),
            strokeWidth = 2.dp,
            color = palette.ink3,
        )
        Spacer(Modifier.width(10.dp))
        Text("正在读取公开分享信息…", fontSize = 14.sp, color = palette.ink2)
    }
}

@Composable
private fun UnavailablePanel(
    message: String,
    canRetry: Boolean,
    palette: ShareSheetPalette,
    onRetry: () -> Unit,
) {
    Column(
        modifier = Modifier
            .fillMaxWidth()
            .padding(horizontal = 16.dp, vertical = 4.dp)
            .clip(RoundedCornerShape(14.dp))
            .background(palette.paper)
            .border(1.dp, palette.hairline, RoundedCornerShape(14.dp))
            .padding(horizontal = 16.dp, vertical = 20.dp),
        horizontalAlignment = Alignment.CenterHorizontally,
        verticalArrangement = Arrangement.spacedBy(8.dp),
    ) {
        Box(
            Modifier
                .size(42.dp)
                .clip(RoundedCornerShape(14.dp))
                .background(palette.dangerSoft),
            contentAlignment = Alignment.Center,
        ) {
            Icon(ShareIcons.Alert, contentDescription = null, tint = palette.danger, modifier = Modifier.size(20.dp))
        }
        Text("暂不支持分享此内容", fontSize = 15.sp, fontWeight = FontWeight.SemiBold, color = palette.ink)
        Text(
            text = message,
            fontSize = 12.5.sp,
            lineHeight = 18.sp,
            color = palette.ink2,
            textAlign = TextAlign.Center,
        )
        if (canRetry) {
            Spacer(Modifier.height(2.dp))
            Box(
                Modifier
                    .height(36.dp)
                    .clip(CircleShape)
                    .background(palette.fill)
                    .clickable(role = Role.Button, onClick = onRetry)
                    .padding(horizontal = 18.dp),
                contentAlignment = Alignment.Center,
            ) {
                Text("重新检查", fontSize = 13.sp, fontWeight = FontWeight.SemiBold, color = palette.ink)
            }
        }
    }
}

@Composable
private fun DeleteConfirmPanel(
    action: ShareDeleteAction,
    palette: ShareSheetPalette,
    busy: Boolean,
    onBack: () -> Unit,
    onConfirm: () -> Unit,
) {
    Column(
        modifier = Modifier
            .fillMaxWidth()
            .padding(start = 16.dp, end = 16.dp, top = 2.dp, bottom = 16.dp)
            .clip(RoundedCornerShape(14.dp))
            .background(palette.paper)
            .border(1.dp, palette.hairline, RoundedCornerShape(14.dp))
            .padding(horizontal = 16.dp, vertical = 18.dp),
        horizontalAlignment = Alignment.CenterHorizontally,
    ) {
        Text(action.confirmTitle, fontSize = 15.5.sp, fontWeight = FontWeight.SemiBold, color = palette.ink)
        Spacer(Modifier.height(5.dp))
        Text(
            text = action.confirmMessage,
            fontSize = 12.5.sp,
            lineHeight = 18.sp,
            color = palette.ink2,
            textAlign = TextAlign.Center,
        )
        Spacer(Modifier.height(16.dp))
        Row(Modifier.fillMaxWidth(), horizontalArrangement = Arrangement.spacedBy(10.dp)) {
            Box(
                Modifier
                    .weight(1f)
                    .height(40.dp)
                    .clip(CircleShape)
                    .background(palette.surface)
                    .border(1.dp, palette.hairline, CircleShape)
                    .clickable(enabled = !busy, role = Role.Button, onClick = onBack),
                contentAlignment = Alignment.Center,
            ) {
                Text("返回", fontSize = 14.sp, fontWeight = FontWeight.SemiBold, color = palette.ink2)
            }
            Box(
                Modifier
                    .weight(1f)
                    .height(40.dp)
                    .clip(CircleShape)
                    .background(palette.dangerSoft)
                    .alpha(if (busy) 0.5f else 1f)
                    .clickable(enabled = !busy, role = Role.Button, onClick = onConfirm),
                contentAlignment = Alignment.Center,
            ) {
                Text(
                    text = if (busy) "正在删除…" else action.confirmButton,
                    fontSize = 14.sp,
                    fontWeight = FontWeight.SemiBold,
                    color = palette.danger,
                )
            }
        }
    }
}

@Composable
private fun ShareCardStudioOverlay(
    content: ShareContent,
    options: ShareCardOptions,
    qrMatrix: List<BooleanArray>?,
    palette: ShareSheetPalette,
    busy: Boolean,
    copyCardSupported: Boolean,
    coverContent: (@Composable (String) -> Unit)?,
    onOptionsChange: (ShareCardOptions) -> Unit,
    onBack: () -> Unit,
    onDismiss: () -> Unit,
    onSave: () -> Unit,
    onSend: () -> Unit,
    onCopyImage: () -> Unit,
) {
    Box(
        Modifier
            .fillMaxSize()
            .background(Brush.verticalGradient(listOf(Color(0xD8101116), Color(0xEB0C0D11))))
            .clickable(indication = null, interactionSource = null) {},
    ) {
        Column(
            modifier = Modifier
                .fillMaxSize()
                .statusBarsPadding()
                .navigationBarsPadding(),
            verticalArrangement = Arrangement.SpaceBetween,
        ) {
            // 顶部极简透明操作栏
            Row(
                modifier = Modifier
                    .fillMaxWidth()
                    .padding(horizontal = 12.dp, vertical = 10.dp),
                verticalAlignment = Alignment.CenterVertically,
                horizontalArrangement = Arrangement.SpaceBetween,
            ) {
                Box(
                    Modifier
                        .size(38.dp)
                        .clip(CircleShape)
                        .clickable(role = Role.Button, onClick = onBack),
                    contentAlignment = Alignment.Center,
                ) {
                    Icon(ShareIcons.Back, contentDescription = "返回", tint = Color.White.copy(alpha = 0.92f), modifier = Modifier.size(20.dp))
                }
                Text("分享卡片", fontSize = 15.5.sp, fontWeight = FontWeight.SemiBold, color = Color.White)
                Box(
                    Modifier
                        .size(38.dp)
                        .clip(CircleShape)
                        .clickable(role = Role.Button, onClick = onDismiss),
                    contentAlignment = Alignment.Center,
                ) {
                    Icon(ShareIcons.Close, contentDescription = "关闭", tint = Color.White.copy(alpha = 0.92f), modifier = Modifier.size(20.dp))
                }
            }

            // 中央悬浮分享卡片 + 紧邻其下的磨砂定制条
            Column(
                modifier = Modifier
                    .weight(1f)
                    .fillMaxWidth()
                    .verticalScroll(rememberScrollState())
                    .padding(horizontal = 24.dp, vertical = 8.dp),
                horizontalAlignment = Alignment.CenterHorizontally,
                verticalArrangement = Arrangement.spacedBy(18.dp, Alignment.CenterVertically),
            ) {
                AnimatedContent(
                    targetState = options,
                    transitionSpec = { fadeIn() togetherWith fadeOut() },
                    label = "share-card-preview",
                ) { active ->
                    EditorialShareCardPreview(
                        content = content,
                        options = active,
                        qrMatrix = qrMatrix,
                        palette = palette,
                        coverContent = coverContent,
                    )
                }

                CardCustomizePill(
                    options = options,
                    hasExcerpt = content.summary.isNotBlank(),
                    onOptionsChange = onOptionsChange,
                )
            }

            // 底部通透三图标操作栏（无白底托盘、无宽大描边药丸）
            Row(
                modifier = Modifier
                    .fillMaxWidth()
                    .padding(horizontal = 24.dp, vertical = 14.dp),
                horizontalArrangement = Arrangement.spacedBy(34.dp, Alignment.CenterHorizontally),
                verticalAlignment = Alignment.Top,
            ) {
                FloatingCardAction(
                    icon = ShareIcons.Download,
                    label = "保存图片",
                    main = false,
                    accent = palette.accent,
                    enabled = !busy,
                    onClick = onSave,
                )
                FloatingCardAction(
                    icon = ShareIcons.Send,
                    label = "发送给好友",
                    main = true,
                    accent = palette.accent,
                    enabled = !busy,
                    onClick = onSend,
                )
                if (copyCardSupported) {
                    FloatingCardAction(
                        icon = ShareIcons.CopyText,
                        label = "复制图片",
                        main = false,
                        accent = palette.accent,
                        enabled = !busy,
                        onClick = onCopyImage,
                    )
                }
            }
        }
    }
}

@Composable
private fun EditorialShareCardPreview(
    content: ShareContent,
    options: ShareCardOptions,
    qrMatrix: List<BooleanArray>?,
    palette: ShareSheetPalette,
    coverContent: (@Composable (String) -> Unit)?,
) {
    val bg: Color
    val ink: Color
    val muted: Color
    val softBg: Color
    val lineColor: Color
    val borderColor: Color
    val quoteBar: Color
    when (options.style) {
        ShareCardStyle.White -> {
            bg = Color(0xFFFFFFFF)
            ink = Color(0xFF1F2024)
            muted = Color(0xFF6E7078)
            softBg = Color(0xFFF4F4F6)
            lineColor = Color(0xFFECECEF)
            borderColor = Color.White.copy(alpha = 0.55f)
            quoteBar = palette.accentSoftLine
        }

        ShareCardStyle.Warm -> {
            bg = Color(0xFFF8F4EC)
            ink = Color(0xFF2B231C)
            muted = Color(0xFF7C6F62)
            softBg = Color(0xFFEFE8DC)
            lineColor = Color(0xFFE5DCCD)
            borderColor = Color(0x66FFF5E6)
            quoteBar = Color(0xFFD9C3A8)
        }

        ShareCardStyle.Ink -> {
            bg = Color(0xFF1C1D22)
            ink = Color(0xFFF2F2F5)
            muted = Color(0xFF989AA4)
            softBg = Color(0xFF27282E)
            lineColor = Color(0xFF2D2E35)
            borderColor = Color.White.copy(alpha = 0.12f)
            quoteBar = Color.White.copy(alpha = 0.24f)
        }
    }

    val cardShape = RoundedCornerShape(18.dp)
    Column(
        modifier = Modifier
            .widthIn(max = 304.dp)
            .fillMaxWidth()
            .shadow(24.dp, cardShape, clip = false)
            .clip(cardShape)
            .background(bg)
            .border(1.dp, borderColor, cardShape)
            .padding(start = 18.dp, end = 18.dp, top = 18.dp, bottom = 16.dp),
    ) {
        // 顶部品牌行
        Row(
            modifier = Modifier.fillMaxWidth(),
            verticalAlignment = Alignment.CenterVertically,
            horizontalArrangement = Arrangement.SpaceBetween,
        ) {
            Row(verticalAlignment = Alignment.CenterVertically, horizontalArrangement = Arrangement.spacedBy(6.dp)) {
                Box(
                    Modifier
                        .size(20.dp)
                        .clip(RoundedCornerShape(6.dp))
                        .background(palette.accent),
                    contentAlignment = Alignment.Center,
                ) {
                    Icon(ShareIcons.Sparkles, contentDescription = null, tint = Color.White, modifier = Modifier.size(12.dp))
                }
                Text("小伴社区", fontSize = 12.sp, fontWeight = FontWeight.SemiBold, color = ink)
            }
            if (content.kindLabel.isNotBlank()) {
                Text(
                    text = content.kindLabel,
                    modifier = Modifier
                        .clip(CircleShape)
                        .background(softBg)
                        .padding(horizontal = 8.dp, vertical = 2.dp),
                    fontSize = 10.5.sp,
                    fontWeight = FontWeight.Medium,
                    color = muted,
                )
            }
        }

        if (!content.coverUrl.isNullOrBlank() && coverContent != null) {
            Spacer(Modifier.height(12.dp))
            Box(
                Modifier
                    .fillMaxWidth()
                    .height(112.dp)
                    .clip(RoundedCornerShape(10.dp))
                    .background(softBg),
            ) {
                coverContent(content.coverUrl)
            }
        }

        Spacer(Modifier.height(12.dp))
        Text(
            text = content.title,
            fontSize = 16.5.sp,
            lineHeight = 23.5.sp,
            fontWeight = FontWeight.Bold,
            color = ink,
            maxLines = 3,
            overflow = TextOverflow.Ellipsis,
        )

        if (options.showExcerpt && content.summary.isNotBlank()) {
            Spacer(Modifier.height(10.dp))
            Row(Modifier.fillMaxWidth().height(IntrinsicSize.Min)) {
                Box(Modifier.width(2.dp).fillMaxHeight().background(quoteBar))
                Spacer(Modifier.width(10.dp))
                Text(
                    text = content.summary.replace(Regex("\\s+"), " ").trim(),
                    fontSize = 12.5.sp,
                    lineHeight = 19.5.sp,
                    color = muted,
                    maxLines = 3,
                    overflow = TextOverflow.Ellipsis,
                )
            }
        }

        if (content.author.isNotBlank() || content.dateLabel.isNotBlank()) {
            Spacer(Modifier.height(12.dp))
            Row(verticalAlignment = Alignment.CenterVertically, horizontalArrangement = Arrangement.spacedBy(6.dp)) {
                val initial = (content.author.firstOrNull() ?: content.title.firstOrNull() ?: '伴').uppercaseChar().toString()
                Box(
                    Modifier
                        .size(20.dp)
                        .clip(CircleShape)
                        .background(if (content.thumbKind == ShareThumbKind.Repo) Color(0xFF6965DB) else palette.accent),
                    contentAlignment = Alignment.Center,
                ) {
                    Text(initial, fontSize = 10.sp, fontWeight = FontWeight.Bold, color = Color.White)
                }
                if (content.author.isNotBlank()) {
                    Text(content.author, fontSize = 11.5.sp, fontWeight = FontWeight.SemiBold, color = muted, maxLines = 1, overflow = TextOverflow.Ellipsis)
                }
                if (content.author.isNotBlank() && content.dateLabel.isNotBlank()) {
                    Text("·", fontSize = 11.5.sp, color = muted)
                }
                if (content.dateLabel.isNotBlank()) {
                    Text(content.dateLabel, fontSize = 11.5.sp, color = muted, maxLines = 1)
                }
            }
        }

        // 虚线分隔
        Spacer(Modifier.height(14.dp))
        Canvas(Modifier.fillMaxWidth().height(1.dp)) {
            drawLine(
                color = lineColor,
                start = Offset(0f, 0f),
                end = Offset(size.width, 0f),
                strokeWidth = 2f,
                pathEffect = PathEffect.dashPathEffect(floatArrayOf(8f, 6f), 0f),
            )
        }
        Spacer(Modifier.height(12.dp))

        Row(
            modifier = Modifier.fillMaxWidth(),
            verticalAlignment = Alignment.CenterVertically,
            horizontalArrangement = Arrangement.SpaceBetween,
        ) {
            Column(Modifier.weight(1f)) {
                Text(
                    text = if (options.showQr) "扫码在浏览器查看" else "来自小伴社区",
                    fontSize = 11.5.sp,
                    fontWeight = FontWeight.SemiBold,
                    color = ink,
                )
                Spacer(Modifier.height(2.dp))
                Text(
                    text = content.displayUrl,
                    fontSize = 10.5.sp,
                    color = muted,
                    maxLines = 1,
                    overflow = TextOverflow.Ellipsis,
                )
            }
            if (options.showQr) {
                Spacer(Modifier.width(10.dp))
                Box(
                    Modifier
                        .size(50.dp)
                        .clip(RoundedCornerShape(8.dp))
                        .background(Color.White)
                        .border(1.dp, Color.Black.copy(alpha = 0.06f), RoundedCornerShape(8.dp))
                        .padding(4.dp),
                ) {
                    QrMatrixCanvas(
                        matrix = qrMatrix,
                        color = if (options.style == ShareCardStyle.Warm) Color(0xFF2B231C) else Color(0xFF1B1C20),
                        modifier = Modifier.fillMaxSize(),
                    )
                }
            }
        }
    }
}

@Composable
private fun QrMatrixCanvas(matrix: List<BooleanArray>?, color: Color, modifier: Modifier = Modifier) {
    Canvas(modifier) {
        if (matrix != null && matrix.isNotEmpty()) {
            val rows = matrix.size
            val cols = matrix[0].size
            if (rows > 0 && cols > 0) {
                val cellW = size.width / cols
                val cellH = size.height / rows
                for (y in 0 until rows) {
                    val row = matrix[y]
                    for (x in 0 until cols) {
                        if (row[x]) {
                            drawRect(
                                color = color,
                                topLeft = Offset(x * cellW, y * cellH),
                                size = Size(cellW + 0.4f, cellH + 0.4f),
                            )
                        }
                    }
                }
                return@Canvas
            }
        }
        // 平台未提供编码矩阵时的示意二维码图形
        val cell = size.width / 7f
        fun finder(gx: Int, gy: Int) {
            drawRect(color, Offset(gx * cell, gy * cell), Size(3 * cell, 3 * cell))
            drawRect(Color.White, Offset((gx + 0.6f) * cell, (gy + 0.6f) * cell), Size(1.8f * cell, 1.8f * cell))
            drawRect(color, Offset((gx + 1.1f) * cell, (gy + 1.1f) * cell), Size(0.8f * cell, 0.8f * cell))
        }
        finder(0, 0)
        finder(4, 0)
        finder(0, 4)
        drawRect(color, Offset(4f * cell, 4f * cell), Size(cell, cell))
        drawRect(color, Offset(5.6f * cell, 5.6f * cell), Size(1.4f * cell, 1.4f * cell))
    }
}

@Composable
private fun CardCustomizePill(
    options: ShareCardOptions,
    hasExcerpt: Boolean,
    onOptionsChange: (ShareCardOptions) -> Unit,
) {
    Row(
        modifier = Modifier
            .clip(CircleShape)
            .background(Color.White.copy(alpha = 0.11f))
            .border(1.dp, Color.White.copy(alpha = 0.13f), CircleShape)
            .padding(horizontal = 12.dp, vertical = 6.dp),
        verticalAlignment = Alignment.CenterVertically,
        horizontalArrangement = Arrangement.spacedBy(10.dp),
    ) {
        Row(horizontalArrangement = Arrangement.spacedBy(9.dp), verticalAlignment = Alignment.CenterVertically) {
            ToneDot(
                color = Color.White,
                selected = options.style == ShareCardStyle.White,
                onClick = { onOptionsChange(options.copy(style = ShareCardStyle.White)) },
            )
            ToneDot(
                color = Color(0xFFF3E8D5),
                selected = options.style == ShareCardStyle.Warm,
                onClick = { onOptionsChange(options.copy(style = ShareCardStyle.Warm)) },
            )
            ToneDot(
                color = Color(0xFF1D1E24),
                selected = options.style == ShareCardStyle.Ink,
                onClick = { onOptionsChange(options.copy(style = ShareCardStyle.Ink)) },
            )
        }

        Box(Modifier.width(1.dp).height(14.dp).background(Color.White.copy(alpha = 0.18f)))

        if (hasExcerpt) {
            GlassToggleChip(
                label = "摘录",
                on = options.showExcerpt,
                onClick = { onOptionsChange(options.copy(showExcerpt = !options.showExcerpt)) },
            )
        }
        GlassToggleChip(
            label = "二维码",
            on = options.showQr,
            onClick = { onOptionsChange(options.copy(showQr = !options.showQr)) },
        )
    }
}

@Composable
private fun ToneDot(color: Color, selected: Boolean, onClick: () -> Unit) {
    Box(
        modifier = Modifier
            .size(24.dp)
            .clip(CircleShape)
            .then(if (selected) Modifier.border(2.dp, Color.White.copy(alpha = 0.88f), CircleShape) else Modifier)
            .padding(3.dp)
            .clip(CircleShape)
            .background(color)
            .border(1.dp, Color.White.copy(alpha = 0.35f), CircleShape)
            .clickable(role = Role.RadioButton, onClick = onClick),
    )
}

@Composable
private fun GlassToggleChip(label: String, on: Boolean, onClick: () -> Unit) {
    Row(
        modifier = Modifier
            .height(24.dp)
            .clip(CircleShape)
            .background(if (on) Color.White.copy(alpha = 0.17f) else Color.Transparent)
            .clickable(role = Role.Checkbox, onClick = onClick)
            .padding(horizontal = 8.dp),
        verticalAlignment = Alignment.CenterVertically,
        horizontalArrangement = Arrangement.spacedBy(4.dp),
    ) {
        Box(
            Modifier
                .size(6.dp)
                .clip(CircleShape)
                .background(if (on) Color.White else Color.White.copy(alpha = 0.32f)),
        )
        Text(
            text = label,
            fontSize = 11.5.sp,
            fontWeight = FontWeight.Medium,
            color = if (on) Color.White else Color.White.copy(alpha = 0.58f),
        )
    }
}

@Composable
private fun FloatingCardAction(
    icon: androidx.compose.ui.graphics.vector.ImageVector,
    label: String,
    main: Boolean,
    accent: Color,
    enabled: Boolean,
    onClick: () -> Unit,
) {
    val shape = RoundedCornerShape(18.dp)
    Column(
        modifier = Modifier
            .alpha(if (enabled) 1f else 0.45f)
            .clip(RoundedCornerShape(14.dp))
            .clickable(enabled = enabled, role = Role.Button, onClick = onClick)
            .padding(horizontal = 4.dp, vertical = 2.dp),
        horizontalAlignment = Alignment.CenterHorizontally,
        verticalArrangement = Arrangement.spacedBy(8.dp),
    ) {
        Box(
            modifier = Modifier
                .size(54.dp)
                .then(if (main) Modifier.shadow(14.dp, shape, clip = false) else Modifier)
                .clip(shape)
                .background(if (main) Color.White else Color.White.copy(alpha = 0.14f))
                .then(if (!main) Modifier.border(1.dp, Color.White.copy(alpha = 0.15f), shape) else Modifier),
            contentAlignment = Alignment.Center,
        ) {
            Icon(
                imageVector = icon,
                contentDescription = label,
                tint = if (main) accent else Color.White,
                modifier = Modifier.size(22.dp),
            )
        }
        Text(
            text = label,
            fontSize = 12.sp,
            lineHeight = 16.sp,
            fontWeight = if (main) FontWeight.SemiBold else FontWeight.Medium,
            color = if (main) Color.White else Color.White.copy(alpha = 0.84f),
        )
    }
}
