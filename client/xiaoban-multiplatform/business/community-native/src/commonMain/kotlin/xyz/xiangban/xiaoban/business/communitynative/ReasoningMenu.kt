package xyz.xiangban.xiaoban.business.communitynative

import androidx.compose.foundation.background
import androidx.compose.foundation.border
import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.*
import androidx.compose.foundation.rememberScrollState
import androidx.compose.foundation.selection.selectable
import androidx.compose.foundation.selection.selectableGroup
import androidx.compose.foundation.selection.toggleable
import androidx.compose.foundation.shape.CircleShape
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.foundation.verticalScroll
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.alpha
import androidx.compose.ui.draw.clip
import androidx.compose.ui.draw.shadow
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.semantics.Role
import androidx.compose.ui.semantics.contentDescription
import androidx.compose.ui.semantics.semantics
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import xyz.xiangban.xiaoban.common.community.DEFAULT_CUSTOM_REASONING_EFFORTS

internal data class ReasoningOptionSpec(val id: String, val label: String, val desc: String)

internal val ReasoningOptionSpecs: List<ReasoningOptionSpec> = listOf(
    ReasoningOptionSpec("low", "低", "更快响应，适合日常简单问答"),
    ReasoningOptionSpec("medium", "中", "均衡速度与推理深度"),
    ReasoningOptionSpec("high", "高", "更深入分析与多步推导"),
    ReasoningOptionSpec("xhigh", "极高", "面向复杂工程与长链路推理"),
    ReasoningOptionSpec("max", "最高", "不限步深思，耗时较长"),
)

internal fun orderedReasoningLevels(levels: List<String>): List<String> =
    DEFAULT_CUSTOM_REASONING_EFFORTS.filter { it in levels } +
        levels.distinct().filter { it !in DEFAULT_CUSTOM_REASONING_EFFORTS }

internal fun effortTitle(effort: String?): String = when (effort) {
    null -> "默认"
    "low" -> "低"
    "medium" -> "中"
    "high" -> "高"
    "xhigh" -> "极高"
    "max" -> "最高"
    else -> effort
}

internal fun effortDescription(effort: String?): String = when (effort) {
    null -> "跟随模型自身设定"
    else -> ReasoningOptionSpecs.firstOrNull { it.id == effort }?.desc.orEmpty()
}

internal fun summarizeReasoningLevels(levels: List<String>): String {
    val ordered = orderedReasoningLevels(levels)
    return if (ordered.isEmpty()) "不支持" else ordered.joinToString(" / ", transform = ::effortTitle)
}

@Composable
internal fun ReasoningEffortChip(
    modelName: String,
    current: String?,
    enabled: Boolean,
    selectedRow: Boolean = false,
    onClick: () -> Unit,
) {
    val extra = LocalExtraColors.current
    val scheme = MaterialTheme.colorScheme
    val tint = if (selectedRow) scheme.onSurface else extra.ink3
    val selectedFill = if (ThemeState.dark) Color(0xFF2C2C30) else scheme.surface
    val level = when (current) {
        "low" -> 1
        "medium" -> 2
        "high", "xhigh", "max" -> 3
        else -> 0
    }
    Row(
        modifier = Modifier
            .heightIn(min = 28.dp)
            .alpha(if (enabled) 1f else 0.45f)
            .then(if (selectedRow) Modifier.shadow(1.dp, CircleShape).background(selectedFill, CircleShape)
                .border(0.5.dp, extra.line, CircleShape) else Modifier)
            .clip(CircleShape)
            .semantics { contentDescription = "$modelName，思考等级：${effortTitle(current)}" }
            .clickable(enabled = enabled, role = Role.Button, onClick = onClick)
            .padding(start = 9.dp, end = 8.dp),
        verticalAlignment = Alignment.CenterVertically,
        horizontalArrangement = Arrangement.spacedBy(5.dp),
    ) {
        Row(horizontalArrangement = Arrangement.spacedBy(1.5.dp), verticalAlignment = Alignment.Bottom) {
            val filled = if (selectedRow && level == 3) Brand else tint
            listOf(4.dp, 7.dp, 10.dp).forEachIndexed { index, height ->
                Box(Modifier.size(2.dp, height).background(if (index < level) filled else extra.line, RoundedCornerShape(1.dp)))
            }
        }
        Text(
            text = effortTitle(current),
            fontSize = 12.sp,
            lineHeight = 15.sp,
            fontWeight = if (selectedRow) FontWeight(550) else FontWeight.Medium,
            color = tint,
            maxLines = 1,
        )
        XIcon(XIcons.ChevronDown, size = 12.dp, tint = tint.copy(alpha = .55f))
    }
}

