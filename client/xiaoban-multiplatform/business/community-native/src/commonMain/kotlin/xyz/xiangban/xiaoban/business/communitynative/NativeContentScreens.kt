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
import kotlinx.coroutines.CancellationException
import kotlinx.coroutines.launch
import kotlinx.serialization.encodeToString
import kotlinx.serialization.json.JsonObject
import xyz.xiangban.xiaoban.common.community.*

private val ContentSpacing = Arrangement.spacedBy(8.dp)

@Composable
private fun ManifestSection(title: String, value: String?, collapsible: Boolean = false) {
    WhiteCard(padding = 14.dp) {
        Text(title, style = XType.Body, fontWeight = FontWeight.SemiBold)
        if (collapsible && value != null) ExpandableDescription(value)
        else SelectionContainer { Text(value ?: "发布者未提供此项说明", Modifier.padding(top = 8.dp), style = XType.Secondary, color = MaterialTheme.colorScheme.onSurfaceVariant) }
    }
}

@Composable
fun AbilityDetailScreen(env: CommunityEnv, id: Long, back: () -> Unit, open: ((Route) -> Unit)? = null) = CommunityDetail {
    val loader = rememberLoader(env, "ability-$id") { env.forum.pack(id) }
    val state by loader.state.collectAsState()
    Column(Modifier.fillMaxSize()) {
        if (state !is Load.Ready<*>) ScreenBar("能力详情", onBack = back)
        Box(Modifier.weight(1f)) { LoadBox(state, loader::reload) { pack -> CapabilityDetailContent(env, pack, back, open = open) } }
    }
}

/** 能力详情同时服务于能力和通用作品入口。 */
@Composable
internal fun CapabilityDetailContent(env: CommunityEnv, pack: Pack, back: () -> Unit, open: ((Route) -> Unit)? = null) {
    var sharing by rememberSaveable(pack.id) { mutableStateOf(false) }
    val manifest = pack.versions.firstOrNull { it.version == pack.version }?.manifest ?: JsonObject(emptyMap())
    val prompt = manifestValue(manifest, "prompt", "promptText", "systemPrompt", "system_prompt")
    val manualOnly = manifest["manualOnly"]?.toString() == "true"
    Column(Modifier.fillMaxSize()) {
        val author = pack.author
        DetailAuthorBar(author?.nickname ?: "社区作品", back, onShare = { sharing = true },
            onAuthor = if (author != null && open != null) ({ open(Route.User(author.uid)) }) else null) {
            author?.let { DetailAuthorFollow(env, it.uid, open) }
        }
        WorkInteractionLayout(env, "pack", pack.id, open, { sharing = true }, enabled = pack.status == "published",
            linkedTopicId = pack.topicId) { comments ->
            Column(Modifier.fillMaxSize().verticalScroll(rememberScrollState())) {
                DetailGallery(listOfNotNull(pack.coverUrl?.takeIf(String::isNotBlank)?.let(env.forum::imageUrl)), pack.title)
                DetailBody {
                    DetailHeading(pack.title)
                    if (pack.summary.isNotBlank()) Text(pack.summary, style = XType.DetailBody)
                    DetailTags(pack.tags)
                    if (manualOnly) BlueTag("手动模板 · 不自动执行", Modifier.padding(top = 8.dp))
                    DetailMetadata("${kindLabel(pack.kind)} · v${pack.version}")
                    if (pack.status != "published") Text(contentWorkState(pack), Modifier.padding(top = 8.dp), color = Danger, style = XType.Secondary)
                }
                Column(Modifier.fillMaxWidth().padding(horizontal = 16.dp), verticalArrangement = ContentSpacing) {
                    if (pack.status == "published") NativeReportAction(env, "pack", pack.id, open)
                    ManifestSection("能做", manifestValue(manifest, "canDo", "can_do", "capabilities"))
                    ManifestSection("不能做", manifestValue(manifest, "cannotDo", "cannot_do", "limitations"))
                    ManifestSection("所需权限", manifestValue(manifest, "permissions", "requiredPermissions"))
                    ManifestSection("运行位置", manifestValue(manifest, "runLocation", "run_location", "runtime", "executionLocation"))
                    ManifestSection("完整提示词", prompt, collapsible = true)
                    if (prompt != null) GhostPillButton("复制提示词", { env.platform?.copy(prompt) }, Modifier.fillMaxWidth(), enabled = env.platform != null)
                    var expanded by rememberSaveable { mutableStateOf(false) }
                    GhostPillButton(if (expanded) "收起完整清单" else "查看完整清单", { expanded = !expanded }, Modifier.fillMaxWidth())
                    if (expanded) ManifestSection("资源清单原文", CommunityJson.encodeToString(manifest))
                    FootNote(if (manualOnly) "这是可下载的手动模板，按说明使用或复制提示词；不会自动执行任务。" else "手机用于浏览和分享。实际运行与授权在电脑端确认，导入不会自动执行；未提供权限说明不代表无需权限。")
                    GhostPillButton("复制电脑端打开链接", { env.platform?.copy(env.forum.packUrl(pack.id)) }, Modifier.fillMaxWidth(), enabled = env.platform != null && pack.status == "published")
                }
                comments()
            }
        }
    }
    OverlayTransition(sharing) { panel ->
        CommunityShareSheet(
            env = env,
            url = env.forum.packUrl(pack.id),
            heading = "分享能力",
            onDismiss = { sharing = false },
            panelModifier = panel,
            author = pack.author?.nickname.orEmpty(),
            dateLabel = pack.updatedAt.take(10),
            coverUrl = pack.coverUrl?.takeIf(String::isNotBlank)?.let(env.forum::imageUrl),
        )
    }
}

