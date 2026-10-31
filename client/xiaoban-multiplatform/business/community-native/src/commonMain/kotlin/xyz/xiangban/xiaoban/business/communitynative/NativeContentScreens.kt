package xyz.xiangban.xiaoban.business.communitynative

import androidx.compose.foundation.background
import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.*
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.items
import androidx.compose.foundation.lazy.itemsIndexed
import androidx.compose.foundation.rememberScrollState
import androidx.compose.foundation.text.selection.SelectionContainer
import androidx.compose.foundation.verticalScroll
import androidx.compose.material3.Checkbox
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.*
import androidx.compose.runtime.saveable.rememberSaveable
import androidx.compose.ui.Modifier
import androidx.compose.ui.ExperimentalComposeUiApi
import androidx.compose.ui.backhandler.BackHandler
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import kotlinx.coroutines.CancellationException
import kotlinx.coroutines.launch
import kotlinx.serialization.encodeToString
import kotlinx.serialization.json.JsonObject
import xyz.xiangban.xiaoban.common.community.*

private val ContentSpacing = Arrangement.spacedBy(8.dp)

@Composable
private fun ManifestSection(title: String, value: String?) {
    WhiteCard(padding = 14.dp) {
        Text(title, fontSize = 15.sp, fontWeight = FontWeight.SemiBold)
        SelectionContainer { Text(value ?: "发布者未提供此项说明", Modifier.padding(top = 8.dp), fontSize = 14.sp, lineHeight = 22.sp, color = MaterialTheme.colorScheme.onSurfaceVariant) }
    }
}

@Composable
fun AbilityDetailScreen(env: CommunityEnv, id: Long, back: () -> Unit) {
    val loader = rememberLoader(env, "ability-$id") { env.forum.pack(id) }
    val state by loader.state.collectAsState()
    Column(Modifier.fillMaxSize()) {
        ScreenBar("能力详情", onBack = back)
        Box(Modifier.weight(1f)) { LoadBox(state, loader::reload) { pack -> CapabilityDetailContent(env, pack, back, showBar = false) } }
    }
}

/** 既有 Pack 入口也可委托至此，保持视觉内容的详情代码原样。 */
@Composable
internal fun CapabilityDetailContent(env: CommunityEnv, pack: Pack, back: () -> Unit, showBar: Boolean = true) {
    val manifest = pack.versions.firstOrNull { it.version == pack.version }?.manifest ?: JsonObject(emptyMap())
    val prompt = manifestValue(manifest, "prompt", "promptText", "systemPrompt", "system_prompt")
    Column(Modifier.fillMaxSize()) {
        if (showBar) ScreenBar("能力详情", onBack = back, trailing = { IconTap(XIcons.Share, "分享", { env.platform?.share(pack.title, env.forum.packUrl(pack.id)) }) })
        Column(Modifier.weight(1f).fillMaxWidth().verticalScroll(rememberScrollState()).padding(12.dp), verticalArrangement = ContentSpacing) {
            WhiteCard(padding = 16.dp) {
                Text(pack.title, fontSize = 22.sp, fontWeight = FontWeight.SemiBold)
                Text("${kindLabel(pack.kind)} · ${pack.trustLabel} · v${pack.version}", Modifier.padding(top = 6.dp), fontSize = 12.sp, color = LocalExtraColors.current.ink3)
                if (pack.status != "published") Text(contentWorkState(pack), Modifier.padding(top = 8.dp), color = Danger, fontSize = 13.sp)
                if (pack.summary.isNotBlank()) Text(pack.summary, Modifier.padding(top = 10.dp), fontSize = 14.sp, lineHeight = 22.sp)
            }
            ManifestSection("能做", manifestValue(manifest, "canDo", "can_do", "capabilities"))
            ManifestSection("不能做", manifestValue(manifest, "cannotDo", "cannot_do", "limitations"))
            ManifestSection("所需权限", manifestValue(manifest, "permissions", "requiredPermissions"))
            ManifestSection("运行位置", manifestValue(manifest, "runLocation", "run_location", "runtime", "executionLocation"))
            ManifestSection("完整提示词", prompt)
            if (prompt != null) GhostPillButton("复制提示词", { env.platform?.copy(prompt) }, Modifier.fillMaxWidth(), enabled = env.platform != null)
            var expanded by rememberSaveable { mutableStateOf(false) }
            GhostPillButton(if (expanded) "收起完整清单" else "查看完整清单", { expanded = !expanded }, Modifier.fillMaxWidth())
            if (expanded) ManifestSection("资源清单原文", CommunityJson.encodeToString(manifest))
            FootNote("手机用于浏览和分享。内容的实际运行与授权在电脑端确认，导入内容不会自动执行。未提供的权限说明不能视为没有权限。")
            GhostPillButton("复制电脑端打开链接", { env.platform?.copy(env.forum.packUrl(pack.id)) }, Modifier.fillMaxWidth(), enabled = env.platform != null && pack.status == "published")
            pack.topicId?.let { topicId -> GhostPillButton("在网页查看讨论", { env.platform?.openUrl(env.forum.topicUrl(topicId)) }, Modifier.fillMaxWidth(), enabled = env.platform != null) }
        }
    }
}

