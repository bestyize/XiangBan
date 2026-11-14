package xyz.xiangban.xiaoban.business.communitynative

import androidx.compose.foundation.background
import androidx.compose.foundation.clickable
import androidx.compose.foundation.horizontalScroll
import androidx.compose.foundation.layout.*
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.items
import androidx.compose.foundation.lazy.rememberLazyListState
import androidx.compose.foundation.rememberScrollState
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.*
import androidx.compose.runtime.saveable.rememberSaveable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.text.font.FontFamily
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import kotlinx.coroutines.CancellationException
import kotlinx.coroutines.launch
import xyz.xiangban.xiaoban.common.community.ReadmeBlock
import xyz.xiangban.xiaoban.common.community.RepoDetail
import xyz.xiangban.xiaoban.common.community.RepoRank
import xyz.xiangban.xiaoban.common.community.OpenSourceCatalog
import xyz.xiangban.xiaoban.common.community.Load
import xyz.xiangban.xiaoban.common.community.parseMarkdown
import xyz.xiangban.xiaoban.common.community.formatGain
import xyz.xiangban.xiaoban.common.community.formatStars
import xyz.xiangban.xiaoban.common.community.messageOf

/** 榜单项目名拆成 owner/repo；名称不合法时返回 null，调用方回退去 GitHub。 */
internal fun RepoRank.route(): Route.Repo? {
    val owner = fullName.substringBefore('/', "")
    val repo = fullName.substringAfter('/', "")
    return if (owner.isNotBlank() && repo.isNotBlank() && '/' !in repo) Route.Repo(owner, repo) else null
}

internal fun openRepo(env: CommunityEnv, r: RepoRank, open: (Route) -> Unit) {
    val route = r.route()
    if (route != null) open(route) else env.platform?.openUrl("https://github.com/${r.fullName}")
}

private val CodeBg = Color(0xFF1F2328)
private val CodeInk = Color(0xFFE6EDF3)
private const val COLLAPSED_BLOCKS = 14

@Composable
private fun ReadmeBlockView(b: ReadmeBlock) {
    val ink3 = LocalExtraColors.current.ink3
    when (b.t) {
        "h" -> Text(b.x, Modifier.padding(top = 14.dp, bottom = 2.dp), fontSize = if (b.l <= 2) 17.sp else 15.sp, lineHeight = 24.sp, fontWeight = FontWeight.Bold)
        "p" -> Text(b.x, fontSize = 14.sp, lineHeight = 24.sp)
        "quote" -> Row(Modifier.height(IntrinsicSize.Min)) {
            Box(Modifier.width(3.dp).fillMaxHeight().background(LocalExtraColors.current.hairline))
            Text(b.x, Modifier.padding(start = 10.dp), fontSize = 14.sp, lineHeight = 24.sp, color = MaterialTheme.colorScheme.onSurfaceVariant)
        }
        "li" -> Row(Modifier.padding(start = (b.d.coerceAtMost(4) * 16).dp), horizontalArrangement = Arrangement.spacedBy(8.dp)) {
            Text(if (b.n != null) "${b.n}." else "•", Modifier.widthIn(min = 14.dp), fontSize = 14.sp, lineHeight = 24.sp, color = ink3)
            Text(b.x, Modifier.weight(1f), fontSize = 14.sp, lineHeight = 24.sp)
        }
        "code" -> Box(Modifier.fillMaxWidth().clip(RoundedCornerShape(10.dp)).background(CodeBg).horizontalScroll(rememberScrollState()).padding(horizontal = 12.dp, vertical = 10.dp)) {
            Text(b.x, fontSize = 12.sp, lineHeight = 20.sp, fontFamily = FontFamily.Monospace, color = CodeInk, softWrap = false)
        }
        "table" -> Column(Modifier.fillMaxWidth().horizontalScroll(rememberScrollState())) {
            b.rows.forEachIndexed { r, row ->
                Row {
                    row.forEach { cell ->
                        Text(cell, Modifier.width(120.dp).background(if (r == 0) LocalExtraColors.current.fill else Color.Transparent).padding(horizontal = 8.dp, vertical = 6.dp), fontSize = 12.sp, lineHeight = 18.sp, fontWeight = if (r == 0) FontWeight.SemiBold else FontWeight.Normal)
                    }
                }
                Hairline()
            }
        }
        "hr" -> Hairline(Modifier.padding(vertical = 8.dp))
    }
}