@Composable
fun SchemeDetailScreen(env: CommunityEnv, id: Long, back: () -> Unit, open: (Route) -> Unit) = CommunityDetail {
    var sharing by rememberSaveable(id) { mutableStateOf(false) }
    val loader = rememberLoader(env, "scheme-$id") { env.content.scheme(id) }
    val state by loader.state.collectAsState()
    Column(Modifier.fillMaxSize()) {
        if (state !is Load.Ready<*>) ScreenBar("桌面方案", onBack = back)
        LoadBox(state, loader::reload) { scheme ->
            val author = scheme.author
            DetailAuthorBar(author.nickname ?: "社区作者", back, onShare = { sharing = true }, onAuthor = { open(Route.User(author.uid)) }) {
                DetailAuthorFollow(env, author.uid, open)
            }
            WorkInteractionLayout(env, "scheme", scheme.id, open, { sharing = true }) { comments ->
                LazyColumn(Modifier.fillMaxSize(), contentPadding = PaddingValues(bottom = 16.dp), verticalArrangement = ContentSpacing) {
                    item { DetailBody {
                        DetailHeading(scheme.title)
                        if (scheme.summary.isNotBlank()) Text(scheme.summary, style = XType.DetailBody)
                        DetailTags(listOf("社区方案"))
                        DetailMetadata("${scheme.createdAt.take(10)} 发布 · ${scheme.items.size} 项作品")
                    } }
                    item { Box(Modifier.padding(horizontal = 16.dp)) { NativeReportAction(env, "scheme", scheme.id, open) } }
                    item { Text("包含的内容", Modifier.padding(horizontal = 16.dp), style = XType.Section, fontWeight = FontWeight.SemiBold) }
                    itemsIndexed(scheme.items, key = { index, item -> "$index-${item.packId}-${item.version}" }) { _, item -> WhiteCard(Modifier.padding(horizontal = 16.dp), padding = 14.dp) {
                        Text(if (item.available) item.title ?: item.slug else "内容不可用", style = XType.Body, fontWeight = FontWeight.Medium)
                        Text(if (item.available) "${kindLabel(item.kind)} · 方案版本 v${item.version}" else "此项已下架或当前不可访问", Modifier.padding(top = 6.dp), style = XType.Caption, color = LocalExtraColors.current.ink3)
                        if (item.available) GhostPillButton("查看内容", { open(if (item.kind in CapabilityPackKinds) Route.Ability(item.packId) else Route.Pack(item.packId)) }, Modifier.padding(top = 8.dp))
                    } }
                    item { Box(Modifier.padding(horizontal = 16.dp)) { FootNote("方案是一份内容清单。手机不会应用桌面内容；在电脑上打开后，逐项检查版本、可用性和权限再确认。") } }
                    item { comments() }
                }
            }
        }
    }
    OverlayTransition(sharing) { panel ->
        val scheme = (state as? Load.Ready)?.value
        CommunityShareSheet(
            env = env,
            url = "${env.config.forumBase.trimEnd('/')}/schemes/$id",
            heading = "分享方案",
            onDismiss = { sharing = false },
            panelModifier = panel,
            author = scheme?.author?.nickname.orEmpty(),
            dateLabel = scheme?.createdAt.orEmpty().take(10),
        )
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
                Text(scheme.title, style = XType.Section, fontWeight = FontWeight.SemiBold)
                Text("${scheme.items.size} 项内容 · ${scheme.author.nickname ?: "作者"}", Modifier.padding(top = 8.dp), style = XType.Caption, color = LocalExtraColors.current.ink3)
            } }
            item { Row(Modifier.fillMaxWidth(), horizontalArrangement = Arrangement.spacedBy(8.dp)) {
                GhostPillButton("上一页", { page-- }, Modifier.weight(1f), enabled = page > 1)
                GhostPillButton("下一页", { page++ }, Modifier.weight(1f), enabled = result.hasMore)
            } }
        }
    }
}