@Composable
fun SchemeDetailScreen(env: CommunityEnv, id: Long, back: () -> Unit, open: (Route) -> Unit) {
    val loader = rememberLoader(env, "scheme-$id") { env.content.scheme(id) }
    val state by loader.state.collectAsState()
    Column(Modifier.fillMaxSize()) {
        ScreenBar("桌面方案", onBack = back)
        LoadBox(state, loader::reload) { scheme ->
            LazyColumn(Modifier.fillMaxSize(), contentPadding = PaddingValues(12.dp), verticalArrangement = ContentSpacing) {
                item { WhiteCard(padding = 16.dp) {
                    Text(scheme.title, fontSize = 22.sp, fontWeight = FontWeight.SemiBold)
                    Text("${scheme.author.nickname ?: "作者"} · ${scheme.createdAt.take(10)}", Modifier.padding(top = 6.dp), fontSize = 12.sp, color = LocalExtraColors.current.ink3)
                    if (scheme.summary.isNotBlank()) Text(scheme.summary, Modifier.padding(top = 10.dp), fontSize = 14.sp, lineHeight = 22.sp)
                } }
                item { Text("包含的内容", Modifier.padding(4.dp), fontSize = 16.sp, fontWeight = FontWeight.SemiBold) }
                itemsIndexed(scheme.items, key = { index, item -> "$index-${item.packId}-${item.version}" }) { _, item -> WhiteCard(padding = 14.dp) {
                    Text(if (item.available) item.title ?: item.slug else "内容不可用", fontSize = 15.sp, fontWeight = FontWeight.Medium)
                    Text(if (item.available) "${kindLabel(item.kind)} · 方案版本 v${item.version}" else "此项已下架或当前不可访问", Modifier.padding(top = 6.dp), fontSize = 12.sp, color = LocalExtraColors.current.ink3)
                    if (item.available) GhostPillButton("查看内容", { open(if (item.kind in CapabilityPackKinds) Route.Ability(item.packId) else Route.Pack(item.packId)) }, Modifier.padding(top = 8.dp))
                } }
                item { FootNote("方案是一份内容清单。手机不会应用桌面内容；在电脑上打开后，逐项检查版本、可用性和权限再确认。") }
                item { GhostPillButton("分享方案到电脑", { env.platform?.share(scheme.title, "${env.config.forumBase.trimEnd('/')}/schemes/${scheme.id}") }, Modifier.fillMaxWidth(), enabled = env.platform != null) }
            }
        }
    }
}

