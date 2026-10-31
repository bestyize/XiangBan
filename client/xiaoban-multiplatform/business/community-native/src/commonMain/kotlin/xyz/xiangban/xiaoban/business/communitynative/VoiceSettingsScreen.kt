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
import androidx.compose.foundation.rememberScrollState
import androidx.compose.foundation.shape.CircleShape
import androidx.compose.foundation.verticalScroll
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.DisposableEffect
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.saveable.rememberSaveable
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.draw.shadow
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import xyz.xiangban.xiaoban.common.community.EdgeVoices
import xyz.xiangban.xiaoban.common.community.SpeechRates
import xyz.xiangban.xiaoban.common.community.SpeechSource

/*
 * 设置 → 声音（设计稿 69、70 号）：语音识别（本地模型）、朗读（自动朗读、来源、音色、语速、试听）、通话（直接开口打断）。
 * 选项都通栏接边；音色和语速只在来源是 Edge 在线时显示。
 */

private const val SampleText = "你好，我是小伴。愿你的每一天，都有一点温暖的陪伴。"

@Composable
fun VoiceSettingsScreen(env: CommunityEnv, back: () -> Unit) {
    val s = SpeechState.settings
    val reader = env.reader
    var sheet by rememberSaveable { mutableStateOf("") }
    var modelRefresh by remember { mutableStateOf(0) }
    DisposableEffect(reader) { onDispose { reader.stop() } }
    Box(Modifier.fillMaxSize()) {
        Column(Modifier.fillMaxSize()) {
            ScreenBar("声音", onBack = back)
            Column(Modifier.weight(1f).fillMaxWidth().verticalScroll(rememberScrollState())) {
                val models = env.speechModels
                if (models != null) {
                    GroupLabel("语音识别")
                    val ready = modelRefresh >= 0 && models.ready
                    Row(Modifier.fillMaxWidth().background(MaterialTheme.colorScheme.surface).padding(horizontal = 16.dp, vertical = 14.dp), verticalAlignment = Alignment.CenterVertically, horizontalArrangement = Arrangement.spacedBy(12.dp)) {
                        Column(Modifier.weight(1f)) {
                            Text("本地语音识别", fontSize = 15.sp)
                            Text(if (ready) "SenseVoice · 在手机上识别你说的话，不上传录音" else "SenseVoice · 约 242MB，下好后手机上就能识别", Modifier.padding(top = 2.dp), fontSize = 12.sp, lineHeight = 18.sp, color = MaterialTheme.colorScheme.onSurfaceVariant)
                        }
                        if (ready) {
                            Row(verticalAlignment = Alignment.CenterVertically, horizontalArrangement = Arrangement.spacedBy(4.dp)) {
                                XIcon(XIcons.Check, tint = Success, size = 16.dp)
                                Text("已就绪", fontSize = 12.sp, color = Success)
                            }
                        } else {
                            Text(
                                "下载", Modifier.height(34.dp).clip(CircleShape).background(LocalExtraColors.current.brandSoft).clickable { sheet = "model" }.padding(horizontal = 16.dp, vertical = 7.dp),
                                color = LocalExtraColors.current.brandInk, fontSize = 14.sp, fontWeight = FontWeight.SemiBold,
                            )
                        }
                    }
                }

                GroupLabel("朗读")
                SwitchRow("自动朗读回复", "回复会一边生成一边朗读；切换对话或发新消息会停止朗读。", s.autoRead) { on -> SpeechState.update { it.copy(autoRead = on) }; if (!on) reader.stop() }
                NavRow("语音来源", s.source.label) { sheet = "source" }
                if (s.source == SpeechSource.Edge) {
                    val voice = EdgeVoices.byId(s.voice)
                    NavRow("音色", "${voice.name} · ${voice.gender}") { sheet = "voice" }
                    RateRow(s.rateStep) { step -> SpeechState.update { it.copy(rateStep = step) } }
                }
                Column(Modifier.fillMaxWidth().background(MaterialTheme.colorScheme.surface)) {
                    Hairline(Modifier.padding(start = 16.dp))
                    Row(
                        Modifier.fillMaxWidth().height(52.dp).clickable { if (reader.speaking) reader.stop() else reader.speak(SampleText) },
                        horizontalArrangement = Arrangement.spacedBy(8.dp, Alignment.CenterHorizontally), verticalAlignment = Alignment.CenterVertically,
                    ) {
                        XIcon(if (reader.speaking) XIcons.Stop else XIcons.Volume, tint = Brand, size = 18.dp)
                        Text(if (reader.speaking) "停止" else "试听", color = Brand, fontSize = 15.sp, fontWeight = FontWeight.SemiBold)
                    }
                }

                GroupLabel("通话")
                SwitchRow("直接开口打断", "小伴说话时，直接开口就能打断它；周围回声大、总被误打断时可以关掉。", s.bargeIn ?: true) { on -> SpeechState.update { it.copy(bargeIn = on) } }
                Spacer(Modifier.height(24.dp))
            }
        }
        OverlayTransition(sheet == "source") { panel ->
            OptionSheet("语音来源", { sheet = "" }, panel) {
                SpeechSource.entries.forEach { source ->
                    OptionRow(source == s.source, { SpeechState.update { it.copy(source = source) }; sheet = "" }) {
                        Text(source.label + if (source == SpeechSource.Edge) "（默认）" else "（离线）", fontSize = 15.sp, fontWeight = FontWeight.SemiBold, color = if (source == s.source) Brand else MaterialTheme.colorScheme.onSurface)
                        Text(source.desc, Modifier.padding(top = 2.dp), fontSize = 12.sp, lineHeight = 18.sp, color = MaterialTheme.colorScheme.onSurfaceVariant)
                    }
                }
            }
        }
        OverlayTransition(sheet == "voice") { panel ->
            OptionSheet("音色", { sheet = "" }, panel) {
                EdgeVoices.All.forEach { voice ->
                    val on = voice.id == s.voice
                    Hairline()
                    Row(Modifier.fillMaxWidth().padding(start = 16.dp, end = 4.dp), verticalAlignment = Alignment.CenterVertically) {
                        Row(
                            Modifier.weight(1f).clickable { SpeechState.update { it.copy(voice = voice.id) }; sheet = "" }.padding(vertical = 14.dp),
                            verticalAlignment = Alignment.CenterVertically, horizontalArrangement = Arrangement.spacedBy(12.dp),
                        ) {
                            RadioMark(on)
                            Text(voice.name, fontSize = 15.sp, fontWeight = if (on) FontWeight.SemiBold else FontWeight.Normal, color = if (on) Brand else MaterialTheme.colorScheme.onSurface)
                            Text(voice.gender, fontSize = 12.sp, color = MaterialTheme.colorScheme.onSurfaceVariant)
                        }
                        IconTap(XIcons.Volume, "试听${voice.name}", { reader.speak(SampleText, voice.id) }, size = 20.dp, tint = MaterialTheme.colorScheme.onSurfaceVariant)
                    }
                }
            }
        }
        OverlayTransition(sheet == "model") { panel ->
            SpeechModelSheet(env, panel) { sheet = ""; modelRefresh += 1 }
        }
    }
}

