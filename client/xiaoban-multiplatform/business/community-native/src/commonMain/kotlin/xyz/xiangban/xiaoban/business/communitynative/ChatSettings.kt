package xyz.xiangban.xiaoban.business.communitynative

import androidx.compose.animation.animateColorAsState
import androidx.compose.foundation.background
import androidx.compose.foundation.clickable
import androidx.compose.foundation.horizontalScroll
import androidx.compose.foundation.selection.toggleable
import androidx.compose.foundation.shape.CircleShape
import androidx.compose.foundation.layout.*
import androidx.compose.foundation.rememberScrollState
import androidx.compose.foundation.verticalScroll
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.OutlinedTextField
import androidx.compose.material3.Text
import androidx.compose.runtime.*
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.draw.shadow
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.graphics.vector.ImageVector
import androidx.compose.ui.semantics.Role
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.unit.sp
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.unit.dp
import xyz.xiangban.xiaoban.common.community.*

@Composable
internal fun CloudPermissionSettings(chat: CloudChatController, cloud: CloudChatState, dismiss: () -> Unit) {
    var confirm by remember { mutableStateOf(false) }
    var error by remember { mutableStateOf<String?>(null) }
    fun choose(mode: String) {
        try { chat.setPermission(mode); dismiss() } catch (e: Exception) { error = e.message }
    }
    AgentChoiceRow(cloud.permissionMode == "ask", "默认询问", "修改内容、设提醒前先向你确认", { choose("ask") })
    AgentChoiceRow(cloud.permissionMode == "auto", "自动模式", "直接执行，高风险操作仍需确认", { confirm = true })
    if (cloud.grants.isNotEmpty()) {
        Text("此对话已授权", Modifier.padding(top = 12.dp), style = XType.Caption, color = MaterialTheme.colorScheme.onSurfaceVariant)
        Column(Modifier.heightIn(max = 220.dp).verticalScroll(rememberScrollState())) {
            cloud.grants.forEach { tool ->
                Row(Modifier.fillMaxWidth().padding(vertical = 10.dp)) {
                    Text(toolLabel(tool), Modifier.weight(1f), style = XType.Body)
                    LinkText("撤销", { try { chat.revokeGrant(tool) } catch (e: Exception) { error = e.message } })
                }
            }
        }
    }
    error?.let { Text(it, Modifier.padding(top = 8.dp), color = MaterialTheme.colorScheme.error, style = XType.Caption) }
    if (confirm) {
        CommunityCardDialog(
            title = "开启自动模式？",
            description = "开启后，小伴可以直接修改本对话的手机工作区、设置提醒。删除内容和覆盖已有作品仍需确认。你可以随时切回默认权限。",
            icon = XIcons.Lock,
            confirmLabel = "开启自动模式",
            onConfirm = { confirm = false; choose("auto") },
            onDismiss = { confirm = false },
        )
    }
}

@Composable
internal fun CloudExecutionSheet(chat: CloudChatController, cloud: CloudChatState, panel: Modifier, dismiss: () -> Unit) {
    var context by remember { mutableStateOf(cloud.contextTokens) }
    var turns by remember { mutableStateOf(cloud.budget.maxTurns?.toString().orEmpty()) }
    var calls by remember { mutableStateOf(cloud.budget.maxToolCalls?.toString().orEmpty()) }
    var minutes by remember { mutableStateOf(cloud.budget.maxMinutes?.toString().orEmpty()) }
    var tokens by remember { mutableStateOf(cloud.budget.maxTokens?.toString().orEmpty()) }
    var error by remember { mutableStateOf<String?>(null) }
    BoxWithConstraints(Modifier.fillMaxSize()) {
        BottomSheet(
            onDismiss = dismiss,
            panelModifier = panel.heightIn(max = maxHeight * 0.85f),
            showHandle = true,
            cancelLabel = "取消",
            confirmLabel = "保存",
            confirmEnabled = !cloud.sending,
            onConfirm = {
                try {
                    fun integer(text: String): Long? = if (text.isBlank()) null else text.toLongOrNull() ?: throw IllegalArgumentException("预算必须是正整数")
                    fun count(text: String): Int? = integer(text)?.let { require(it in 1..Int.MAX_VALUE.toLong()) { "预算超出范围" }; it.toInt() }
                    chat.setExecution(context, AgentExecutionBudget(count(turns), count(calls), integer(minutes), integer(tokens)))
                    dismiss()
                } catch (e: Exception) { error = e.message ?: "设置无效" }
            },
        ) {
            Text("这个对话的任务设置", style = XType.Section, fontWeight = FontWeight.SemiBold)
            Column(Modifier.weight(1f, fill = false).verticalScroll(rememberScrollState())) {
                Text("上下文", Modifier.padding(top = 12.dp), style = XType.Secondary)
                Row(Modifier.horizontalScroll(rememberScrollState()).padding(vertical = 10.dp), horizontalArrangement = Arrangement.spacedBy(6.dp)) {
                    listOf(32_000, 64_000, 128_000, 256_000, 512_000, 1_000_000).forEach { size ->
                        ChipPill(if (size == 1_000_000) "1M" else "${size / 1000}K", selected = context == size, onClick = { context = size })
                    }
                }
                if (cloud.nativeAgentAvailable) {
                    Text("本次任务预算", Modifier.padding(top = 8.dp), style = XType.Secondary)
                    Text("留空表示不限制。停止、防重复和无进展暂停始终有效。", Modifier.padding(vertical = 6.dp), style = XType.Caption, color = MaterialTheme.colorScheme.onSurfaceVariant)
                    OutlinedTextField(turns, { turns = it }, label = { Text("模型轮数") }, singleLine = true, modifier = Modifier.fillMaxWidth())
                    OutlinedTextField(calls, { calls = it }, label = { Text("工具次数") }, singleLine = true, modifier = Modifier.padding(top = 8.dp).fillMaxWidth())
                    OutlinedTextField(minutes, { minutes = it }, label = { Text("时间（分钟）") }, singleLine = true, modifier = Modifier.padding(top = 8.dp).fillMaxWidth())
                    OutlinedTextField(tokens, { tokens = it }, label = { Text("模型用量（Token）") }, singleLine = true, modifier = Modifier.padding(top = 8.dp).fillMaxWidth())
                }
                error?.let { Text(it, Modifier.padding(top = 8.dp), color = MaterialTheme.colorScheme.error, style = XType.Caption) }
            }
        }
    }
}

