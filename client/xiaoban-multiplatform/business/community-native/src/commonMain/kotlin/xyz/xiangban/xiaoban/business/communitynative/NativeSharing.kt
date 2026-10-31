package xyz.xiangban.xiaoban.business.communitynative

import androidx.compose.foundation.Image
import androidx.compose.foundation.layout.*
import androidx.compose.foundation.rememberScrollState
import androidx.compose.foundation.verticalScroll
import androidx.compose.material3.*
import androidx.compose.runtime.*
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.ImageBitmap
import androidx.compose.ui.layout.ContentScale
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import kotlinx.coroutines.CancellationException
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.launch
import xyz.xiangban.xiaoban.common.community.*

internal val EmptyCommunityLinks = MutableStateFlow<String?>(null)
enum class CommunityPosterResult { Saved, ChooserOpened, Cancelled, Unsupported, PermissionDenied, Failed }

/** Only the anonymous share DTO is rendered or sent to another application. */
@Composable
internal fun CommunityShareSheet(env: CommunityEnv, url: String, heading: String, hint: String?, onDismiss: () -> Unit, onCopied: () -> Unit, panelModifier: Modifier = Modifier) {
    val api = remember(env) { CommunitySharingApi(env.client, env.config) }
    val target = remember(url, env.config) { communityShareTarget(url, env.config) }
    val loader = rememberLoader(env, "share-$url") { api.publicInfo(target ?: throw ApiException(400, "不支持分享此链接")) }
    val state by loader.state.collectAsState()
    val scope = rememberCoroutineScope()
    var poster by remember(url) { mutableStateOf<ImageBitmap?>(null) }
    var busy by remember(url) { mutableStateOf(false) }
    var notice by remember(url) { mutableStateOf<String?>(null) }
    val platform = env.platform
    fun perform(action: suspend () -> String?) {
        if (busy) return
        busy = true; notice = null
        scope.launch {
            try { notice = action() }
            catch (e: CancellationException) { throw e }
            catch (_: Exception) { notice = "操作未完成，请重试，也可以复制链接" }
            finally { busy = false }
        }
    }
    BottomSheet(onDismiss, panelModifier) {
        Column(Modifier.fillMaxWidth().heightIn(max = 680.dp).verticalScroll(rememberScrollState()), verticalArrangement = Arrangement.spacedBy(10.dp)) {
            Text(heading, fontSize = 18.sp, fontWeight = FontWeight.SemiBold)
            when (val current = state) {
                is Load.Ready -> {
                    val info = current.value
                    Text(info.title, fontSize = 17.sp, fontWeight = FontWeight.Medium)
                    if (info.summary.isNotBlank()) Text(info.summary, maxLines = 3, fontSize = 13.sp, color = MaterialTheme.colorScheme.onSurfaceVariant)
                    Text(hint ?: "分享公开网页链接，未安装小伴也能在浏览器查看。", fontSize = 13.sp, color = MaterialTheme.colorScheme.onSurfaceVariant)
                    Row(horizontalArrangement = Arrangement.spacedBy(8.dp)) {
                        GhostPillButton("复制链接", { perform { platform?.copy(info.url); onCopied(); "链接已复制" } }, Modifier.weight(1f), enabled = platform != null && !busy)
                        PillButton("系统分享", { perform { platform?.share(info.title, info.url); "已打开系统分享，可选择可用的应用" } }, Modifier.weight(1f), enabled = platform?.communitySystemShareSupported == true && !busy)
                    }
                    GhostPillButton("浏览器查看", { perform { platform?.openUrl(info.url); null } }, Modifier.fillMaxWidth(), enabled = platform != null && !busy)
                    if (platform?.communityPosterSupported == true) {
                        val image = poster
                        if (image == null) GhostPillButton(if (busy) "正在生成…" else "生成分享海报", { perform { poster = platform.createCommunityPoster(info); if (poster == null) "海报生成失败，请重试" else null } }, Modifier.fillMaxWidth(), enabled = !busy)
                        else {
                            Image(image, "分享海报预览", Modifier.fillMaxWidth().height(240.dp), contentScale = ContentScale.Fit)
                            Row(horizontalArrangement = Arrangement.spacedBy(8.dp)) {
                                GhostPillButton("保存海报", { perform { posterNotice(platform.saveCommunityPoster(image)) } }, Modifier.weight(1f), enabled = !busy)
                                PillButton("分享海报", { perform { posterNotice(platform.shareCommunityPoster(image)) } }, Modifier.weight(1f), enabled = !busy)
                            }
                        }
                    } else Text("当前平台暂不支持海报保存，可复制链接或在浏览器打开。", fontSize = 12.sp, color = MaterialTheme.colorScheme.onSurfaceVariant)
                }
                is Load.Failed -> {
                    Text(if (current.notFound) "此内容未公开或已下架，暂不能分享。" else "分享信息加载失败，请重试。", fontSize = 14.sp)
                    GhostPillButton("重试", loader::reload, Modifier.fillMaxWidth())
                }
                else -> Text("正在读取公开分享信息…", fontSize = 14.sp)
            }
            notice?.let { Text(it, fontSize = 13.sp, color = MaterialTheme.colorScheme.primary) }
            if (busy) LinearProgressIndicator(Modifier.fillMaxWidth())
            GhostPillButton("关闭", onDismiss, Modifier.fillMaxWidth())
        }
    }
}

