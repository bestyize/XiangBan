package xyz.xiangban.xiaoban.business.communitynative

import androidx.compose.foundation.background
import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.*
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.LazyListScope
import androidx.compose.foundation.lazy.items
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.material3.TextButton
import androidx.compose.runtime.*
import androidx.compose.runtime.saveable.rememberSaveable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.layout.ContentScale
import androidx.compose.ui.text.font.FontWeight
import coil3.compose.AsyncImage
import xyz.xiangban.xiaoban.common.community.Load
import xyz.xiangban.xiaoban.common.community.MdBlock
import xyz.xiangban.xiaoban.common.community.ProjectReadme
import xyz.xiangban.xiaoban.common.community.ProjectReadmeCard
import xyz.xiangban.xiaoban.common.community.OpenSourceCatalog
import xyz.xiangban.xiaoban.common.community.OpenSourceCollection
import xyz.xiangban.xiaoban.common.community.OpenSourceProject
import androidx.compose.ui.unit.dp
import kotlin.time.Duration.Companion.hours
import kotlin.time.Instant
import xyz.xiangban.xiaoban.common.community.formatStars

private fun OpenSourceProject.route() = Route.OpenSource(fullName.substringBefore('/'), fullName.substringAfter('/'))

/** 真实图片加载失败时收起，不制造渐变封面。 */
@Composable
private fun OpenSourceImage(env: CommunityEnv, project: OpenSourceProject) {
    var failed by remember(project.coverUrl) { mutableStateOf(false) }
    if (project.coverUrl.isNotBlank() && !failed) {
        AsyncImage(
            model = env.forum.imageUrl(project.coverUrl), imageLoader = rememberImageLoader(), contentDescription = null,
            contentScale = ContentScale.Fit, onError = { failed = true },
            modifier = Modifier.fillMaxWidth().aspectRatio(16f / 9f).background(LocalExtraColors.current.fill),
        )
    }
}

@Composable
internal fun OpenSourceTab(env: CommunityEnv, open: (Route) -> Unit, hotRankingOpenCount: Int) {
    var category by key(hotRankingOpenCount) {
        rememberSaveable { mutableStateOf(if (hotRankingOpenCount == 0) "recommend" else "hot") }
    }
    var tag by rememberSaveable { mutableStateOf("") }
    val loader = rememberLoader(env, "open-source") { env.forum.openSourceCatalog() }
    val state by loader.state.collectAsState()
    val categoryTabs = (state as? Load.Ready<OpenSourceCatalog>)?.value?.categories.orEmpty().map { it.key to it.label }
    Column(Modifier.fillMaxSize()) {
        TabStrip(listOf("recommend" to "推荐", "hot" to "热榜") + categoryTabs, category, centered = true) { category = it; tag = "" }
        if (category == "hot") {
            key(hotRankingOpenCount) { HotTab(env, open) }
        } else LoadBox(state, loader::reload) { catalog ->
        val projects = remember(catalog, category, tag) {
            catalog.filter(category, tag = tag)
        }
        val tags = remember(catalog, category) { catalog.filter(category).flatMap { it.tags }.distinct().take(8) }
        val filtered = tag.isNotEmpty()
        Column(Modifier.fillMaxSize()) {
            if (category != "recommend") PageChipRow(listOf("" to "全部") + tags.map { it to it }, tag, { tag = it }, Modifier.padding(horizontal = 12.dp).padding(bottom = 8.dp))
            LazyColumn(Modifier.fillMaxSize(), contentPadding = PaddingValues(start = 12.dp, end = 12.dp, top = 8.dp, bottom = 24.dp + NavInset), verticalArrangement = Arrangement.spacedBy(12.dp)) {
                item(key = "heading") {
                    Row(Modifier.fillMaxWidth().padding(bottom = 4.dp), verticalAlignment = Alignment.CenterVertically) {
                        Column(Modifier.weight(1f)) {
                            Text("${projects.size} 个项目", Modifier.padding(top = 4.dp), style = XType.Caption, color = LocalExtraColors.current.ink3)
                        }
                    }
                }
                if (projects.isEmpty()) item(key = "empty") {
                    WhiteCard {
                        Text("这个分类的项目正在整理中", style = XType.Section)
                        Text("可以先看看其他分类。", style = XType.Secondary, color = LocalExtraColors.current.ink3, modifier = Modifier.padding(top = 8.dp))
                        GhostPillButton("浏览项目", { tag = "" }, Modifier.fillMaxWidth().padding(top = 16.dp))
                    }
                }
                projects.forEachIndexed { index, project ->
                    item(key = project.fullName) { OpenSourceCard(env, project, open) }
                    if (category == "recommend" && !filtered && index in listOf(3, 7)) {
                        val collection = catalog.collections.getOrNull(if (index == 3) 0 else 1)
                        if (collection != null) item(key = "collection:${collection.key}") { OpenSourceCollectionCard(collection, open) }
                    }
                }
                item(key = "source") { FootNote("根据项目公开资料整理 · ${catalog.reviewedAt}") }
            }
        }
    }
}
}

