package xyz.xiangban.xiaoban.business.communitynative

import androidx.compose.foundation.background
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.PaddingValues
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.widthIn
import androidx.compose.foundation.lazy.LazyRow
import androidx.compose.foundation.lazy.items
import androidx.compose.foundation.lazy.rememberLazyListState
import androidx.compose.foundation.selection.selectable
import androidx.compose.foundation.selection.selectableGroup
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.snapshotFlow
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.alpha
import androidx.compose.ui.draw.drawBehind
import androidx.compose.ui.geometry.CornerRadius
import androidx.compose.ui.geometry.Offset
import androidx.compose.ui.geometry.Size
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.graphics.lerp
import androidx.compose.ui.semantics.Role
import androidx.compose.ui.semantics.clearAndSetSemantics
import androidx.compose.ui.text.TextStyle
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.unit.dp
import kotlinx.coroutines.flow.first

internal val communityBarSurface: Color
    get() = if (ThemeState.dark) lerp(Color(0xFF191F2C), Color(0xFF0F1219), .2f)
        else lerp(Color.White, Color(0xFFF2F5FA), .2f)

@Composable
internal fun CommunityNavLabel(
    label: String,
    active: Boolean,
    style: TextStyle,
    activeWeight: FontWeight = FontWeight.Bold,
) {
    val underline = if (active) Modifier.drawBehind {
        val thickness = 2.dp.toPx()
        val width = 20.dp.toPx()
        // Draw below the label without including the underline in its measured height.
        drawRoundRect(
            color = Brand,
            topLeft = Offset((size.width - width) / 2f, size.height + 2.dp.toPx()),
            size = Size(width, thickness),
            cornerRadius = CornerRadius(thickness / 2f),
        )
    } else Modifier
    Box(contentAlignment = Alignment.Center) {
        // Keep hit areas and text centers stationary when switching font weight.
        Text(label, Modifier.alpha(0f).clearAndSetSemantics {}, style = style, fontWeight = activeWeight,
            maxLines = 1, overflow = TextOverflow.Ellipsis)
        Text(
            label, modifier = underline, style = style, color = if (active) feedInk else feedMuted,
            fontWeight = if (active) activeWeight else FontWeight.Medium,
            maxLines = 1, overflow = TextOverflow.Ellipsis,
        )
    }
}

@Composable
internal fun CommunityTabRow(
    tabs: List<Pair<String, String>>,
    selected: String,
    modifier: Modifier = Modifier,
    filter: Boolean = false,
    onSelect: (String) -> Unit,
) {
    val state = rememberLazyListState()
    LaunchedEffect(selected, tabs) {
        val index = tabs.indexOfFirst { it.first == selected }
        if (index >= 0) {
            val layout = snapshotFlow { state.layoutInfo }.first { it.visibleItemsInfo.isNotEmpty() }
            val visible = layout.visibleItemsInfo.any {
                it.key == selected && it.offset >= layout.viewportStartOffset &&
                    it.offset + it.size <= layout.viewportEndOffset
            }
            if (!visible) state.animateScrollToItem(index)
        }
    }
    val style = if (filter) feedText(12, 18.0) else feedText(13, 19.5)
    LazyRow(
        modifier.fillMaxWidth().then(if (filter) Modifier else Modifier.background(communityBarSurface).padding(top = 2.dp, bottom = 8.dp))
            .selectableGroup(),
        state = state,
        contentPadding = if (filter) PaddingValues() else PaddingValues(horizontal = 10.dp),
        horizontalArrangement = Arrangement.spacedBy(4.dp, if (filter) Alignment.Start else Alignment.CenterHorizontally),
        verticalAlignment = Alignment.CenterVertically,
    ) {
        items(tabs, key = { it.first }) { (key, label) ->
            val active = key == selected
            Box(
                Modifier.height(30.dp)
                    .selectable(active, role = Role.Tab, onClick = { onSelect(key) })
                    .padding(horizontal = if (filter) 7.dp else 10.dp)
                    .widthIn(max = 200.dp),
                contentAlignment = Alignment.Center,
            ) {
                CommunityNavLabel(label, active, style)
            }
        }
    }
}