private fun posterNotice(result: CommunityPosterResult): String = when (result) {
    CommunityPosterResult.Saved -> "海报已保存"
    CommunityPosterResult.ChooserOpened -> "已打开系统分享，请选择可用的应用"
    CommunityPosterResult.Cancelled -> "已取消保存，可以重试"
    CommunityPosterResult.Unsupported -> "当前平台暂不支持海报，可复制链接"
    CommunityPosterResult.PermissionDenied -> "未获得保存授权，请重试选择保存位置，或分享海报"
    CommunityPosterResult.Failed -> "操作未完成，请重试，也可以复制链接"
}

/** Every incoming link is checked, resolved anonymously, then confirmed. Never auto-apply a pack. */
@Composable
internal fun CommunityLinkConfirmation(env: CommunityEnv, open: (Route) -> Unit) {
    val platform = env.platform ?: return
    val raw by platform.communityLinks.collectAsState()
    val pending = raw ?: return
    val target = remember(pending, env.config) { communityShareTarget(pending, env.config) }
    val api = remember(env) { CommunitySharingApi(env.client, env.config) }
    val loader = rememberLoader(env, "incoming-$pending") { api.publicInfo(target ?: throw ApiException(400, "此链接不是可打开的公开社区内容")) }
    val state by loader.state.collectAsState()
    AlertDialog(
        onDismissRequest = platform::consumeCommunityLink,
        title = { Text("打开社区内容") },
        text = {
            Column(verticalArrangement = Arrangement.spacedBy(8.dp)) {
                when (val current = state) {
                    is Load.Ready -> {
                        Text(current.value.title)
                        Text(if (target?.requestsApply == true) "此链接指向桌面内容。手机只查看详情，不会自动下载或应用。" else "确认后查看公开社区详情。")
                    }
                    is Load.Failed -> Text(if (current.notFound) "内容未公开或已下架。" else "无法读取此公开链接，可重试或取消。")
                    else -> Text("正在检查公开内容…")
                }
            }
        },
        confirmButton = {
            val info = (state as? Load.Ready)?.value
            if (target != null && info != null) TextButton(onClick = {
                platform.consumeCommunityLink()
                open(when (target.kind) {
                    "pack" -> if (info.packKind in CapabilityPackKinds) Route.Ability(target.id.toLong()) else Route.Pack(target.id.toLong())
                    "topic" -> Route.Topic(target.id.toLong())
                    "scheme" -> Route.Scheme(target.id.toLong())
                    "project" -> Route.Project(target.id)
                    else -> Route.Weekly(target.id)
                })
            }) { Text("查看详情") }
            else if (target != null && state is Load.Failed) TextButton(onClick = loader::reload) { Text("重试") }
        },
        dismissButton = { TextButton(onClick = platform::consumeCommunityLink) { Text("取消") } },
    )
}