@Composable
internal fun OpenSourceCard(env: CommunityEnv, project: OpenSourceProject, open: (Route) -> Unit) {
    Column(Modifier.fillMaxWidth().outlinedCard(16.dp).clickable { open(project.route()) }) {
        OpenSourceProjectCover(env, project)
        Column(Modifier.padding(16.dp), verticalArrangement = Arrangement.spacedBy(8.dp)) {
            Text(project.title, style = XType.Secondary, fontWeight = FontWeight.Normal)
            Row(horizontalArrangement = Arrangement.spacedBy(8.dp)) { project.tags.take(2).forEach { GrayTag(it, compact = true) } }
        }
    }
}

@Composable
private fun OpenSourceCollectionCard(collection: OpenSourceCollection, open: (Route) -> Unit) {
    Column(Modifier.fillMaxWidth().outlinedCard(16.dp).clickable { open(Route.OpenSourceCollection(collection.key)) }.padding(20.dp), verticalArrangement = Arrangement.spacedBy(12.dp)) {
        Row(Modifier.fillMaxWidth(), verticalAlignment = Alignment.CenterVertically) {
            Text("专题精选", Modifier.weight(1f), style = XType.Caption, color = Brand)
            Text("${collection.projects.size} 个项目", style = XType.Caption, color = LocalExtraColors.current.ink3)
            XIcon(XIcons.ChevronRight, size = 16.dp, tint = LocalExtraColors.current.ink3)
        }
        Text(collection.title, style = XType.Section, fontWeight = FontWeight.SemiBold)
        Text(collection.summary, style = XType.Secondary, color = MaterialTheme.colorScheme.onSurfaceVariant)
    }
}

@Composable
internal fun OpenSourceCollectionScreen(env: CommunityEnv, key: String, back: () -> Unit, open: (Route) -> Unit) {
    val loader = rememberLoader(env, "open-source-collection:$key") { env.forum.openSourceCatalog() }
    val state by loader.state.collectAsState()
    Column(Modifier.fillMaxSize().background(MaterialTheme.colorScheme.background)) {
        ScreenBar("开源专题", onBack = back)
        LoadBox(state, loader::reload) { catalog ->
            val collection = catalog.collections.firstOrNull { it.key == key }
            if (collection == null) CenterMessage("这个专题尚未收录") else {
                LazyColumn(contentPadding = PaddingValues(16.dp), verticalArrangement = Arrangement.spacedBy(16.dp)) {
                    item {
                        Column(verticalArrangement = Arrangement.spacedBy(12.dp)) {
                            Text("专题精选 · ${collection.projects.size} 个项目", style = XType.Caption, color = Brand)
                            Text(collection.title, style = XType.Headline, fontWeight = FontWeight.SemiBold)
                            Text(collection.description, style = XType.Body, color = MaterialTheme.colorScheme.onSurfaceVariant)
                        }
                    }
                    collection.projects.forEachIndexed { index, name ->
                        catalog.project(name)?.let { project -> item(key = name) {
                            Column(verticalArrangement = Arrangement.spacedBy(12.dp)) {
                                Text("0${index + 1}  ${collection.notes.getOrNull(index).orEmpty()}", style = XType.Body, color = MaterialTheme.colorScheme.onSurfaceVariant)
                                OpenSourceCard(env, project, open)
                            }
                        } }
                    }
                    item { FootNote(catalog.notice) }
                }
            }
        }
    }
}

