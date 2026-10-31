package xyz.xiangban.xiaoban.business.communitynative

import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.*
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.items
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.*
import androidx.compose.ui.Modifier
import androidx.compose.ui.ExperimentalComposeUiApi
import androidx.compose.ui.backhandler.BackHandler
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import xyz.xiangban.xiaoban.common.community.*

/** Private metadata stays in this screen; access starts closed and is reset on account changes. */
@OptIn(ExperimentalComposeUiApi::class)
@Composable
fun NativeSchemeBackupsScreen(env: CommunityEnv, back: () -> Unit, open: (Route) -> Unit) {
    val session by env.session.state.collectAsState()
    var enabled by remember(session.accountId) { mutableStateOf(false) }
    var removed by remember(session.accountId) { mutableStateOf(false) }
    var page by remember(session.accountId) { mutableStateOf(1) }
    var selected by remember(session.accountId) { mutableStateOf<String?>(null) }
    val api = remember(env, session.accountId) { SchemeBackupsApi(env.client, env.config, { env.session.accessToken() }, { enabled }) }
    BackHandler(enabled = selected != null) { selected = null }
    Column(Modifier.fillMaxSize()) {
        ScreenBar(if (selected == null) "账号方案备份" else "备份详情", onBack = { if (selected != null) selected = null else back() })
        if (!session.loggedIn) { CenterMessage("登录后查看自己的方案备份", "登录", { open(Route.Login) }); return@Column }
        Column(Modifier.padding(12.dp), verticalArrangement = Arrangement.spacedBy(8.dp)) {
            Text(if (enabled) "云端查看已开启" else "云端访问已关闭", fontSize = 15.sp, fontWeight = FontWeight.SemiBold)
            Text("备份仅自己可见，只保存主动选择的作品清单。手机可查看每项内容，不会自动应用。", fontSize = 12.sp, lineHeight = 19.sp, color = MaterialTheme.colorScheme.onSurfaceVariant)
            GhostPillButton(if (enabled) "关闭云端查看" else "开启云端查看", { enabled = !enabled; selected = null; page = 1 }, Modifier.fillMaxWidth())
        }
        if (!enabled) return@Column
        val id = selected
        if (id != null) {
            val loader = rememberLoader(env, "backup-detail-$id-${session.accountId}") { api.detail(id) }
            val state by loader.state.collectAsState()
            Box(Modifier.weight(1f)) { LoadBox(state, loader::reload) { backup ->
                LazyColumn(Modifier.fillMaxSize(), contentPadding = PaddingValues(12.dp), verticalArrangement = Arrangement.spacedBy(8.dp)) {
                    item { WhiteCard(padding = 14.dp) {
                        Text(backup.title, fontSize = 20.sp, fontWeight = FontWeight.SemiBold)
                        if (backup.summary.isNotBlank()) Text(backup.summary, Modifier.padding(top = 8.dp), fontSize = 14.sp)
                        Text("${if (backup.status == "removed") "回收站" else "已备份"} · 修订 ${backup.revision} · ${agoText(backup.updatedAt)}", Modifier.padding(top = 8.dp), fontSize = 12.sp, color = LocalExtraColors.current.ink3)
                    } }
                    items(backup.items, key = { it.packId }) { ref -> WhiteCard(padding = 14.dp) {
                        Text(if (ref.available) ref.title ?: "作品 #${ref.packId}" else "此项已不可用", fontSize = 15.sp, fontWeight = FontWeight.Medium)
                        Text("固定版本 v${ref.version}", Modifier.padding(top = 5.dp), fontSize = 12.sp, color = LocalExtraColors.current.ink3)
                        if (ref.available) GhostPillButton("查看公开作品", { open(if (ref.kind in CapabilityPackKinds) Route.Ability(ref.packId) else Route.Pack(ref.packId)) }, Modifier.padding(top = 8.dp))
                        else Text("作品已下架或当前不可公开访问。", Modifier.padding(top = 5.dp), fontSize = 12.sp, color = LocalExtraColors.current.ink3)
                    } }
                    item { FootNote("查看作品会打开当前公开详情，请核对备份标注的版本。手机不会下载或执行备份。") }
                }
            } }
        } else {
            TabStrip(listOf("active" to "已备份", "removed" to "回收站"), if (removed) "removed" else "active") { removed = it == "removed"; page = 1 }
            val loader = rememberLoader(env, "backups-$page-$removed-${session.accountId}") { api.list(page, removed) }
            val state by loader.state.collectAsState()
            Box(Modifier.weight(1f)) { LoadBox(state, loader::reload) { result ->
                if (result.items.isEmpty()) CenterMessage(if (page > 1) "这一页没有备份" else if (removed) "回收站为空" else "还没有账号方案备份", if (page > 1) "上一页" else "刷新", { if (page > 1) page-- else loader.reload() })
                else LazyColumn(Modifier.fillMaxSize(), contentPadding = PaddingValues(12.dp), verticalArrangement = Arrangement.spacedBy(8.dp)) {
                    item { GhostPillButton("刷新备份", loader::reload, Modifier.fillMaxWidth()) }
                    items(result.items, key = { it.id }) { row -> WhiteCard(Modifier.fillMaxWidth().clickable { selected = row.id }, padding = 14.dp) {
                        Text(row.title, fontSize = 16.sp, fontWeight = FontWeight.SemiBold)
                        if (row.summary.isNotBlank()) Text(row.summary, Modifier.padding(top = 6.dp), fontSize = 13.sp, maxLines = 2, color = MaterialTheme.colorScheme.onSurfaceVariant)
                        Text("${row.items.size} 项内容 · 修订 ${row.revision} · ${agoText(row.updatedAt)}", Modifier.padding(top = 6.dp), fontSize = 12.sp, color = LocalExtraColors.current.ink3)
                    } }
                    item { Row(horizontalArrangement = Arrangement.spacedBy(10.dp)) {
                        GhostPillButton("上一页", { page-- }, Modifier.weight(1f), enabled = page > 1)
                        Text("第 $page 页", Modifier.padding(top = 10.dp), fontSize = 12.sp)
                        GhostPillButton("下一页", { page++ }, Modifier.weight(1f), enabled = result.hasMore)
                    } }
                }
            } }
        }
    }
}
