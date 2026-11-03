package xyz.xiangban.xiaoban.business.communitynative

import androidx.compose.foundation.background
import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.*
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.items
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.*
import androidx.compose.runtime.saveable.rememberSaveable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.layout.ContentScale
import androidx.compose.ui.text.font.FontWeight
import coil3.compose.AsyncImage
import xyz.xiangban.xiaoban.common.community.Load
import xyz.xiangban.xiaoban.common.community.OpenSourceCatalog
import xyz.xiangban.xiaoban.common.community.OpenSourceCollection
import xyz.xiangban.xiaoban.common.community.OpenSourceProject
import androidx.compose.ui.unit.dp

private fun OpenSourceProject.route() = Route.OpenSource(fullName.substringBefore('/'), fullName.substringAfter('/'))

@Composable
internal fun OpenSourceSharesScreen(env: CommunityEnv, back: () -> Unit, open: (Route) -> Unit) {
    Column(Modifier.fillMaxSize()) { ScreenBar("社区分享", onBack = back); OpenSourceCommunityShares(env, open) }
}

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
internal fun OpenSourceTab(env: CommunityEnv, open: (Route) -> Unit, initialCategory: String = "recommend") {
    var category by rememberSaveable(initialCategory) { mutableStateOf(initialCategory) }
    var tag by rememberSaveable { mutableStateOf("") }
    val loader = rememberLoader(env, "open-source") { env.forum.openSourceCatalog() }
    val state by loader.state.collectAsState()
    val categoryTabs = (state as? Load.Ready<OpenSourceCatalog>)?.value?.categories.orEmpty().map { it.key to it.label }
    Column(Modifier.fillMaxSize()) {
        TabStrip(listOf("recommend" to "推荐", "hot" to "热榜") + categoryTabs, category, centered = true) { category = it; tag = "" }
        if (category == "hot") HotTab(env, open) else LoadBox(state, loader::reload) { catalog ->
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
                        LinkText("社区分享", { open(Route.OpenSourceShares) })
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

/** 由仓库详情页承载，评论继续复用同一仓库身份。 */
@Composable
internal fun OpenSourceArticle(env: CommunityEnv, catalog: OpenSourceCatalog, project: OpenSourceProject) {
    val (saved, setSaved) = rememberPrefFlag(env, "fav_repo:${project.fullName}")
    Column(Modifier.fillMaxWidth().background(MaterialTheme.colorScheme.background).padding(12.dp), verticalArrangement = Arrangement.spacedBy(12.dp)) {
        Column(Modifier.fillMaxWidth().outlinedCard(16.dp)) {
            Column(Modifier.padding(20.dp), verticalArrangement = Arrangement.spacedBy(12.dp)) {
                Text("${catalog.categoryLabel(project.category)} · ${project.name}", style = XType.Caption, color = LocalExtraColors.current.ink3)
                Text(project.title, style = XType.Headline, fontWeight = FontWeight.SemiBold)
                Text(project.fullName, style = XType.Secondary, color = LocalExtraColors.current.ink3)
                Text(project.summary, style = XType.Body, color = MaterialTheme.colorScheme.onSurfaceVariant)
            }
            OpenSourceImage(env, project)
            if (project.coverUrl.isNotBlank()) Text("${project.coverCaption} · 项目官方仓库", Modifier.padding(12.dp), style = XType.Caption, color = LocalExtraColors.current.ink3)
        }
        WhiteCard {
            Text("项目速览", style = XType.Section, fontWeight = FontWeight.SemiBold)
            Text(project.effort, Modifier.padding(top = 12.dp), style = XType.Body)
            Row(Modifier.fillMaxWidth().padding(top = 16.dp), horizontalArrangement = Arrangement.spacedBy(8.dp)) {
                GhostPillButton("前往 GitHub ↗", { env.platform?.openUrl(project.githubUrl) }, Modifier.weight(1f), height = 44.dp, fontSize = 14, textColor = LocalExtraColors.current.brandInk)
                GhostPillButton(if (saved) "已收藏" else "收藏项目", { setSaved(!saved) }, Modifier.weight(1f), height = 44.dp, fontSize = 14)
            }
        }
        OpenSourceSection("为什么值得看看") { project.highlights.forEachIndexed { index, text -> Text("0${index + 1}  $text", style = XType.Body, color = MaterialTheme.colorScheme.onSurfaceVariant) } }
        OpenSourceSection("适合谁") { Text(project.audience, style = XType.Body, color = MaterialTheme.colorScheme.onSurfaceVariant) }
        OpenSourceSection("从这里开始") { Text(project.gettingStarted, style = XType.Body, color = MaterialTheme.colorScheme.onSurfaceVariant) }
        OpenSourceSection("开始前了解一下") { project.notes.forEach { Text("· $it", style = XType.Body, color = MaterialTheme.colorScheme.onSurfaceVariant) } }
        OpenSourceSection("继续了解") {
            OpenSourceLink("项目官网与文档") { env.platform?.openUrl(project.homepageUrl) }
            OpenSourceLink("README 与源代码") { env.platform?.openUrl(project.sourceUrl) }
            OpenSourceLink("查看项目许可") { env.platform?.openUrl("${project.githubUrl}#license") }
            Text("${catalog.notice}\n资料整理：${catalog.reviewedAt}", style = XType.Caption, color = LocalExtraColors.current.ink3)
        }
    }
}

@Composable
private fun OpenSourceSection(title: String, content: @Composable ColumnScope.() -> Unit) {
    WhiteCard { Text(title, style = XType.Section, fontWeight = FontWeight.SemiBold); Column(Modifier.padding(top = 16.dp), verticalArrangement = Arrangement.spacedBy(12.dp), content = content) }
}

@Composable
private fun OpenSourceLink(text: String, onClick: () -> Unit) {
    Row(Modifier.fillMaxWidth().heightIn(min = 44.dp).clickable(onClick = onClick), verticalAlignment = Alignment.CenterVertically) {
        Text(text, Modifier.weight(1f), style = XType.Body)
        XIcon(XIcons.ExternalLink, size = 16.dp, tint = LocalExtraColors.current.ink3)
    }
}
