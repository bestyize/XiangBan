package xyz.xiangban.xiaoban.business.communitynative

import androidx.compose.foundation.background
import androidx.compose.foundation.border
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
import androidx.compose.foundation.layout.width
import androidx.compose.foundation.layout.wrapContentWidth
import androidx.compose.foundation.rememberScrollState
import androidx.compose.foundation.shape.CircleShape
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.foundation.verticalScroll
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.graphics.Brush
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.unit.Dp
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import xyz.xiangban.xiaoban.common.community.DesignTokens as Tokens

/** 设置 → 主题：跟随系统 / 浅色 / 深色，三张迷你界面预览，选中的描边加对号。 */
@Composable
fun AppearanceScreen(back: () -> Unit, open: (Route) -> Unit) {
    val mode = ThemeState.mode
    Column(Modifier.fillMaxSize()) {
        ScreenBar("主题", onBack = back)
        Column(Modifier.weight(1f).fillMaxWidth().verticalScroll(rememberScrollState())) {
            GroupLabel("外观模式")
            Row(Modifier.padding(horizontal = 16.dp).fillMaxWidth().background(MaterialTheme.colorScheme.surface).padding(horizontal = 16.dp, vertical = 20.dp), horizontalArrangement = Arrangement.SpaceBetween) {
                AppearanceMode.entries.forEach { m -> ModeCard(m, selected = mode == m) { ThemeState.select(m) } }
            }
            Text("跟随系统时，随手机的深浅色设置自动切换。", Modifier.padding(start = 16.dp, end = 16.dp, top = 6.dp), fontSize = 12.sp, lineHeight = 18.sp, color = MaterialTheme.colorScheme.onSurfaceVariant)
            GroupLabel("主题色")
            Row(
                Modifier.padding(horizontal = 16.dp).fillMaxWidth().background(MaterialTheme.colorScheme.surface).clickable { open(Route.AccentColor) }.padding(horizontal = 16.dp, vertical = 14.dp),
                verticalAlignment = Alignment.CenterVertically, horizontalArrangement = Arrangement.spacedBy(12.dp),
            ) {
                Text("主题色", Modifier.weight(1f), fontSize = 15.sp)
                Box(Modifier.size(16.dp).clip(CircleShape).background(Brand))
                Text(ThemeState.accent.name, fontSize = 12.sp, color = MaterialTheme.colorScheme.onSurfaceVariant)
                XIcon(XIcons.ChevronRight, tint = LocalExtraColors.current.ink3, size = 16.dp)
            }
        }
    }
}

@Composable
private fun ModeCard(mode: AppearanceMode, selected: Boolean, onClick: () -> Unit) {
    val palette = ThemeState.accent
    val ring = Color(if (ThemeState.dark) palette.dark.btn else palette.light.btn)
    Column(Modifier.clip(RoundedCornerShape(12.dp)).clickable(onClick = onClick), horizontalAlignment = Alignment.CenterHorizontally, verticalArrangement = Arrangement.spacedBy(12.dp)) {
        Box(Modifier.size(100.dp, 140.dp).clip(RoundedCornerShape(12.dp)).let { if (selected) it.border(2.dp, ring, RoundedCornerShape(12.dp)) else it }) {
            when (mode) {
                AppearanceMode.Light -> MiniPhone(dark = false, scale = 0.606f)
                AppearanceMode.Dark -> MiniPhone(dark = true, scale = 0.606f)
                // 跟随系统：左半边浅色、右半边深色。
                AppearanceMode.System -> Row {
                    Box(Modifier.size(50.dp, 140.dp).clip(RoundedCornerShape(0.dp))) { MiniPhone(dark = false, scale = 0.606f, modifier = Modifier.wrapContentWidth(Alignment.Start, unbounded = true)) }
                    Box(Modifier.size(50.dp, 140.dp).clip(RoundedCornerShape(0.dp))) { MiniPhone(dark = true, scale = 0.606f, modifier = Modifier.wrapContentWidth(Alignment.End, unbounded = true)) }
                }
            }
        }
        Text(mode.label, fontSize = 14.sp, fontWeight = if (selected) FontWeight.SemiBold else FontWeight.Normal)
        Box(
            Modifier.size(20.dp).clip(CircleShape).let { if (selected) it.background(Brand) else it.border(1.5.dp, LocalExtraColors.current.line, CircleShape) },
            contentAlignment = Alignment.Center,
        ) { if (selected) XIcon(XIcons.CheckBold, tint = Color.White, size = 12.dp) }
    }
}

/** 设置 → 主题色：上面是浅色和深色两个迷你界面预览，下面六个色点（2 行 3 列），点哪个预览就跟着换。 */
@Composable
fun AccentScreen(back: () -> Unit) {
    val current = ThemeState.accent
    Column(Modifier.fillMaxSize()) {
        ScreenBar("主题色", onBack = back)
        Column(Modifier.weight(1f).fillMaxWidth().verticalScroll(rememberScrollState())) {
            GroupLabel("效果")
            Row(Modifier.padding(horizontal = 16.dp).fillMaxWidth().background(MaterialTheme.colorScheme.surface).padding(16.dp), horizontalArrangement = Arrangement.SpaceBetween) {
                listOf(false to "浅色", true to "深色").forEach { (dark, label) ->
                    Column(horizontalAlignment = Alignment.CenterHorizontally, verticalArrangement = Arrangement.spacedBy(8.dp)) {
                        Box(Modifier.size(152.dp, 214.dp).clip(RoundedCornerShape(12.dp))) { MiniPhone(dark = dark, scale = 0.92f, accent = current) }
                        Text(label, fontSize = 12.sp, color = MaterialTheme.colorScheme.onSurfaceVariant)
                    }
                }
            }
            GroupLabel("主题色")
            Column(Modifier.padding(horizontal = 16.dp).fillMaxWidth().background(MaterialTheme.colorScheme.surface).padding(start = 8.dp, end = 8.dp, top = 20.dp, bottom = 12.dp), verticalArrangement = Arrangement.spacedBy(20.dp)) {
                Tokens.Accents.chunked(3).forEach { row ->
                    Row(Modifier.fillMaxWidth()) {
                        row.forEach { accent -> Box(Modifier.weight(1f), contentAlignment = Alignment.Center) { Swatch(accent, accent.id == current.id) { ThemeState.select(accent) } } }
                    }
                }
            }
        }
    }
}

