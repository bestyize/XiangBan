package xyz.xiangban.xiaoban.business.communitynative

import androidx.compose.foundation.horizontalScroll
import androidx.compose.foundation.layout.*
import androidx.compose.foundation.rememberScrollState
import androidx.compose.foundation.verticalScroll
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.OutlinedTextField
import androidx.compose.material3.Text
import androidx.compose.runtime.*
import androidx.compose.ui.Modifier
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
    Text(if (confirm) "开启完全信任？" else "这个对话的权限", style = XType.Title, fontWeight = FontWeight.SemiBold)
    if (confirm) {
        Text("开启后，小伴可以直接修改本对话的手机工作区、设置提醒。删除内容和覆盖已有作品仍需确认。你可以随时切回默认权限。", Modifier.padding(top = 12.dp), style = XType.Secondary)
        Row(Modifier.padding(top = 16.dp), horizontalArrangement = Arrangement.spacedBy(10.dp)) {
            GhostPillButton("取消", { confirm = false }, Modifier.weight(1f))
            PillButton("开启完全信任", { choose("auto") }, Modifier.weight(1f))
        }
    } else {
        RadioRow(cloud.permissionMode == "ask", "默认权限", "修改内容前先询问", { choose("ask") })
        RadioRow(cloud.permissionMode == "auto", "完全信任", "直接执行，高风险操作仍需确认", { confirm = true })
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
        GhostPillButton("关闭", dismiss, Modifier.padding(top = 12.dp).fillMaxWidth())
    }
    error?.let { Text(it, Modifier.padding(top = 8.dp), color = MaterialTheme.colorScheme.error, style = XType.Caption) }
}

@Composable
internal fun ColumnScope.CloudExecutionSettings(chat: CloudChatController, cloud: CloudChatState, dismiss: () -> Unit) {
    var context by remember { mutableStateOf(cloud.contextTokens) }
    var turns by remember { mutableStateOf(cloud.budget.maxTurns?.toString().orEmpty()) }
    var calls by remember { mutableStateOf(cloud.budget.maxToolCalls?.toString().orEmpty()) }
    var minutes by remember { mutableStateOf(cloud.budget.maxMinutes?.toString().orEmpty()) }
    var tokens by remember { mutableStateOf(cloud.budget.maxTokens?.toString().orEmpty()) }
    var error by remember { mutableStateOf<String?>(null) }
    Text("这个对话的任务设置", style = XType.Title, fontWeight = FontWeight.SemiBold)
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
    PillButton("保存", {
        try {
            fun integer(text: String): Long? = if (text.isBlank()) null else text.toLongOrNull() ?: throw IllegalArgumentException("预算必须是正整数")
            fun count(text: String): Int? = integer(text)?.let { require(it in 1..Int.MAX_VALUE.toLong()) { "预算超出范围" }; it.toInt() }
            chat.setExecution(context, AgentExecutionBudget(count(turns), count(calls), integer(minutes), integer(tokens)))
            dismiss()
        } catch (e: Exception) { error = e.message ?: "设置无效" }
    }, Modifier.padding(top = 12.dp).fillMaxWidth(), enabled = !cloud.sending)
    GhostPillButton("取消", dismiss, Modifier.padding(top = 6.dp).fillMaxWidth())
}