/** 对话模型选择中的思考等级单选弹层。 */
@Composable
internal fun ReasoningEffortBottomSheet(
    modelName: String,
    levels: List<String>,
    selected: String?,
    onDismiss: () -> Unit,
    onSelect: (String?) -> Unit,
    panelModifier: Modifier = Modifier,
) {
    val ordered = orderedReasoningLevels(levels)
    val current = selected?.takeIf { it in ordered }
    val extra = LocalExtraColors.current
    val scheme = MaterialTheme.colorScheme

    BottomSheet(
        onDismiss = onDismiss,
        panelModifier = panelModifier,
        flush = true,
        topPadding = 0.dp,
        scrim = ModelSheetScrim,
        showHandle = true,
        cancelLabel = "取消",
    ) {
        Column(Modifier.fillMaxWidth().padding(start = 16.dp, end = 16.dp, top = 6.dp, bottom = 10.dp)) {
            Text("思考等级", style = XType.Section, fontWeight = FontWeight.SemiBold)
            Text(
                modelName,
                Modifier.padding(top = 2.dp),
                style = XType.Caption,
                color = extra.ink3,
                maxLines = 1,
                overflow = TextOverflow.Ellipsis,
            )
        }
        Column(Modifier.fillMaxWidth().weight(1f, fill = false).heightIn(max = 420.dp).verticalScroll(rememberScrollState()).selectableGroup()) {
            val allOptions: List<String?> = listOf(null) + ordered
            allOptions.forEachIndexed { idx, effort ->
                if (idx > 0) Hairline(Modifier.padding(horizontal = 16.dp))
                val isSelected = effort == current
                Row(
                    modifier = Modifier
                        .fillMaxWidth()
                        .heightIn(min = 52.dp)
                        .selectable(isSelected, role = Role.RadioButton) { onSelect(effort) }
                        .padding(horizontal = 16.dp, vertical = 12.dp),
                    verticalAlignment = Alignment.CenterVertically,
                    horizontalArrangement = Arrangement.spacedBy(10.dp),
                ) {
                    Text(
                        text = effortTitle(effort),
                        style = XType.Body,
                        fontWeight = if (isSelected) FontWeight.SemiBold else FontWeight.Normal,
                        color = scheme.onSurface,
                    )
                    val desc = effortDescription(effort)
                    if (desc.isNotBlank()) {
                        Text(
                            text = desc,
                            modifier = Modifier.weight(1f),
                            style = XType.Caption,
                            color = extra.ink3,
                            maxLines = 1,
                            overflow = TextOverflow.Ellipsis,
                        )
                    } else {
                        Spacer(Modifier.weight(1f))
                    }
                    ModelCheck(isSelected)
                }
            }
        }
    }
}