@Composable
private fun Swatch(accent: Tokens.Accent, selected: Boolean, onClick: () -> Unit) {
    val p = accent.light
    Column(Modifier.clip(RoundedCornerShape(12.dp)).clickable(onClick = onClick).padding(4.dp), horizontalAlignment = Alignment.CenterHorizontally, verticalArrangement = Arrangement.spacedBy(8.dp)) {
        Box(Modifier.size(52.dp), contentAlignment = Alignment.Center) {
            if (selected) Box(Modifier.size(52.dp).border(2.dp, Color(p.btn), CircleShape))
            Box(Modifier.size(44.dp).clip(CircleShape).background(Brush.linearGradient(listOf(Color(p.hi), Color(p.btn)))), contentAlignment = Alignment.Center) {
                if (selected) XIcon(XIcons.CheckBold, tint = Color.White, size = 20.dp)
            }
        }
        Text(accent.name, fontSize = 14.sp, fontWeight = if (selected) FontWeight.SemiBold else FontWeight.Normal)
    }
}

/**
 * 迷你手机界面：顶栏、用户气泡、带按钮的卡片、开关、输入框。尺寸按设计稿 165×232 乘 [scale]，
 * 颜色是这个外观下的固定配色，主题色跟着 [accent] 走，用来在主题页和主题色页里预览效果。
 */
@Composable
private fun MiniPhone(dark: Boolean, scale: Float, modifier: Modifier = Modifier, accent: Tokens.Accent = ThemeState.accent) {
    fun d(v: Float): Dp = (v * scale).dp
    val pal = if (dark) accent.dark else accent.light
    val page = Color(if (dark) Tokens.PaperDark else Tokens.PaperLight)
    val card = Color(if (dark) Tokens.SurfaceDark else Tokens.SurfaceLight)
    val fill = Color(if (dark) Tokens.FillDark else Tokens.FillLight)
    val bar = Color(if (dark) 0xFF3A3A3CL else 0xFFD9D9D9L)
    val btn = Color(pal.btn)
    val soft = Color(pal.soft)
    val ink = Color(pal.ink)
    @Composable
    fun Bar(w: Float, color: Color, h: Float = 6f) = Box(Modifier.size(d(w), d(h)).clip(RoundedCornerShape(d(h / 2))).background(color))

    Column(modifier.size(d(165f), d(232f)).background(page)) {
        Row(Modifier.fillMaxWidth().height(d(28f)).background(card).padding(horizontal = d(10f)), verticalAlignment = Alignment.CenterVertically) {
            Bar(48f, bar)
            Spacer(Modifier.weight(1f))
            Box(Modifier.size(d(14f)).clip(CircleShape).background(btn))
        }
        Column(Modifier.weight(1f).fillMaxWidth().padding(d(10f)), verticalArrangement = Arrangement.spacedBy(d(8f))) {
            Box(Modifier.align(Alignment.End).size(d(88f), d(26f)).clip(RoundedCornerShape(d(8f))).background(soft).padding(horizontal = d(8f)), contentAlignment = Alignment.CenterStart) { Bar(56f, ink, 5f) }
            Column(Modifier.width(d(116f)).clip(RoundedCornerShape(d(8f))).background(card).padding(d(8f)), verticalArrangement = Arrangement.spacedBy(d(6f))) {
                Bar(92f, bar); Bar(64f, bar)
                Row(Modifier.padding(top = d(2f)), horizontalArrangement = Arrangement.spacedBy(d(6f))) {
                    Box(Modifier.size(d(52f), d(20f)).clip(CircleShape).background(soft), contentAlignment = Alignment.Center) { Bar(26f, ink, 4f) }
                    Box(Modifier.size(d(38f), d(20f)).clip(CircleShape).background(fill))
                }
            }
            Row(Modifier.fillMaxWidth().clip(RoundedCornerShape(d(8f))).background(card).padding(d(8f)), verticalAlignment = Alignment.CenterVertically) {
                Bar(60f, bar)
                Spacer(Modifier.weight(1f))
                Box(Modifier.size(d(28f), d(16f)).clip(CircleShape).background(btn).padding(d(2f)), contentAlignment = Alignment.CenterEnd) { Box(Modifier.size(d(12f)).clip(CircleShape).background(Color.White)) }
            }
        }
        Row(Modifier.padding(start = d(8f), end = d(8f), bottom = d(8f)).fillMaxWidth().height(d(34f)).clip(CircleShape).background(card).padding(start = d(12f), end = d(5f)), verticalAlignment = Alignment.CenterVertically) {
            Bar(60f, bar, 5f)
            Spacer(Modifier.weight(1f))
            Box(Modifier.size(d(24f)).clip(CircleShape).background(btn), contentAlignment = Alignment.Center) { XIcon(XIcons.ArrowRight, tint = Color.White, size = d(12f)) }
        }
    }
}