@Composable
internal fun AgentConfiguration(
    cloud: CloudChatState, model: String, quota: String, exhausted: Boolean, signedIn: Boolean, onlineComputer: String?,
    back: () -> Unit, onTopic: () -> Unit, onHistory: () -> Unit, onModel: () -> Unit, onQuota: () -> Unit,
    onPermission: () -> Unit, onBudget: () -> Unit, onDevices: () -> Unit, onAutoRead: (Boolean) -> Unit, onClear: () -> Unit,
) {
    val colors = MaterialTheme.colorScheme
    val extra = LocalExtraColors.current
    Column(Modifier.fillMaxSize().background(colors.background).statusBarsPadding().navigationBarsPadding()) {
        Box(Modifier.fillMaxWidth().height(48.dp).padding(horizontal = 8.dp), contentAlignment = Alignment.Center) {
            AgentIconButton(AiHomeIcons.Back, "返回", back, Modifier.align(Alignment.CenterStart))
            Text("小伴配置", style = XType.Title.copy(fontSize = 17.sp, lineHeight = 21.sp), fontWeight = FontWeight.Bold)
        }
        Column(Modifier.fillMaxWidth().weight(1f).verticalScroll(rememberScrollState()).padding(bottom = 24.dp)) {
            Row(Modifier.fillMaxWidth().background(colors.surface).padding(16.dp),
                verticalAlignment = Alignment.CenterVertically, horizontalArrangement = Arrangement.spacedBy(13.dp)) {
                AgentMascot(compact = true)
                Column(Modifier.weight(1f)) {
                    Text("小伴 AI", style = XType.Section, fontWeight = FontWeight(650))
                    Text("连续主线 · 记住你的偏好与常用习惯", Modifier.padding(top = 2.dp),
                        style = XType.Caption.copy(fontSize = 12.5.sp, lineHeight = 18.sp), color = colors.onSurfaceVariant)
                }
                Row(Modifier.background(extra.fill, CircleShape).padding(horizontal = 9.dp, vertical = 3.dp),
                    verticalAlignment = Alignment.CenterVertically, horizontalArrangement = Arrangement.spacedBy(5.dp)) {
                    Box(Modifier.size(6.dp).background(Success, CircleShape))
                    Text(if (cloud.nativeAgentAvailable) "云端 + 本机" else "云端", style = XType.Caption.copy(fontSize = 11.sp),
                        color = colors.onSurfaceVariant, fontWeight = FontWeight.Medium)
                }
            }

            AgentConfigGroup("话题与记录") {
                AgentConfigRow(AiHomeIcons.SparkleLine, "开启新话题", "在主线划出分割线并重置上下文，保留历史可回看", primary = true, onClick = onTopic)
                AgentConfigRow(AiHomeIcons.Search, "查找聊天记录", divider = true, onClick = onHistory)
            }
            AgentConfigGroup("模型与额度") {
                AgentConfigRow(AiHomeIcons.Cpu, "对话模型", "切换官方档位、思考等级或自有模型", model, onClick = onModel)
                AgentConfigRow(AiHomeIcons.Star, "模型配置", value = quota, valueTint = if (exhausted) Danger else colors.onSurfaceVariant,
                    divider = true, onClick = onQuota)
            }
            AgentConfigGroup("个人 Agent 能力") {
                AgentConfigRow(AiHomeIcons.Lock, "本机操作权限", "提醒、手机工作区与工具授权",
                    if (!cloud.nativeAgentAvailable) "不可用" else if (cloud.permissionMode == "auto") "自动模式" else "默认询问", onClick = onPermission)
                val context = cloud.contextTokens
                val contextLabel = if (context == 1_000_000) "1M" else "${context / 1000}K"
                val budgetLabel = if (cloud.budget == AgentExecutionBudget()) "不限" else "已设置"
                AgentConfigRow(AiHomeIcons.Sliders, "上下文与任务预算", value = "$contextLabel · $budgetLabel", divider = true, onClick = onBudget)
                val device = onlineComputer?.let { "$it 在线" } ?: if (signedIn) "电脑离线" else "未登录"
                AgentConfigRow(AiHomeIcons.Pc, "已连接电脑", value = device, divider = true, online = onlineComputer != null, onClick = onDevices)
                val autoRead = SpeechState.settings.autoRead
                AgentConfigRow(AiHomeIcons.Vol, "自动朗读回复", "回答完成后自动语音播报", divider = true,
                    autoRead = autoRead, onClick = { onAutoRead(!autoRead) })
            }
            Box(Modifier.padding(top = 14.dp).fillMaxWidth().height(50.dp).background(colors.surface)
                .clickable(onClick = onClear), contentAlignment = Alignment.Center) {
                Text("清空主线聊天记录", style = XType.Body, color = Danger, fontWeight = FontWeight.Medium)
            }
        }
    }
}

