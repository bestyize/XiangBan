package xyz.xiangban.xiaoban.business.communitynative

import androidx.compose.foundation.background
import androidx.compose.foundation.border
import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.*
import androidx.compose.foundation.rememberScrollState
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
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.semantics.Role
import androidx.compose.ui.semantics.contentDescription
import androidx.compose.ui.semantics.selected
import androidx.compose.ui.semantics.semantics
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.style.TextAlign
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

/**
 * 中性思考等级触发胶囊：文字区固定占两字宽度（居中对齐），整体等宽，
 * 无论显示「低 / 中 / 高」还是「默认 / 极高 / 最高」，多行纵向边缘严格对齐。
 */
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
    val shape = RoundedCornerShape(8.dp)
    val bg = if (selectedRow) scheme.surface else extra.fill
    Row(
        modifier = Modifier
            .height(28.dp)
            .widthIn(min = 74.dp)
            .alpha(if (enabled) 1f else 0.45f)
            .clip(shape)
            .background(bg)
            .then(if (selectedRow) Modifier.border(0.8.dp, extra.hairline, shape) else Modifier)
            .semantics { contentDescription = "$modelName，思考等级：${effortTitle(current)}" }
            .clickable(enabled = enabled, role = Role.Button, onClick = onClick)
            .padding(horizontal = 8.dp),
        verticalAlignment = Alignment.CenterVertically,
        horizontalArrangement = Arrangement.SpaceBetween,
    ) {
        XIcon(XIcons.Brain, size = 13.dp, tint = scheme.onSurfaceVariant)
        Text(
            text = effortTitle(current),
            modifier = Modifier.widthIn(min = 26.dp),
            fontSize = 11.5.sp,
            lineHeight = 15.sp,
            fontWeight = FontWeight.Medium,
            color = scheme.onSurfaceVariant,
            textAlign = TextAlign.Center,
            maxLines = 1,
        )
        XIcon(XIcons.ChevronDown, size = 12.dp, tint = scheme.onSurfaceVariant)
    }
}

/** 贴底思考等级单选 BottomSheet：用于小伴 AI 官方配置页与对话模型选择弹层。 */
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

    BottomSheet(onDismiss = onDismiss, panelModifier = panelModifier, flush = true, cornerRadius = 18.dp) {
        SheetDragHandle()
        Row(
            Modifier.fillMaxWidth().padding(start = 16.dp, end = 14.dp, bottom = 12.dp),
            verticalAlignment = Alignment.CenterVertically,
            horizontalArrangement = Arrangement.spacedBy(8.dp),
        ) {
            Column(Modifier.weight(1f)) {
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
            SheetCloseCircle(onDismiss)
        }
        Column(Modifier.fillMaxWidth().heightIn(max = 420.dp).verticalScroll(rememberScrollState())) {
            val allOptions: List<String?> = listOf(null) + ordered
            allOptions.forEachIndexed { idx, effort ->
                if (idx > 0) Hairline(Modifier.padding(start = 16.dp))
                val isSelected = effort == current
                Row(
                    modifier = Modifier
                        .fillMaxWidth()
                        .heightIn(min = 52.dp)
                        .semantics { this.selected = isSelected }
                        .clickable(role = Role.RadioButton) { onSelect(effort) }
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
                    Box(Modifier.size(18.dp), contentAlignment = Alignment.Center) {
                        if (isSelected) XIcon(XIcons.Check, size = 17.dp, tint = extra.brandInk)
                    }
                }
            }
        }
        Spacer(Modifier.height(8.dp))
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

    BottomSheet(onDismiss = onDismiss, panelModifier = panelModifier, flush = true, cornerRadius = 18.dp) {
        SheetDragHandle()
        Row(
            Modifier.fillMaxWidth().padding(start = 16.dp, end = 14.dp, bottom = 12.dp),
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
            SheetCloseCircle(onDismiss)
        }
        Column(Modifier.fillMaxWidth().heightIn(max = 380.dp).verticalScroll(rememberScrollState())) {
            ReasoningOptionSpecs.forEachIndexed { idx, spec ->
                if (idx > 0) Hairline(Modifier.padding(start = 16.dp))
                val checked = spec.id in selected
                Row(
                    modifier = Modifier
                        .fillMaxWidth()
                        .heightIn(min = 52.dp)
                        .clickable(role = Role.Checkbox) {
                            val next = if (checked) selected - spec.id else orderedReasoningLevels(selected + spec.id)
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
        Hairline()
        Box(Modifier.fillMaxWidth().padding(start = 16.dp, end = 16.dp, top = 12.dp, bottom = 10.dp)) {
            SoftAccentPillButton(
                label = "确定（已选 ${selected.size} 档）",
                onClick = onDismiss,
                modifier = Modifier.fillMaxWidth(),
            )
        }
    }
}

@Composable
internal fun SheetDragHandle() {
    Box(Modifier.fillMaxWidth().padding(top = 8.dp, bottom = 10.dp), contentAlignment = Alignment.Center) {
        Box(Modifier.size(36.dp, 4.dp).clip(CircleShape).background(LocalExtraColors.current.line))
    }
}

@Composable
internal fun SheetCloseCircle(onClick: () -> Unit) {
    Box(
        Modifier
            .size(30.dp)
            .clip(CircleShape)
            .background(LocalExtraColors.current.fill)
            .clickable(role = Role.Button, onClick = onClick),
        contentAlignment = Alignment.Center,
    ) {
        XIcon(XIcons.Close, size = 15.dp, tint = MaterialTheme.colorScheme.onSurfaceVariant)
    }
}

@Composable
internal fun SoftAccentPillButton(
    label: String,
    onClick: () -> Unit,
    modifier: Modifier = Modifier,
    enabled: Boolean = true,
) {
    val extra = LocalExtraColors.current
    val lineColor = Brand.copy(alpha = if (ThemeState.dark) 0.36f else 0.24f)
    Box(
        modifier = modifier
            .height(44.dp)
            .alpha(if (enabled) 1f else 0.45f)
            .clip(CircleShape)
            .background(extra.brandSoft)
            .border(1.dp, lineColor, CircleShape)
            .clickable(enabled = enabled, role = Role.Button, onClick = onClick)
            .padding(horizontal = 18.dp),
        contentAlignment = Alignment.Center,
    ) {
        Text(label, style = XType.Body, fontWeight = FontWeight.SemiBold, color = extra.brandInk)
    }
}