/** 一行：左边标题和说明，右边开关；通栏、白底。 */
@Composable
private fun SwitchRow(title: String, desc: String, checked: Boolean, onChange: (Boolean) -> Unit) {
    Column(Modifier.fillMaxWidth().background(MaterialTheme.colorScheme.surface)) {
        Hairline(Modifier.padding(start = 16.dp))
        Row(Modifier.fillMaxWidth().padding(horizontal = 16.dp, vertical = 14.dp), verticalAlignment = Alignment.CenterVertically, horizontalArrangement = Arrangement.spacedBy(12.dp)) {
            Column(Modifier.weight(1f)) {
                Text(title, fontSize = 15.sp)
                Text(desc, Modifier.padding(top = 2.dp), fontSize = 12.sp, lineHeight = 18.sp, color = MaterialTheme.colorScheme.onSurfaceVariant)
            }
            Toggle(checked, onChange)
        }
    }
}

/** 语速：五档，点哪一档就选哪一档；选中的是白色圆钮，已经过的一段用主题色。 */
@Composable
private fun RateRow(step: Int, onChange: (Int) -> Unit) {
    val extra = LocalExtraColors.current
    Column(Modifier.fillMaxWidth().background(MaterialTheme.colorScheme.surface).padding(start = 16.dp, end = 16.dp, top = 14.dp, bottom = 12.dp)) {
        Row(verticalAlignment = Alignment.CenterVertically) {
            Text("语速", Modifier.weight(1f), fontSize = 15.sp)
            Text(SpeechRates.Names[step], fontSize = 12.sp, color = MaterialTheme.colorScheme.onSurfaceVariant)
        }
        Box(Modifier.padding(top = 10.dp).fillMaxWidth().height(28.dp), contentAlignment = Alignment.CenterStart) {
            Box(Modifier.padding(horizontal = 12.dp).fillMaxWidth().height(4.dp).clip(CircleShape).background(extra.fill)) {
                Box(Modifier.fillMaxWidth(step / 4f).height(4.dp).background(Brand))
            }
            Row(Modifier.fillMaxWidth(), horizontalArrangement = Arrangement.SpaceBetween, verticalAlignment = Alignment.CenterVertically) {
                SpeechRates.Names.indices.forEach { i ->
                    Box(Modifier.size(28.dp).clip(CircleShape).clickable { onChange(i) }, contentAlignment = Alignment.Center) {
                        if (i == step) Box(Modifier.size(24.dp).shadow(2.dp, CircleShape).background(Color.White, CircleShape))
                        else Box(Modifier.size(6.dp).clip(CircleShape).background(if (i < step) Color.White else extra.line))
                    }
                }
            }
        }
        Row(Modifier.padding(top = 4.dp).fillMaxWidth(), horizontalArrangement = Arrangement.SpaceBetween) {
            Text("慢", fontSize = 12.sp, color = extra.ink3)
            Text("快", fontSize = 12.sp, color = extra.ink3)
        }
    }
}