@OptIn(ExperimentalComposeUiApi::class)
@Composable
fun PublishContentScreen(
    env: CommunityEnv, packId: Long?, back: () -> Unit, open: (Route) -> Unit, localGameId: String? = null,
    onPublished: ((Long, String) -> Unit)? = null,
) {
    val session by env.session.state.collectAsState()
    val accountGeneration by rememberUpdatedState(session.generation)
    val scope = rememberCoroutineScope()
    val forPost = onPublished != null
    var kind by rememberSaveable(packId, session.accountId) { mutableStateOf(if (localGameId != null) "game" else "effect") }
    var title by rememberSaveable(packId, session.accountId) { mutableStateOf("") }
    var summary by rememberSaveable(packId, session.accountId) { mutableStateOf("") }
    var tags by rememberSaveable(packId, session.accountId) { mutableStateOf("") }
    var changelog by rememberSaveable(packId, session.accountId) { mutableStateOf("") }
    var rights by rememberSaveable(packId, session.accountId) { mutableStateOf(false) }
    var fileKey by rememberSaveable(packId, session.accountId) { mutableStateOf("") }
    var file by remember(packId, session.accountId) { mutableStateOf<ForumUploadFile?>(null) }
    var selectedName by remember(session.accountId) { mutableStateOf("") }
    var suggestedTitle by rememberSaveable(session.accountId) { mutableStateOf("") }
    var suggestedSummary by rememberSaveable(session.accountId) { mutableStateOf("") }
    var busy by remember { mutableStateOf(false) }
    var step by remember { mutableStateOf("") }
    var error by remember { mutableStateOf<String?>(null) }
    var published by rememberSaveable(packId, session.accountId) { mutableStateOf<Long?>(null) }
    var importedGame by rememberSaveable(localGameId, session.accountId) { mutableStateOf(false) }
    LaunchedEffect(localGameId, session.accountId) {
        if (localGameId != null && session.loggedIn && file == null && fileKey.isEmpty()) {
            busy = true; step = "正在读取本地作品…"
            try {
                val game = env.games?.read(localGameId) ?: error("本地作品不可用")
                file = env.platform?.gamePackage(localGameId) ?: error("当前平台不支持作品打包")
                if (!importedGame) { kind = game.kind; title = game.name; summary = game.description; importedGame = true }
            } catch (e: CancellationException) { throw e }
            catch (e: Exception) { error = e.message ?: "读取本地作品失败" }
            finally { busy = false; step = "" }
        }
    }
    BackHandler(enabled = busy || forPost) { if (!busy) back() }
    Column(Modifier.fillMaxSize().imePadding()) {
        val heading = if (forPost) "上传作品" else if (packId == null) "发布内容包" else "发布新版本"
        ScreenBar(heading, onBack = { if (!busy) back() })
        if (!session.loggedIn) { CenterMessage("登录后发布作品", "登录", { open(Route.Login) }); return@Column }
        val platform = env.platform
        val canPick = platform != null && if (forPost) platform.forumWorkPickerSupported else platform.forumFilePickerSupported
        val webPublisher = "${env.config.forumBase.trimEnd('/')}/publish${packId?.let { "?pack=$it" }.orEmpty()}"
        if (forPost && !canPick) {
            CenterMessage("在网页版发布后，回到发帖页从「作品」中选取。", "打开网页版", { platform?.openUrl(webPublisher) })
            return@Column
        }
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
            if (forPost && (file != null || fileKey.isNotEmpty())) {
                Text(kindLabel(actualKind), style = XType.Secondary, color = LocalExtraColors.current.ink3)
            } else if (packId == null && !forPost) {
                Text("资源类型", style = XType.Secondary, color = LocalExtraColors.current.ink3)
                if (localGameId != null) Text("${kindLabel(actualKind)} · 本机作品", style = XType.Body, fontWeight = FontWeight.Medium)
                else PageChipRow(ContentPackLimits.keys.map { it to kindLabel(it) }, actualKind, { if (!busy) { kind = it; fileKey = "" } })
            } else if (packId != null) {
                when (val state = existingState) {
                    Load.Loading -> Text("正在读取原作品…", style = XType.Body)
                    is Load.Failed -> Banner(state.message, warn = true, actionText = "重试", onAction = existingLoader::reload)
                    is Load.Ready -> Text(existing?.title ?: "作品不可用", style = XType.Title, fontWeight = FontWeight.SemiBold)
                }
                if (existing != null && !versionReady) Text("只有作者可以发布新版本", color = Danger, style = XType.Body)
            }
            if (!canPick) {
                Text("当前设备请在网页版发布内容包。", style = XType.Secondary, color = LocalExtraColors.current.ink3)
                GhostPillButton("打开网页版", { platform?.openUrl(webPublisher) }, enabled = platform != null)
            } else {
                val pickLabel = if (forPost) {
                    if (file == null && fileKey.isEmpty()) "选择作品文件" else "更换作品文件"
                } else if (file == null) "选择内容包 .zip" else "更换内容包"
                GhostPillButton(pickLabel, {
                    if (busy) return@GhostPillButton
                    val generation = session.generation
                    busy = true; error = null
                    scope.launch {
                        try {
                            if (forPost) {
                                val prepared = platform?.pickForumWork() ?: return@launch
                                check(accountGeneration == generation) { "账号已切换，请重新选择" }
                                kind = prepared.kind
                                selectedName = prepared.originalName
                                if (title.isBlank() || title == suggestedTitle) title = prepared.title
                                if (summary.isBlank() || summary == suggestedSummary) summary = prepared.summary
                                suggestedTitle = prepared.title
                                suggestedSummary = prepared.summary
                                file = prepared.file
                            } else {
                                val selected = platform?.pickForumFile(false, ContentPackLimits[actualKind] ?: 0) ?: return@launch
                                check(accountGeneration == generation) { "账号已切换，请重新选择" }
                                file = selected
                            }
                            fileKey = ""
                        } catch (e: CancellationException) { throw e }
                        catch (e: Exception) {
                            error = when (e) {
                                is ApiException, is IllegalArgumentException -> e.message
                                else -> null
                            } ?: "无法读取文件，请重新选择"
                        }
                        finally { busy = false }
                    }
                }, Modifier.fillMaxWidth(), enabled = !busy && versionReady)
                val chosenFile = file
                if (chosenFile != null) {
                    val name = selectedName.ifBlank { chosenFile.name }
                    Text("$name · ${formatSize(chosenFile.bytes.size.toLong())}", style = XType.Secondary)
                }
                val formats = if (forPost) "支持 ZIP、HTML 和 HTM。ZIP 根目录需包含 manifest.json。"
                    else "${kindLabel(actualKind)}上限 ${formatSize(ContentPackLimits[actualKind] ?: 0)}，ZIP 根目录需包含 manifest.json；由服务器检查内容清单。"
                Text(formats, style = XType.Caption, color = LocalExtraColors.current.ink3)
            }
            if (packId == null) {
                TextBox(title, { title = it }, placeholder = "标题（最多 60 字）", enabled = !busy)
                AreaBox(summary, { summary = it }, placeholder = "简介（最多 500 字）", minHeight = 120.dp, enabled = !busy)
                TextBox(tags, { tags = it }, placeholder = "标签，空格或逗号分隔（最多 8 个）", enabled = !busy)
            } else AreaBox(changelog, { changelog = it }, placeholder = "新版本更新说明（最多 500 字）", minHeight = 120.dp, enabled = !busy)
            Row(Modifier.fillMaxWidth(), verticalAlignment = androidx.compose.ui.Alignment.CenterVertically) {
                Checkbox(rights, { rights = it }, enabled = !busy)
                Text("我确认这是我的原创或已获授权，并同意按社区规则发布。", style = XType.Secondary)
            }
            if (fileKey.isNotEmpty()) Text("文件已上传，可继续校验与发布。", style = XType.Caption, color = LocalExtraColors.current.ink3)
            Text(if (forPost) "作品发布成功后会添加到当前帖子草稿。" else "上传和导入不会执行内容。发布后会出现在社区作品里。",
                style = XType.Caption, color = LocalExtraColors.current.ink3)
            error?.let { Text(it, color = Danger, style = XType.Body) }
            if (busy) Text(step.ifEmpty { "正在选择文件…" }, style = XType.Secondary, color = LocalExtraColors.current.ink3)
            PillButton(if (busy) "处理中…" else if (fileKey.isEmpty()) "上传并提交发布" else "继续校验并发布", {
                if (busy) return@PillButton
                val draft = ContentPublishDraft(actualKind, title, summary, tags.split(Regex("[,，、\\s]+")).filter { it.isNotBlank() }, rights, changelog)
                val problem = contentDraftError(draft, packId != null)
                if (problem != null) { error = problem; return@PillButton }
                if (fileKey.isEmpty() && file == null) {
                    error = if (forPost) "请选择作品文件" else "请选择内容包 .zip"
                    return@PillButton
                }
                val generation = session.generation
                busy = true; error = null
                scope.launch {
                    try {
                        if (fileKey.isEmpty()) {
                            step = "正在计算校验值并上传内容包…"
                            fileKey = env.content.upload(actualKind, file!!).fileKey
                        }
                        check(accountGeneration == generation) { "账号已切换，请重新选择文件" }
                        step = "正在等待资源服务器校验…"
                        if (!env.content.awaitVerified(fileKey, recoverPublished = packId == null)) {
                            error = "文件校验尚未完成，已保留上传记录和表单，请稍后点继续。"
                            return@launch
                        }
                        step = "正在提交发布…"
                        check(accountGeneration == generation) { "账号已切换，请重新选择文件" }
                        val result = env.content.publish(draft, fileKey, packId)
                        check(accountGeneration == generation) { "账号已切换，请在我的作品核对发布结果" }
                        file = null
                        if (onPublished != null) onPublished(result.id, draft.title.trim()) else published = result.id
                    } catch (e: CancellationException) { throw e }
                    catch (e: Exception) { error = messageOf(e) }
                    finally { busy = false; step = "" }
                }
            }, Modifier.fillMaxWidth(), enabled = !busy && rights && versionReady && canPick)
        }
    }
}
