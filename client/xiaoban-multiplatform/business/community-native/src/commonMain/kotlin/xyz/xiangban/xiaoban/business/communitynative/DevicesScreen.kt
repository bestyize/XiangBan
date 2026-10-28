package xyz.xiangban.xiaoban.business.communitynative

import androidx.compose.foundation.background
import androidx.compose.foundation.border
import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.rememberScrollState
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.foundation.verticalScroll
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.OutlinedButton
import androidx.compose.material3.OutlinedTextField
import androidx.compose.material3.Text
import androidx.compose.material3.TextButton
import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.collectAsState
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import xyz.xiangban.xiaoban.common.link.PeerSummary

/**
 * 设备管理页：
 * - 查看已配对的电脑列表（名称、ID、在线/离线状态）
 * - 解除电脑配对
 * - 绑定新电脑（输入或粘贴电脑端生成的配对串 `xbl1:...`）
 */
@Composable
fun DevicesScreen(env: CommunityEnv, back: () -> Unit) {
    val remoteState by env.remoteChat.state.collectAsState()
    var pairInput by remember { mutableStateOf("") }
    var unpairTarget by remember { mutableStateOf<PeerSummary?>(null) }

    LaunchedEffect(Unit) {
        env.remoteChat.refreshPeers()
    }

    Column(Modifier.fillMaxSize()) {
        PageHeader("我的电脑", onBack = back)
        Column(
            Modifier
                .fillMaxSize()
                .verticalScroll(rememberScrollState())
                .padding(Gutter),
            verticalArrangement = Arrangement.spacedBy(16.dp),
        ) {
            // 已配对电脑列表
            Text("已连接的电脑", fontSize = 15.sp, fontWeight = FontWeight.SemiBold)
            if (remoteState.peers.isEmpty()) {
                Box(
                    Modifier
                        .fillMaxWidth()
                        .clip(CardShape)
                        .background(MaterialTheme.colorScheme.surface)
                        .padding(20.dp),
                    contentAlignment = Alignment.Center,
                ) {
                    Text("暂未绑定电脑，完成配对后可在手机上遥控电脑小伴", color = LocalExtraColors.current.ink3, fontSize = 13.sp)
                }
            } else {
                remoteState.peers.forEach { peer ->
                    PeerCard(
                        peer = peer,
                        onUnpair = { unpairTarget = peer },
                    )
                }
            }

            // 配对新电脑
            Text("配对新电脑", fontSize = 15.sp, fontWeight = FontWeight.SemiBold)
            Column(
                Modifier
                    .fillMaxWidth()
                    .clip(CardShape)
                    .background(MaterialTheme.colorScheme.surface)
                    .padding(16.dp),
                verticalArrangement = Arrangement.spacedBy(12.dp),
            ) {
                Text(
                    "在电脑端小伴中打开「设置 → 连接与网络」，点击「连接手机」生成二维码，或点击「复制配对串」并粘贴在下方：",
                    color = MaterialTheme.colorScheme.onSurfaceVariant,
                    fontSize = 13.sp,
                    lineHeight = 20.sp,
                )

                OutlinedTextField(
                    value = pairInput,
                    onValueChange = {
                        pairInput = it.trim()
                        env.remoteChat.clearPairingError()
                    },
                    modifier = Modifier.fillMaxWidth(),
                    label = { Text("配对串（xbl1:...）") },
                    placeholder = { Text("粘贴 xbl1:... 开头的配对内容") },
                    maxLines = 3,
                )

                if (remoteState.pairingError != null) {
                    Text(remoteState.pairingError.orEmpty(), color = Brand, fontSize = 13.sp)
                }

                Row(
                    Modifier.fillMaxWidth(),
                    horizontalArrangement = Arrangement.spacedBy(8.dp),
                    verticalAlignment = Alignment.CenterVertically,
                ) {
                    BrandButton(
                        text = if (remoteState.pairingBusy) "配对中…" else "确认配对",
                        onClick = {
                            if (pairInput.isNotBlank()) {
                                env.remoteChat.pair(pairInput)
                                pairInput = ""
                            }
                        },
                        enabled = !remoteState.pairingBusy && pairInput.startsWith("xbl1:"),
                        modifier = Modifier.weight(1f),
                    )
                }

                Text(
                    "安全提示：配对要求手机与电脑登录同一小伴账号。通信基于 P-256 ECDH 端到端加密，中转服务无法解密你的对话正文。",
                    color = LocalExtraColors.current.ink3,
                    fontSize = 11.5.sp,
                    lineHeight = 17.sp,
                )
            }
        }
    }

    // 解除配对确认弹窗
    unpairTarget?.let { peer ->
        Box(
            Modifier
                .fillMaxSize()
                .background(MaterialTheme.colorScheme.background.copy(alpha = 0.5f))
                .clickable(indication = null, interactionSource = null) { unpairTarget = null },
            contentAlignment = Alignment.Center,
        ) {
            Column(
                Modifier
                    .padding(24.dp)
                    .clip(CardShape)
                    .background(MaterialTheme.colorScheme.surface)
                    .border(0.5.dp, LocalExtraColors.current.hairline, CardShape)
                    .padding(20.dp),
                verticalArrangement = Arrangement.spacedBy(14.dp),
            ) {
                Text("解除配对确认", fontSize = 16.sp, fontWeight = FontWeight.SemiBold)
                Text("确定要解除与「${peer.name}」的配对吗？解除后手机将无法再遥控该电脑。", fontSize = 14.sp)
                Row(Modifier.fillMaxWidth(), horizontalArrangement = Arrangement.End) {
                    TextButton(onClick = { unpairTarget = null }) { Text("取消") }
                    TextButton(onClick = {
                        val id = peer.id
                        unpairTarget = null
                        env.remoteChat.unpair(id)
                    }) { Text("解除配对", color = Brand) }
                }
            }
        }
    }
}

@Composable
private fun PeerCard(peer: PeerSummary, onUnpair: () -> Unit) {
    Row(
        Modifier
            .fillMaxWidth()
            .clip(CardShape)
            .background(MaterialTheme.colorScheme.surface)
            .border(0.5.dp, LocalExtraColors.current.hairline, CardShape)
            .padding(16.dp),
        horizontalArrangement = Arrangement.SpaceBetween,
        verticalAlignment = Alignment.CenterVertically,
    ) {
        Column(Modifier.weight(1f), verticalArrangement = Arrangement.spacedBy(4.dp)) {
            Row(verticalAlignment = Alignment.CenterVertically, horizontalArrangement = Arrangement.spacedBy(8.dp)) {
                Text(peer.name, fontSize = 15.sp, fontWeight = FontWeight.SemiBold)
                Box(
                    Modifier
                        .clip(RoundedCornerShape(999.dp))
                        .background(if (peer.online) MaterialTheme.colorScheme.primaryContainer else MaterialTheme.colorScheme.surfaceVariant)
                        .padding(horizontal = 8.dp, vertical = 2.dp),
                ) {
                    Text(
                        if (peer.online) "在线" else "离线",
                        fontSize = 11.sp,
                        color = if (peer.online) Brand else LocalExtraColors.current.ink3,
                        fontWeight = FontWeight.Medium,
                    )
                }
            }
            MetaText("设备编号：${peer.id}")
        }
        OutlinedButton(
            onClick = onUnpair,
            shape = CardShape,
        ) {
            Text("移除", fontSize = 12.sp, color = Brand)
        }
    }
}
