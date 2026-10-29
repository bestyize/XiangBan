package xyz.xiangban.xiaoban.business.communitynative

import androidx.compose.foundation.background
import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.layout.statusBarsPadding
import androidx.compose.foundation.layout.width
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.saveable.rememberSaveable
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp

private enum class CommunitySection(val label: String) { Home("首页"), Resources("资源"), Discuss("讨论") }

/** 社区：顶部是 首页 / 资源 / 讨论 三个栏目，下面是对应的内容列表。悬浮导航会盖住底部，所以内容留出一段空白。 */
@Composable
fun CommunityScreen(env: CommunityEnv, open: (Route) -> Unit) {
    var section by rememberSaveable { mutableStateOf(CommunitySection.Home.name) }
    Column(Modifier.fillMaxSize()) {
        CommunityHeader(CommunitySection.valueOf(section)) { section = it.name }
        Hairline()
        Box(Modifier.weight(1f).fillMaxWidth().padding(bottom = NavInset)) {
            when (CommunitySection.valueOf(section)) {
                CommunitySection.Home -> FeedScreen(env, open)
                CommunitySection.Resources -> DiscussScreen(env, open, category = "share", emptyText = "还没有资源分享")
                CommunitySection.Discuss -> DiscussScreen(env, open)
            }
        }
    }
}

@Composable
private fun CommunityHeader(current: CommunitySection, onSelect: (CommunitySection) -> Unit) {
    Column(Modifier.fillMaxWidth().background(MaterialTheme.colorScheme.surface).statusBarsPadding()) {
        Row(Modifier.fillMaxWidth().height(52.dp).padding(horizontal = 16.dp), verticalAlignment = Alignment.CenterVertically) {
            Box(Modifier.width(72.dp)) {
                Box(Modifier.size(28.dp).clip(RoundedCornerShape(8.dp)).background(Brand), contentAlignment = Alignment.Center) {
                    XIcon(XIcons.SparkleSmall, tint = Color.White, size = 18.dp)
                }
            }
            Row(Modifier.weight(1f), horizontalArrangement = Arrangement.spacedBy(28.dp, Alignment.CenterHorizontally)) {
                CommunitySection.entries.forEach { s ->
                    val selected = s == current
                    Column(Modifier.clickable { onSelect(s) }.height(52.dp), horizontalAlignment = Alignment.CenterHorizontally, verticalArrangement = Arrangement.Center) {
                        Text(
                            s.label, fontSize = 16.sp, fontWeight = if (selected) FontWeight.SemiBold else FontWeight.Normal,
                            color = if (selected) Brand else MaterialTheme.colorScheme.onSurfaceVariant,
                        )
                        Box(Modifier.padding(top = 4.dp).size(width = 18.dp, height = 3.dp).clip(RoundedCornerShape(2.dp)).background(if (selected) Brand else Color.Transparent))
                    }
                }
            }
            Spacer(Modifier.width(72.dp))
        }
    }
}

/** 游戏：小游戏的玩法和试玩还没有接入手机端，先把入口位置占住，并说清楚。 */
@Composable
fun GamesScreen() {
    Column(Modifier.fillMaxSize()) {
        HomeBar("游戏")
        Hairline()
        Box(Modifier.weight(1f).fillMaxWidth().padding(bottom = NavInset), contentAlignment = Alignment.Center) {
            EmptyState(XIcons.Gamepad, "游戏还在路上", "社区里的小游戏很快就能在手机上试玩。现在可以先去社区的“首页 → 游戏”里看看别人做的游戏。")
        }
    }
}
