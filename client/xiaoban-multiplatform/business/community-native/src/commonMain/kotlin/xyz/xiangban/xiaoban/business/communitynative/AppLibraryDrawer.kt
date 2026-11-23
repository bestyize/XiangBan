package xyz.xiangban.xiaoban.business.communitynative

import androidx.compose.foundation.background
import androidx.compose.foundation.layout.*
import androidx.compose.foundation.rememberScrollState
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.foundation.verticalScroll
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.*
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.unit.dp
import xyz.xiangban.xiaoban.common.community.AppShortcut

@Composable
internal fun AppLibraryDrawer(
    env: CommunityEnv, recent: List<AppShortcut>, loading: Boolean, errors: List<String>,
    reload: () -> Unit, dismiss: () -> Unit, choose: (AppShortcut) -> Unit,
) {
    Column(Modifier.fillMaxSize().clip(RoundedCornerShape(bottomStart = 16.dp, bottomEnd = 16.dp))
        .background(MaterialTheme.colorScheme.surface).statusBarsPadding()
        .padding(start = 16.dp, end = 16.dp, top = 12.dp, bottom = 16.dp)) {
        Column(Modifier.weight(1f).verticalScroll(rememberScrollState()), verticalArrangement = Arrangement.spacedBy(12.dp)) {
            Row(verticalAlignment = Alignment.CenterVertically) {
                Text("最近用过", Modifier.weight(1f), style = XType.Section, fontWeight = FontWeight.SemiBold)
                IconTap(XIcons.Close, "收起最近用过", dismiss)
            }
            if (loading) Spinner(Modifier.fillMaxWidth().height(48.dp))
            else if (recent.isEmpty()) Text("还没有使用记录", style = XType.Secondary, color = LocalExtraColors.current.ink3)
            AppShortcutGrid(env, recent, choose = choose)
            if (errors.isNotEmpty()) Banner(errors.joinToString(" · "), warn = true, actionText = "重试", onAction = reload)
        }
    }
}

@Composable
internal fun AppShortcutGrid(env: CommunityEnv, items: List<AppShortcut>, choose: (AppShortcut) -> Unit) {
    items.chunked(4).forEach { row -> Row(Modifier.fillMaxWidth(), horizontalArrangement = Arrangement.spacedBy(8.dp)) {
        row.forEach { item -> key(item.key) {
            if (item.source == "community") {
                CommunityShortcutTile(env, item, Modifier.weight(1f), showGameDetails = true) { choose(item) }
            } else AppShortcutTile(env, item, Modifier.weight(1f)) { choose(item) }
        } }
        repeat(4 - row.size) { Spacer(Modifier.weight(1f)) }
    } }
}