/** 精选项目使用与 Web 相同的 README 快照，评论继续复用仓库身份。 */
internal fun LazyListScope.curatedReadmeItems(
    env: CommunityEnv,
    catalog: OpenSourceCatalog,
    project: OpenSourceProject,
    state: Load<ProjectReadme?>,
    blocks: List<MdBlock>,
    chinese: Boolean,
    setChinese: (Boolean) -> Unit,
    retry: () -> Unit,
    open: (Route) -> Unit,
) {
    val readme = (state as? Load.Ready)?.value
    val github = readme?.card?.githubUrl.orEmpty().ifBlank { project.githubUrl }
    val follow: (String) -> Unit = { webPageRoute(it)?.let(open) }
    item(key = "curated-project") { CuratedProjectCard(env, catalog, project, readme?.card, github, follow) }
    item(key = "curated-readme-toolbar") {
        Row(
            Modifier.fillMaxWidth().padding(start = 16.dp, end = 12.dp, top = 12.dp),
            verticalAlignment = Alignment.CenterVertically,
        ) {
            Text("README", Modifier.weight(1f), style = XType.Section, fontWeight = FontWeight.SemiBold)
            if (readme != null) {
                PageChip("中文", chinese) { setChinese(true) }
                Spacer(Modifier.width(6.dp))
                PageChip("English", !chinese) { setChinese(false) }
            }
        }
        Hairline(Modifier.padding(horizontal = 12.dp))
    }
    when (state) {
        Load.Loading -> item(key = "curated-readme-loading") { Spinner(Modifier.fillMaxWidth().height(64.dp)) }
        is Load.Failed -> item(key = "curated-readme-error") {
            CenterMessage(state.message, "重试", retry, Modifier.fillMaxWidth())
        }
        is Load.Ready -> {
            if (readme == null) item(key = "curated-readme-missing") {
                Column(Modifier.fillMaxWidth().padding(16.dp), verticalArrangement = Arrangement.spacedBy(8.dp)) {
                    Text("双语 README 正在整理中，可以先阅读项目官方原文。", style = XType.Body)
                    OpenSourceLink("查看官方 README") { follow(project.sourceUrl) }
                    OpenSourceImage(env, project)
                }
            } else {
                items(blocks.size, key = { "curated-readme-block:$it" }) { index ->
                    Column(Modifier.fillMaxWidth().padding(horizontal = 16.dp, vertical = 5.dp)) {
                        MarkdownContentBlock(blocks[index], env.platform, follow)
                    }
                }
                item(key = "curated-readme-source") { ReadmeProvenance(readme, github, follow) }
            }
        }
    }
}