@Composable
private fun LangToggle(zh: Boolean, zhEnabled: Boolean, onPick: (Boolean) -> Unit) {
    Row(Modifier.clip(RoundedCornerShape(10.dp)).background(LocalExtraColors.current.fill).padding(2.dp)) {
        listOf(true to "中文", false to "原文").forEach { (isZh, label) ->
            val on = zh == isZh
            val enabled = !isZh || zhEnabled
            Text(
                label,
                Modifier.clip(RoundedCornerShape(8.dp)).background(if (on) MaterialTheme.colorScheme.surface else Color.Transparent)
                    .clickable(enabled = enabled) { onPick(isZh) }.padding(horizontal = 14.dp, vertical = 6.dp),
                fontSize = 13.sp, fontWeight = if (on) FontWeight.SemiBold else FontWeight.Normal,
                color = if (!enabled) LocalExtraColors.current.ink3.copy(alpha = 0.5f) else if (on) MaterialTheme.colorScheme.onSurface else MaterialTheme.colorScheme.onSurfaceVariant,
            )
        }
    }
}

@Composable
fun RepoDetailScreen(env: CommunityEnv, owner: String, repo: String, back: () -> Unit, open: (Route) -> Unit, curated: Boolean = false) {
    val scope = rememberCoroutineScope()
    val session by env.session.state.collectAsState()
    val name = "$owner/$repo"
    val (fav, setFav) = rememberPrefFlag(env, "fav_repo:$name")
    var detail by remember(name, curated) { mutableStateOf<RepoDetail?>(null) }
    var curatedCatalog by remember(name, curated) { mutableStateOf<OpenSourceCatalog?>(null) }
    var error by remember(name) { mutableStateOf<String?>(null) }
    var loadingDetail by remember(name) { mutableStateOf(false) }
    var zh by rememberSaveable(name) { mutableStateOf(true) }
    var expanded by rememberSaveable(name) { mutableStateOf(false) }
    val discussion = rememberRepoComments(env, owner, repo, open)
    val listState = rememberLazyListState()

    val readmeLoader = if (curated) rememberLoader(env, "project-readme:$name") { env.forum.projectReadme(name) } else null
    val curatedReadme = readmeLoader?.state?.collectAsState()?.value
    val curatedProject = curatedCatalog?.project(name)
    val readme = (curatedReadme as? Load.Ready)?.value
    val curatedGithub = readme?.card?.githubUrl.orEmpty().ifBlank { curatedProject?.githubUrl.orEmpty() }
    val readmeBlocks = remember(readme, zh) {
        readme?.let { parseMarkdown(if (zh) it.zh else it.en) }.orEmpty()
    }

    fun loadDetail() {
        if (loadingDetail) return
        loadingDetail = true; error = null
        scope.launch {
            try {
                if (curated) {
                    val catalog = env.forum.openSourceProject(owner, repo)
                    val project = catalog.project(name) ?: error("这个项目尚未收录")
                    curatedCatalog = catalog
                    detail = RepoDetail(fullName = project.fullName)
                } else detail = env.forum.repoDetail(owner, repo)
            }
            catch (e: CancellationException) { throw e }
            catch (e: Exception) { error = messageOf(e) }
            finally { loadingDetail = false }
        }
    }
    LaunchedEffect(name, curated) { loadDetail() }

    CommentScaffold(env, discussion, Modifier.background(MaterialTheme.colorScheme.surface)) {
        ScreenBar(if (curated) "开源精选" else "仓库详情", onBack = back, trailing = {
            webPageRoute(curatedGithub)?.let { github ->
                IconTap(XIcons.ExternalLink, "在 GitHub 查看项目", { open(github) })
            }
        })
        val d = detail
        if (d == null) {
            Box(Modifier.weight(1f)) {
                if (error != null) CenterMessage("项目加载失败：$error", "重试", ::loadDetail, Modifier.fillMaxSize())
                else Spinner(Modifier.fillMaxSize())
            }
        } else {
            val showZh = zh && d.hasZhReadme
            val blocks = (if (showZh) d.readme.zhBlocks else d.readme.blocks).orEmpty()
            val long = blocks.size > COLLAPSED_BLOCKS
            val shown = if (long && !expanded) blocks.take(COLLAPSED_BLOCKS) else blocks
            val ink3 = LocalExtraColors.current.ink3
            val hasReadme = d.readme.status == "ok" && blocks.isNotEmpty()
            val note = when {
                !hasReadme -> null
                showZh -> "AI 翻译，可能有误，以原文为准。" + if (d.readme.zhTruncated) " 译文只覆盖前半部分，完整内容请看原文。" else ""
                zh && !d.hasZhReadme -> "中文翻译暂未提供，已显示英文原文。"
                else -> null
            }
            // 精选页在评论前有项目卡、README 栏、正文块和来源；缺省或失败时只显示一条状态。
            val commentsIndex = if (curated) {
                when {
                    curatedProject == null || curatedReadme == null -> 0
                    readme == null -> 3
                    else -> 3 + readmeBlocks.size
                }
            } else {
                2 + if (hasReadme) (if (note != null) 1 else 0) + shown.size + (if (long) 1 else 0) else 1
            }
            LazyColumn(Modifier.weight(1f), state = listState, contentPadding = PaddingValues(bottom = 24.dp)) {
                if (curated) {
                    val catalog = curatedCatalog
                    if (catalog != null && curatedProject != null && curatedReadme != null && readmeLoader != null) {
                        curatedReadmeItems(env, catalog, curatedProject, curatedReadme, readmeBlocks, zh, { zh = it }, readmeLoader::reload, open)
                    }
                } else {
                item(key = "hero") {
                    Column(Modifier.fillMaxWidth().padding(horizontal = 20.dp, vertical = 20.dp)) {
                        val asRank = RepoRank(rank = d.rank, fullName = d.fullName, language = d.language, languageColor = d.languageColor)
                        Row(verticalAlignment = Alignment.CenterVertically, horizontalArrangement = Arrangement.spacedBy(12.dp)) {
                            RepoTile(asRank, 44.dp, 12.dp)
                            Column(Modifier.weight(1f)) {
                                Text("${d.owner.ifBlank { owner }} /", fontSize = 13.sp, color = ink3)
                                Text(d.repo.ifBlank { repo }, fontSize = 22.sp, lineHeight = 28.sp, fontWeight = FontWeight.Bold, maxLines = 2, overflow = TextOverflow.Ellipsis)
                            }
                            Text(
                                if (d.isLatest) "本周 No.${d.rank}" else "已掉出本周榜",
                                Modifier.clip(RoundedCornerShape(999.dp)).background(if (d.isLatest) DangerSoft else LocalExtraColors.current.fill).padding(horizontal = 10.dp, vertical = 4.dp),
                                fontSize = 12.sp, fontWeight = FontWeight.SemiBold, color = if (d.isLatest) Brand else ink3,
                            )
                        }
                        val summary = d.descriptionZh.ifBlank { d.description }
                        if (summary.isNotBlank()) Text(summary, Modifier.padding(top = 14.dp), fontSize = 15.sp, lineHeight = 25.sp)
                        if (d.descriptionZh.isNotBlank() && d.description.isNotBlank() && d.descriptionZh != d.description) {
                            Text("原文：${d.description}", Modifier.padding(top = 6.dp), fontSize = 12.sp, lineHeight = 18.sp, color = ink3)
                        }
                        Hairline(Modifier.padding(top = 16.dp))
                        Row(Modifier.fillMaxWidth().padding(top = 16.dp), verticalAlignment = Alignment.Bottom) {
                            Column(Modifier.weight(1f)) {
                                Text(d.starGain?.let(::formatGain) ?: "—", fontSize = 32.sp, lineHeight = 34.sp, fontWeight = FontWeight.Bold, color = Brand)
                                Text("近 7 天新增 Star", Modifier.padding(top = 6.dp), fontSize = 12.sp, color = MaterialTheme.colorScheme.onSurfaceVariant)
                            }
                            Column(horizontalAlignment = Alignment.End) {
                                Text(formatStars(d.starTotal), fontSize = 15.sp, fontWeight = FontWeight.SemiBold)
                                Text("总 Star", fontSize = 12.sp, color = ink3)
                            }
                            if (d.language.isNotBlank()) Column(Modifier.padding(start = 20.dp), horizontalAlignment = Alignment.End) {
                                Row(verticalAlignment = Alignment.CenterVertically, horizontalArrangement = Arrangement.spacedBy(5.dp)) {
                                    Dot(hexColor(d.languageColor, DataBlue), 8.dp)
                                    Text(d.language, fontSize = 15.sp, fontWeight = FontWeight.SemiBold)
                                }
                                Text("主要语言", fontSize = 12.sp, color = ink3)
                            }
                        }
                        Row(Modifier.fillMaxWidth().padding(top = 16.dp), horizontalArrangement = Arrangement.spacedBy(8.dp)) {
                            Text(
                                if (fav) "已收藏" else "收藏",
                                Modifier.weight(1f).clip(RoundedCornerShape(12.dp)).background(if (fav) DangerSoft else Brand).clickable { setFav(!fav) }.padding(vertical = 11.dp),
                                fontSize = 14.sp, fontWeight = FontWeight.SemiBold, color = if (fav) Brand else Color.White, textAlign = androidx.compose.ui.text.style.TextAlign.Center,
                            )
                            Text(
                                "在 GitHub 查看 ↗",
                                Modifier.weight(1f).clip(RoundedCornerShape(12.dp)).background(LocalExtraColors.current.fill).clickable { webPageRoute(d.url.ifBlank { "https://github.com/$name" })?.let(open) }.padding(vertical = 11.dp),
                                fontSize = 14.sp, fontWeight = FontWeight.SemiBold, textAlign = androidx.compose.ui.text.style.TextAlign.Center,
                            )
                        }
                        Row(Modifier.padding(top = 12.dp), horizontalArrangement = Arrangement.spacedBy(4.dp)) {
                            Text("数据来自 GitHub Trending 周榜 · ", fontSize = 12.sp, color = ink3)
                            Text("跳到评论", Modifier.clickable { scope.launch { listState.animateScrollToItem(commentsIndex) } }, fontSize = 12.sp, color = Brand)
                        }
                    }
                    Hairline()
                }
                item(key = "intro-head") {
                    Row(Modifier.fillMaxWidth().padding(start = 20.dp, end = 20.dp, top = 20.dp, bottom = 12.dp), verticalAlignment = Alignment.CenterVertically) {
                        Text("项目介绍", Modifier.weight(1f), fontSize = 17.sp, fontWeight = FontWeight.Bold)
                        if (d.readme.status == "ok" && blocks.isNotEmpty()) LangToggle(showZh, d.hasZhReadme) { zh = it; expanded = false }
                    }
                }
                if (hasReadme) {
                    if (note != null) item(key = "note") {
                        Text(note, Modifier.padding(horizontal = 20.dp).padding(bottom = 12.dp).fillMaxWidth().clip(RoundedCornerShape(10.dp)).background(LocalExtraColors.current.fill).padding(horizontal = 10.dp, vertical = 8.dp), fontSize = 12.sp, lineHeight = 18.sp, color = MaterialTheme.colorScheme.onSurfaceVariant)
                    }
                    items(shown.size, key = { "b$it" }) { i ->
                        Box(Modifier.fillMaxWidth().padding(horizontal = 20.dp).padding(bottom = 8.dp)) { ReadmeBlockView(shown[i]) }
                    }
                    if (long) item(key = "more") {
                        Text(
                            if (expanded) "收起" else "展开全文",
                            Modifier.padding(horizontal = 20.dp, vertical = 4.dp).fillMaxWidth().clip(RoundedCornerShape(12.dp)).background(LocalExtraColors.current.fill).clickable { expanded = !expanded }.padding(vertical = 11.dp),
                            fontSize = 14.sp, fontWeight = FontWeight.SemiBold, textAlign = androidx.compose.ui.text.style.TextAlign.Center,
                        )
                    }
                } else item(key = "no-readme") {
                    Column(Modifier.fillMaxWidth().padding(horizontal = 20.dp, vertical = 16.dp), horizontalAlignment = Alignment.CenterHorizontally) {
                        Text(if (d.readme.status == "none") "这个仓库没有 README" else "暂时无法获取 README", fontSize = 15.sp, fontWeight = FontWeight.SemiBold)
                        Text(if (d.readme.status == "none") "作者还没有写项目介绍，可以去 GitHub 查看代码和文档。" else "稍后再来看看，或直接去 GitHub 查看。", Modifier.padding(top = 4.dp), fontSize = 13.sp, lineHeight = 20.sp, color = MaterialTheme.colorScheme.onSurfaceVariant, textAlign = androidx.compose.ui.text.style.TextAlign.Center)
                    }
                }
                }
                plainCommentItems(env, discussion, open)
                val related = curatedCatalog?.related.orEmpty()
                if (related.isNotEmpty()) item(key = "curated-related") {
                    Column(Modifier.fillMaxWidth().padding(16.dp)) {
                        Text("还可以看看", style = XType.Section, fontWeight = FontWeight.SemiBold)
                        related.forEach { project ->
                            Column(Modifier.fillMaxWidth().clickable {
                                open(Route.OpenSource(project.fullName.substringBefore('/'), project.fullName.substringAfter('/')))
                            }.padding(vertical = 10.dp)) {
                                Text(project.name, style = XType.Body, fontWeight = FontWeight.SemiBold)
                                Text(project.summary, style = XType.Caption, color = ink3)
                            }
                        }
                    }
                }
            }
            CommentBar(if (session.loggedIn) "写评论…" else "登录后评论", onWrite = { discussion.begin() }) {
                if (curated) {
                    webPageRoute(curatedGithub)?.let { github ->
                        IconTap(XIcons.ExternalLink, "在 GitHub 查看项目", { open(github) })
                    }
                    IconTap(XIcons.Star, if (fav) "取消收藏" else "收藏", { setFav(!fav) },
                        tint = if (fav) Brand else MaterialTheme.colorScheme.onSurface)
                }
                CommentCountAction(discussion.total) { scope.launch { listState.animateScrollToItem(commentsIndex) } }
            }
        }
    }
}