/** 创作栏的“方案”读取 schemes，不把它作为压缩包 kind。 */
@Composable
internal fun NativeSchemesFeed(env: CommunityEnv, open: (Route) -> Unit) {
    var page by rememberSaveable { mutableStateOf(1) }
    val loader = rememberLoader(env, "scheme-page-$page") { env.content.schemes(page) }
    val state by loader.state.collectAsState()
    LoadBox(state, loader::reload) { result ->
        if (result.items.isEmpty()) CenterMessage("这里还没有桌面方案", "刷新", loader::reload)
        else LazyColumn(Modifier.fillMaxSize(), contentPadding = FeedPadding, verticalArrangement = ContentSpacing) {
            items(result.items, key = { it.id }) { scheme -> WhiteCard(Modifier.clickable { open(Route.Scheme(scheme.id)) }, padding = 14.dp) {
                Text(scheme.title, fontSize = 16.sp, fontWeight = FontWeight.SemiBold)
                Text("${scheme.items.size} 项内容 · ${scheme.author.nickname ?: "作者"}", Modifier.padding(top = 8.dp), fontSize = 12.sp, color = LocalExtraColors.current.ink3)
            } }
            item { Row(Modifier.fillMaxWidth(), horizontalArrangement = Arrangement.spacedBy(8.dp)) {
                GhostPillButton("上一页", { page-- }, Modifier.weight(1f), enabled = page > 1)
                GhostPillButton("下一页", { page++ }, Modifier.weight(1f), enabled = result.hasMore)
            } }
        }
    }
}