/** 底部选项面板：标题，通栏的选项行，灰色一道缝，再一行“取消”。 */
@Composable
private fun OptionSheet(title: String, onDismiss: () -> Unit, panel: Modifier, options: @Composable androidx.compose.foundation.layout.ColumnScope.() -> Unit) {
    BottomSheet(onDismiss, panel, flush = true) {
        Text(title, Modifier.padding(start = 16.dp, end = 16.dp, bottom = 10.dp), fontSize = 18.sp, fontWeight = FontWeight.SemiBold)
        options()
        Box(Modifier.fillMaxWidth().height(8.dp).background(MaterialTheme.colorScheme.background))
        Box(Modifier.fillMaxWidth().height(52.dp).clickable(onClick = onDismiss), contentAlignment = Alignment.Center) {
            Text("取消", fontSize = 15.sp, fontWeight = FontWeight.SemiBold)
        }
    }
}

@Composable
private fun OptionRow(selected: Boolean, onClick: () -> Unit, text: @Composable androidx.compose.foundation.layout.ColumnScope.() -> Unit) {
    Hairline()
    Row(Modifier.fillMaxWidth().clickable(onClick = onClick).padding(horizontal = 16.dp, vertical = 14.dp), verticalAlignment = Alignment.Top, horizontalArrangement = Arrangement.spacedBy(12.dp)) {
        RadioMark(selected)
        Column(Modifier.weight(1f), content = text)
    }
}

@Composable
private fun RadioMark(selected: Boolean) {
    Box(Modifier.size(20.dp).clip(CircleShape).border(2.dp, if (selected) Brand else LocalExtraColors.current.line, CircleShape), contentAlignment = Alignment.Center) {
        if (selected) Box(Modifier.size(10.dp).clip(CircleShape).background(Brand))
    }
}