@Composable
private fun AgentConfigGroup(label: String, content: @Composable ColumnScope.() -> Unit) {
    Text(label, Modifier.padding(start = 16.dp, end = 16.dp, top = 14.dp, bottom = 6.dp), style = XType.Caption,
        color = LocalExtraColors.current.ink3, fontWeight = FontWeight.Medium)
    Column(Modifier.fillMaxWidth().background(MaterialTheme.colorScheme.surface), content = content)
}

@Composable
private fun AgentConfigRow(
    icon: ImageVector, title: String, description: String? = null, value: String? = null,
    valueTint: Color = MaterialTheme.colorScheme.onSurfaceVariant, primary: Boolean = false,
    divider: Boolean = false, online: Boolean = false, autoRead: Boolean? = null, onClick: () -> Unit,
) {
    val extra = LocalExtraColors.current
    val colors = MaterialTheme.colorScheme
    if (divider) Box(Modifier.padding(start = 16.dp).fillMaxWidth().height(.5.dp).background(AgentHairline))
    val action = if (autoRead == null) Modifier.clickable(onClick = onClick)
        else Modifier.toggleable(value = autoRead, role = Role.Switch, onValueChange = { onClick() })
    Row(Modifier.fillMaxWidth().heightIn(min = 52.dp).then(action).padding(horizontal = 16.dp, vertical = 11.dp),
        verticalAlignment = Alignment.CenterVertically, horizontalArrangement = Arrangement.spacedBy(12.dp)) {
        XIcon(icon, size = 19.dp, tint = if (primary) Brand else colors.onSurfaceVariant)
        Column(Modifier.weight(1f)) {
            Text(title, style = XType.Body.copy(lineHeight = 21.sp), color = if (primary) extra.brandInk else colors.onBackground,
                fontWeight = if (primary) FontWeight.SemiBold else FontWeight.Medium)
            if (description != null) Text(description, Modifier.padding(top = 1.dp), style = XType.Caption, color = extra.ink3)
        }
        if (value != null) {
            Row(Modifier.widthIn(max = 130.dp), verticalAlignment = Alignment.CenterVertically, horizontalArrangement = Arrangement.spacedBy(5.dp)) {
                if (online) Box(Modifier.size(6.dp).background(Success, CircleShape))
                Text(value, style = XType.Secondary, color = if (online) Success else valueTint,
                    fontWeight = if (online) FontWeight.Medium else FontWeight.Normal, maxLines = 1, overflow = TextOverflow.Ellipsis)
            }
        }
        if (autoRead == null) XIcon(AiHomeIcons.ChevRight, size = 15.dp, tint = extra.ink3.copy(alpha = .7f))
        else {
            val track by animateColorAsState(if (autoRead) Brand else AgentLine, label = "自动朗读开关")
            Box(Modifier.size(44.dp, 26.dp).clip(CircleShape).background(track).padding(2.dp)) {
                Box(Modifier.align(if (autoRead) Alignment.CenterEnd else Alignment.CenterStart)
                    .size(22.dp).shadow(2.dp, CircleShape).background(Color.White, CircleShape))
            }
        }
    }
}