@Composable
fun MyContentScreen(env: CommunityEnv, back: () -> Unit, open: (Route) -> Unit) {
    val session by env.session.state.collectAsState()
    Column(Modifier.fillMaxSize()) {
        ScreenBar("我的作品", onBack = back, trailing = { IconTap(XIcons.Upload, "发布内容包", { open(Route.PublishPack()) }) })
        if (!session.loggedIn) { CenterMessage("登录后查看自己的作品", "登录", { open(Route.Login) }); return@Column }
        val loader = rememberLoader(env, "my-packs-${session.accountId}") { env.content.myPacks() }
        val state by loader.state.collectAsState()
        val scope = rememberCoroutineScope()
        var removing by remember { mutableStateOf<Pack?>(null) }
        var busy by remember { mutableStateOf(false) }
        var error by remember { mutableStateOf<String?>(null) }
        Box(Modifier.weight(1f)) {
            LoadBox(state, loader::reload) { result ->
                if (result.items.isEmpty()) CenterMessage("还没有发布过内容包", "发布内容包", { open(Route.PublishPack()) })
                else LazyColumn(Modifier.fillMaxSize(), contentPadding = PaddingValues(12.dp), verticalArrangement = ContentSpacing) {
                    item { GhostPillButton("刷新作品状态", loader::reload, Modifier.fillMaxWidth()) }
                    items(result.items, key = { it.id }) { pack -> WhiteCard(padding = 14.dp) {
                        Text(pack.title, fontSize = 16.sp, fontWeight = FontWeight.SemiBold)
                        Text("${kindLabel(pack.kind)} · v${pack.version} · ${contentWorkState(pack)}", Modifier.padding(top = 6.dp), fontSize = 13.sp, color = LocalExtraColors.current.ink3)
                        Text("更新于 ${pack.updatedAt.take(10)}", Modifier.padding(top = 4.dp), fontSize = 12.sp, color = LocalExtraColors.current.ink3)
                        Row(Modifier.padding(top = 10.dp), horizontalArrangement = Arrangement.spacedBy(8.dp)) {
                            GhostPillButton("查看", { open(if (pack.kind in CapabilityPackKinds) Route.Ability(pack.id) else Route.Pack(pack.id)) }, Modifier.weight(1f))
                            if (pack.status != "removed") GhostPillButton("新版本", { open(Route.PublishPack(pack.id)) }, Modifier.weight(1f))
                            if (pack.status != "removed") GhostPillButton("下架", { removing = pack; error = null }, Modifier.weight(1f))
                        }
                    } }
                }
            }
            OverlayTransition(removing != null, direction = "center") {
                CenterDialog({ if (!busy) removing = null }) {
                    Text("下架「${removing?.title.orEmpty()}」？", fontSize = 18.sp, fontWeight = FontWeight.SemiBold)
                    Text("下架后不再出现在公开入口，已经应用过的内容不受影响。", Modifier.padding(top = 10.dp), fontSize = 14.sp, color = LocalExtraColors.current.ink3)
                    error?.let { Text(it, Modifier.padding(top = 8.dp), color = Danger, fontSize = 13.sp) }
                    Row(Modifier.padding(top = 16.dp), horizontalArrangement = Arrangement.spacedBy(8.dp)) {
                        GhostPillButton("取消", { removing = null }, Modifier.weight(1f), enabled = !busy)
                        PillButton(if (busy) "处理中…" else "确认下架", {
                            if (busy) return@PillButton
                            val id = removing?.id ?: return@PillButton
                            busy = true
                            scope.launch { try { env.content.removeOwn(id); removing = null; loader.reload() }
                                catch (e: CancellationException) { throw e } catch (e: Exception) { error = messageOf(e) }
                                finally { busy = false } }
                        }, Modifier.weight(1f), enabled = !busy)
                    }
                }
            }
        }
    }
}
@OptIn(ExperimentalComposeUiApi::class)
@Composable
fun PublishContentScreen(env: CommunityEnv, packId: Long?, back: () -> Unit, open: (Route) -> Unit) {
    val session by env.session.state.collectAsState()
    val scope = rememberCoroutineScope()
    var kind by rememberSaveable(packId, session.accountId) { mutableStateOf("effect") }
    var title by rememberSaveable(packId, session.accountId) { mutableStateOf("") }
    var summary by rememberSaveable(packId, session.accountId) { mutableStateOf("") }
    var tags by rememberSaveable(packId, session.accountId) { mutableStateOf("") }
    var changelog by rememberSaveable(packId, session.accountId) { mutableStateOf("") }
    var rights by rememberSaveable(packId, session.accountId) { mutableStateOf(false) }
    var fileKey by rememberSaveable(packId, session.accountId) { mutableStateOf("") }
    var file by remember(packId, session.accountId) { mutableStateOf<ForumUploadFile?>(null) }
    var busy by remember { mutableStateOf(false) }
    var step by remember { mutableStateOf("") }
    var error by remember { mutableStateOf<String?>(null) }
    var published by rememberSaveable(packId, session.accountId) { mutableStateOf<Long?>(null) }
    BackHandler(enabled = busy) { /* 提交中避免退出导致状态不明；系统选择器仍可自行取消。 */ }
    Column(Modifier.fillMaxSize().imePadding()) {
        ScreenBar(if (packId == null) "发布内容包" else "发布新版本", onBack = { if (!busy) back() })
        if (!session.loggedIn) { CenterMessage("登录后发布作品", "登录", { open(Route.Login) }); return@Column }
        val existingLoader = rememberLoader(env, "publish-original-$packId-${session.accountId}") { packId?.let { env.forum.pack(it) } }
        val existingState by existingLoader.state.collectAsState()
        val existing = (existingState as? Load.Ready)?.value
        val actualKind = existing?.kind ?: kind
        val versionReady = packId == null || (existing?.author?.uid == session.accountId && existing?.status != "removed")
        if (published != null) {
            CenterMessage("已提交发布\n审核与上架状态可在我的作品查看", "我的作品", { open(Route.MyWorks) })
            return@Column
        }
        Column(Modifier.weight(1f).fillMaxWidth().verticalScroll(rememberScrollState()).padding(16.dp), verticalArrangement = Arrangement.spacedBy(12.dp)) {
            if (packId == null) {
                Text("资源类型", fontSize = 13.sp, color = LocalExtraColors.current.ink3)
                PageChipRow(ContentPackLimits.keys.map { it to kindLabel(it) }, actualKind, { if (!busy) { kind = it; fileKey = "" } })
            } else {
                when (val state = existingState) {
                    Load.Loading -> Text("正在读取原作品…", fontSize = 14.sp)
                    is Load.Failed -> Banner(state.message, warn = true, actionText = "重试", onAction = existingLoader::reload)
                    is Load.Ready -> Text(existing?.title ?: "作品不可用", fontSize = 18.sp, fontWeight = FontWeight.SemiBold)
                }
                if (existing != null && !versionReady) Text("只有作者可以发布新版本", color = Danger, fontSize = 14.sp)
            }
            val platform = env.platform
            if (platform?.forumFilePickerSupported != true) {
                Text("当前设备请在网页版发布内容包。", fontSize = 14.sp, color = LocalExtraColors.current.ink3)
                GhostPillButton("打开网页版", { platform?.openUrl("${env.config.forumBase.trimEnd('/')}/publish${packId?.let { "?pack=$it" }.orEmpty()}") }, enabled = platform != null)
            } else {
                GhostPillButton(if (file == null) "选择内容包 .zip" else "更换内容包", {
                    if (busy) return@GhostPillButton
                    busy = true; error = null
                    scope.launch {
                        try {
                            val selected = platform.pickForumFile(false, ContentPackLimits[actualKind] ?: 0)
                            if (selected != null) { file = selected; fileKey = "" }
                        } catch (e: CancellationException) { throw e } catch (e: Exception) { error = (e as? ApiException)?.message ?: "无法读取文件，请重新选择" }
                        finally { busy = false }
                    }
                }, Modifier.fillMaxWidth(), enabled = !busy && versionReady)
                file?.let { Text("${it.name} · ${formatSize(it.bytes.size.toLong())}", fontSize = 13.sp) }
                Text("${kindLabel(actualKind)}上限 ${formatSize(ContentPackLimits[actualKind] ?: 0)}，ZIP 根目录需包含 manifest.json；由服务器检查内容清单。", fontSize = 12.sp, lineHeight = 18.sp, color = LocalExtraColors.current.ink3)
            }
            if (packId == null) {
                TextBox(title, { title = it }, placeholder = "标题（最多 60 字）", enabled = !busy)
                AreaBox(summary, { summary = it }, placeholder = "简介（最多 500 字）", minHeight = 120.dp, enabled = !busy)
                TextBox(tags, { tags = it }, placeholder = "标签，空格或逗号分隔（最多 8 个）", enabled = !busy)
            } else AreaBox(changelog, { changelog = it }, placeholder = "新版本更新说明（最多 500 字）", minHeight = 120.dp, enabled = !busy)
            Row(Modifier.fillMaxWidth(), verticalAlignment = androidx.compose.ui.Alignment.CenterVertically) {
                Checkbox(rights, { rights = it }, enabled = !busy)
                Text("我确认这是我的原创或已获授权，并同意按社区规则发布。", fontSize = 13.sp, lineHeight = 20.sp)
            }
            if (fileKey.isNotEmpty()) Text("文件已上传，可继续校验与发布。", fontSize = 12.sp, color = LocalExtraColors.current.ink3)
            Text("上传和导入不会执行内容。新版本会重新进入审核，实际状态以我的作品为准。", fontSize = 12.sp, lineHeight = 18.sp, color = LocalExtraColors.current.ink3)
            error?.let { Text(it, color = Danger, fontSize = 14.sp) }
            if (busy) Text(step.ifEmpty { "正在选择文件…" }, fontSize = 13.sp, color = LocalExtraColors.current.ink3)
            PillButton(if (busy) "处理中…" else if (fileKey.isEmpty()) "上传并提交发布" else "继续校验并发布", {
                if (busy) return@PillButton
                val draft = ContentPublishDraft(actualKind, title, summary, tags.split(Regex("[,，、\\s]+")).filter { it.isNotBlank() }, rights, changelog)
                val problem = contentDraftError(draft, packId != null)
                if (problem != null) { error = problem; return@PillButton }
                if (fileKey.isEmpty() && file == null) { error = "请选择内容包 .zip"; return@PillButton }
                busy = true; error = null
                scope.launch {
                    try {
                        if (fileKey.isEmpty()) {
                            step = "正在计算校验值并上传内容包…"
                            fileKey = env.content.upload(actualKind, file!!).fileKey
                        }
                        step = "正在等待资源服务器校验…"
                        if (!env.content.awaitVerified(fileKey)) {
                            error = "文件校验尚未完成，已保留上传记录和表单，请稍后点继续。"
                            return@launch
                        }
                        step = "正在提交发布…"
                        published = env.content.publish(draft, fileKey, packId).id
                        file = null
                    } catch (e: CancellationException) { throw e }
                    catch (e: Exception) { error = messageOf(e) }
                    finally { busy = false; step = "" }
                }
            }, Modifier.fillMaxWidth(), enabled = !busy && rights && versionReady && env.platform?.forumFilePickerSupported == true)
        }
    }
}
