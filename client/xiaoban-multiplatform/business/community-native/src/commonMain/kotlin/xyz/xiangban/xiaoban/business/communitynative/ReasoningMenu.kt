package xyz.xiangban.xiaoban.business.communitynative

import androidx.compose.foundation.background
import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.*
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material3.DropdownMenu
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.*
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.semantics.Role
import androidx.compose.ui.semantics.contentDescription
import androidx.compose.ui.semantics.selected
import androidx.compose.ui.semantics.semantics
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp

@Composable
internal fun ReasoningMenu(modelName: String, levels: List<String>, selected: String?, enabled: Boolean, onSelect: (String?) -> Unit) {
    var expanded by remember(modelName) { mutableStateOf(false) }
    val ordered = listOf("low", "medium", "high", "xhigh", "max").filter { it in levels } +
        levels.distinct().filter { it !in listOf("low", "medium", "high", "xhigh", "max") }
    val current = selected?.takeIf { it in levels }
    val colors = MaterialTheme.colorScheme
    Box(Modifier.padding(start = 10.dp)) {
        Row(
            Modifier.heightIn(min = 44.dp).clip(RoundedCornerShape(12.dp))
                .background(if (expanded) colors.primary.copy(alpha = 0.1f) else LocalExtraColors.current.fill)
                .semantics { contentDescription = "$modelName，思考等级：${effortTitle(current)}" }
                .clickable(enabled = enabled, role = Role.Button) { expanded = true }.padding(horizontal = 10.dp),
            verticalAlignment = Alignment.CenterVertically, horizontalArrangement = Arrangement.spacedBy(6.dp),
        ) {
            XIcon(XIcons.Brain, size = 15.dp, tint = colors.onSurfaceVariant)
            Text(effortTitle(current), style = XType.Caption, fontWeight = FontWeight.Medium)
            XIcon(if (expanded) XIcons.ChevronUp else XIcons.ChevronDown, size = 12.dp, tint = colors.onSurfaceVariant)
        }
        DropdownMenu(
            expanded = expanded, onDismissRequest = { expanded = false },
            modifier = Modifier.width(248.dp).heightIn(max = 440.dp),
            shape = RoundedCornerShape(18.dp), containerColor = colors.surface,
            tonalElevation = 0.dp, shadowElevation = 8.dp,
        ) {
            Column(Modifier.padding(horizontal = 16.dp, vertical = 10.dp)) {
                Text("思考等级", style = XType.Body, fontWeight = FontWeight.SemiBold)
                Text(modelName, Modifier.padding(top = 3.dp), style = XType.Caption,
                    color = colors.onSurfaceVariant, maxLines = 1, overflow = TextOverflow.Ellipsis)
            }
            (listOf<String?>(null) + ordered).forEach { effort ->
                val isSelected = effort == current
                Row(
                    Modifier.padding(horizontal = 6.dp, vertical = 1.dp).fillMaxWidth()
                        .clip(RoundedCornerShape(10.dp))
                        .background(if (isSelected) colors.primary.copy(alpha = 0.09f) else colors.surface)
                        .semantics { this.selected = isSelected }
                        .clickable(enabled = enabled, role = Role.RadioButton) { expanded = false; onSelect(effort) }
                        .heightIn(min = 48.dp).padding(horizontal = 12.dp, vertical = 10.dp),
                    verticalAlignment = Alignment.CenterVertically,
                ) {
                    Text(effortTitle(effort), style = XType.Body,
                        fontWeight = if (isSelected) FontWeight.SemiBold else FontWeight.Normal,
                        color = if (isSelected) colors.primary else colors.onSurface)
                    val hint = when (effort) { null -> "跟随模型"; "low" -> "更快响应"; "high" -> "更深入"; else -> null }
                    if (hint != null) Text(hint, Modifier.padding(start = 10.dp), style = XType.Caption, color = colors.onSurfaceVariant)
                    Spacer(Modifier.weight(1f))
                    if (isSelected) XIcon(XIcons.Check, size = 18.dp, tint = colors.primary)
                }
                if (effort == null) Box(Modifier.padding(horizontal = 16.dp, vertical = 5.dp).fillMaxWidth().height(1.dp).background(colors.outlineVariant))
            }
        }
    }
}

private fun effortTitle(effort: String?): String = when (effort) {
    null -> "默认"
    "low" -> "低"
    "medium" -> "中"
    "high" -> "高"
    "xhigh" -> "超高"
    "max" -> "最高"
    else -> effort
}
