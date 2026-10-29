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
 * 我的电脑：同一账号下登录了小伴电脑版并打开「允许手机遥控」的电脑会自动出现在这里，不需要配对。
 * 可以查看在线状态，也可以把不用的电脑从账号的设备列表里移除。
 */
@Composable
fun DevicesScreen(env: CommunityEnv, back: () -> Unit) {
    val remoteState by env.remoteChat.state.collectAsState()
    var removeTarget by remember { mutableStateOf<PeerSummary?>(null) }

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
            Text("同账号下的电脑", fontSize = 15.sp, fontWeight = FontWeight.SemiBold)
            if (remoteState.peers.isEmpty()) {
                Column(
                    Modifier
                        .fillMaxWidth()
                        .clip(CardShape)
                        .background(MaterialTheme.colorScheme.surface)
                        .padding(20.dp),
                    verticalArrangement = Arrangement.spacedBy(8.dp),
                ) {
                    Text("还没有找到电脑", fontSize = 14.sp, fontWeight = FontWeight.Medium)
                    Text(
                        "在电脑上登录同一个小伴账号，打开「设置 → 手机遥控」里的「允许手机遥控这台电脑」，电脑就会自动出现在这里。",
                        color = LocalExtraColors.current.ink3,
                        fontSize = 13.sp,
                        lineHeight = 20.sp,
                    )
                }
            } else {
                remoteState.peers.forEach { peer ->
                    PeerCard(peer = peer, onRemove = { removeTarget = peer })
                }
            }
            remoteState.error?.let { Text(it, color = Brand, fontSize = 13.sp) }
        }
    }

    // 移除确认弹窗
    removeTarget?.let { peer ->
        Box(
            Modifier
                .fillMaxSize()
                .background(MaterialTheme.colorScheme.background.copy(alpha = 0.5f))
                .clickable(indication = null, interactionSource = null) { removeTarget = null },
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
                Text("移除这台电脑", fontSize = 16.sp, fontWeight = FontWeight.SemiBold)
                Text("确定要把「${peer.name}」从账号的设备列表里移除吗？它再次启动并开启手机遥控后会重新出现。", fontSize = 14.sp)
                Row(Modifier.fillMaxWidth(), horizontalArrangement = Arrangement.End) {
                    TextButton(onClick = { removeTarget = null }) { Text("取消") }
                    TextButton(onClick = {
                        val id = peer.id
                        removeTarget = null
                        env.remoteChat.removePeer(id)
                    }) { Text("移除", color = Brand) }
                }
            }
        }
    }
}

@Composable
private fun PeerCard(peer: PeerSummary, onRemove: () -> Unit) {
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
        }
        OutlinedButton(
            onClick = onRemove,
            shape = CardShape,
        ) {
            Text("移除", fontSize = 12.sp, color = Brand)
        }
    }
}