@Composable
private fun CuratedProjectCard(
    env: CommunityEnv,
    catalog: OpenSourceCatalog,
    project: OpenSourceProject,
    card: ProjectReadmeCard?,
    github: String,
    follow: (String) -> Unit,
) {
    val summary = card?.summaryZh.orEmpty().ifBlank { project.summary }
    val name = card?.name.orEmpty().ifBlank { project.name }
    val icon = project.iconUrl.ifBlank { card?.iconSourceUrl.orEmpty() }
    val primaryUrl = card?.primaryUrl.orEmpty().ifBlank { project.homepageUrl }
    var brokenIcon by remember(icon) { mutableStateOf(false) }
    val metadata = listOfNotNull(
        card?.stars?.let { "${formatStars(it)} Stars" },
        card?.codeUpdatedAt?.takeIf { it.isNotBlank() }?.let { "更新于 ${readmeDate(it)}" },
        card?.license?.takeIf { it.isNotBlank() },
        card?.platforms?.takeIf { it.isNotEmpty() }?.joinToString(" / "),
    )
    Column(Modifier.fillMaxWidth().padding(12.dp), verticalArrangement = Arrangement.spacedBy(10.dp)) {
        Text("${catalog.categoryLabel(project.category)} · OPEN SOURCE", style = XType.Caption, color = LocalExtraColors.current.ink3)
        Text("$name — ${summary.trimEnd('。', '.')}", style = XType.Headline, fontWeight = FontWeight.SemiBold)
        WhiteCard {
            Row(verticalAlignment = Alignment.CenterVertically, horizontalArrangement = Arrangement.spacedBy(12.dp)) {
                Box(
                    Modifier.size(44.dp).clip(RoundedCornerShape(10.dp)).background(LocalExtraColors.current.fill),
                    contentAlignment = Alignment.Center,
                ) {
                    if (icon.isBlank() || brokenIcon) Text(name.take(1), style = XType.Section)
                    else AsyncImage(
                        model = env.forum.imageUrl(icon), imageLoader = rememberImageLoader(),
                        contentDescription = null, modifier = Modifier.fillMaxSize().padding(4.dp),
                        onError = { brokenIcon = true },
                    )
                }
                Column(Modifier.weight(1f)) {
                    Text(name, style = XType.Section, fontWeight = FontWeight.SemiBold)
                    Text(project.fullName, Modifier.clickable { follow(github) }, style = XType.Caption, color = Brand)
                }
            }
            Text(summary, Modifier.padding(top = 12.dp), style = XType.Body)
            if (metadata.isNotEmpty()) Text(metadata.joinToString(" · "), Modifier.padding(top = 8.dp), style = XType.Caption)
            if (primaryUrl.isNotBlank() && primaryUrl != github) {
                OpenSourceLink(card?.primaryLabel.orEmpty().ifBlank { "项目官网" }) { follow(primaryUrl) }
            }
            OpenSourceLink("GitHub") { follow(github) }
            if (card != null && (card.githubArchived || card.upstreamMaintenanceNotice.isNotBlank())) {
                Text(card.upstreamMaintenanceNotice.ifBlank { "上游仓库已归档，使用前请留意维护状态。" }, style = XType.Caption)
            }
            card?.iconAttribution?.takeIf { it.isNotBlank() }?.let { Text(it, style = XType.Caption) }
        }
    }
}

@Composable
private fun ReadmeProvenance(readme: ProjectReadme, github: String, follow: (String) -> Unit) {
    var showLicenses by rememberSaveable(readme.fullName) { mutableStateOf(false) }
    Column(Modifier.fillMaxWidth().padding(16.dp), verticalArrangement = Arrangement.spacedBy(6.dp)) {
        Hairline()
        Text(readme.translation.ifBlank { "依据项目官方 README 整理" }, style = XType.Caption)
        Text("资料采集 ${readmeDate(readme.fetchedAt)}", style = XType.Caption)
        OpenSourceLink("查看上游原文") { follow(readme.sourceUrl) }
        if (readme.commit.isNotBlank()) {
            OpenSourceLink("版本 ${readme.commit.take(7)}") { follow("${github.trimEnd('/')}/commit/${readme.commit}") }
        }
        if (readme.licenses.isNotEmpty()) {
            TextButton(onClick = { showLicenses = !showLicenses }) {
                Text(if (showLicenses) "收起许可与版权声明" else "许可与版权声明")
            }
            if (showLicenses) readme.licenses.forEach { license ->
                Text(license.name, style = XType.Section)
                MarkdownContentBlock(MdBlock.Code("", license.text), null) {}
            }
        }
    }
}

@OptIn(kotlin.time.ExperimentalTime::class)
private fun readmeDate(value: String): String =
    runCatching { (Instant.parse(value) + 8.hours).toString().substringBefore('T') }
        .getOrElse { value.substringBefore('T') }

@Composable
private fun OpenSourceLink(text: String, onClick: () -> Unit) {
    Row(Modifier.fillMaxWidth().heightIn(min = 44.dp).clickable(onClick = onClick), verticalAlignment = Alignment.CenterVertically) {
        Text(text, Modifier.weight(1f), style = XType.Body)
        XIcon(XIcons.ExternalLink, size = 16.dp, tint = LocalExtraColors.current.ink3)
    }
}