/** 贴底思考等级多选 BottomSheet：供自定义第三方模型服务配置支持的思考档位（默认五档全选）。 */
@Composable
internal fun CustomReasoningMultiSheet(
    selected: List<String>,
    onChange: (List<String>) -> Unit,
    onDismiss: () -> Unit,
    panelModifier: Modifier = Modifier,
) {
    val extra = LocalExtraColors.current
    val scheme = MaterialTheme.colorScheme
    val allSelected = ReasoningOptionSpecs.all { it.id in selected }

    BottomSheet(
        onDismiss = onDismiss,
        panelModifier = panelModifier,
        flush = true,
        topPadding = 0.dp,
        scrim = ModelSheetScrim,
        showHandle = true,
        confirmLabel = "确定（已选 ${selected.size} 档）",
        onConfirm = onDismiss,
    ) {
        Row(
            Modifier.fillMaxWidth().padding(start = 16.dp, end = 16.dp, top = 6.dp, bottom = 10.dp),
            verticalAlignment = Alignment.CenterVertically,
            horizontalArrangement = Arrangement.spacedBy(8.dp),
        ) {
            Column(Modifier.weight(1f)) {
                Text("支持的思考等级（多选）", style = XType.Section, fontWeight = FontWeight.SemiBold)
                Text(
                    "勾选后可在对话模型列表中自由切换对应档位",
                    Modifier.padding(top = 2.dp),
                    style = XType.Caption,
                    color = extra.ink3,
                )
            }
            Text(
                text = if (allSelected) "清空" else "全选",
                modifier = Modifier
                    .clip(RoundedCornerShape(8.dp))
                    .clickable(role = Role.Button) {
                        onChange(if (allSelected) emptyList() else DEFAULT_CUSTOM_REASONING_EFFORTS)
                    }
                    .padding(horizontal = 8.dp, vertical = 5.dp),
                style = XType.Caption,
                fontWeight = FontWeight.Medium,
                color = scheme.onSurfaceVariant,
            )
        }
        Column(Modifier.fillMaxWidth().weight(1f, fill = false).heightIn(max = 380.dp).verticalScroll(rememberScrollState())) {
            ReasoningOptionSpecs.forEachIndexed { idx, spec ->
                if (idx > 0) Hairline(Modifier.padding(horizontal = 16.dp))
                val checked = spec.id in selected
                Row(
                    modifier = Modifier
                        .fillMaxWidth()
                        .heightIn(min = 52.dp)
                        .toggleable(checked, role = Role.Checkbox) { on ->
                            val next = if (on) orderedReasoningLevels(selected + spec.id) else selected - spec.id
                            onChange(next)
                        }
                        .padding(horizontal = 16.dp, vertical = 12.dp),
                    verticalAlignment = Alignment.CenterVertically,
                    horizontalArrangement = Arrangement.spacedBy(10.dp),
                ) {
                    val boxShape = RoundedCornerShape(6.dp)
                    Box(
                        modifier = Modifier
                            .size(20.dp)
                            .clip(boxShape)
                            .background(if (checked) Brand else Color.Transparent)
                            .then(if (!checked) Modifier.border(1.6.dp, extra.line, boxShape) else Modifier),
                        contentAlignment = Alignment.Center,
                    ) {
                        if (checked) XIcon(XIcons.Check, size = 13.dp, tint = Color.White)
                    }
                    Text(
                        text = spec.label,
                        style = XType.Body,
                        fontWeight = if (checked) FontWeight.SemiBold else FontWeight.Normal,
                        color = scheme.onSurface,
                    )
                    Text(
                        text = spec.desc,
                        modifier = Modifier.weight(1f),
                        style = XType.Caption,
                        color = extra.ink3,
                        maxLines = 1,
                        overflow = TextOverflow.Ellipsis,
                    )
                }
            }
        }
    }
}

@Composable
internal fun ModelPrimaryButton(
    label: String,
    onClick: () -> Unit,
    modifier: Modifier = Modifier,
) {
    Box(
        modifier = modifier
            .heightIn(min = 44.dp)
            .clip(CircleShape)
            .background(Brand)
            .clickable(role = Role.Button, onClick = onClick)
            .padding(horizontal = 16.dp, vertical = 5.dp),
        contentAlignment = Alignment.Center,
    ) {
        Text(label, style = XType.Body.copy(fontSize = 14.5.sp), fontWeight = FontWeight.SemiBold, color = Color.White)
    }
}
