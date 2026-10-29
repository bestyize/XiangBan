package xyz.xiangban.xiaoban.common.link

import xyz.xiangban.xiaoban.common.community.AssistantReply
import xyz.xiangban.xiaoban.common.community.AttentionNotice
import xyz.xiangban.xiaoban.common.community.CallAssistant
import xyz.xiangban.xiaoban.common.community.EngineUnavailable
import xyz.xiangban.xiaoban.common.community.RecognizerEngine
import kotlinx.coroutines.delay
import kotlinx.coroutines.withTimeoutOrNull

/** 语音通话里把话交给电脑上的小伴：文字发过去，回答文字流回来由手机念。 */
class RemoteAssistant(private val chat: RemoteChatController) : CallAssistant {
    override fun unavailableReason(): String? = if (chat.state.value.isPeerOnline) null else "设备不在线，没法通话"
    override fun ask(text: String) = chat.send(text)
    override fun askWithNote(text: String, note: String) = chat.sendVoiceText(text, note)
    override fun cancel() { if (chat.state.value.sending) chat.cancel() }
    override fun reply(): AssistantReply {
        val state = chat.state.value
        val turn = state.messages.lastOrNull { it.role == "assistant" }
        return AssistantReply(
            turn?.text.orEmpty(), busy = state.sending || turn?.streaming == true,
            error = turn?.error?.takeIf { it != TurnCancelled },
            attention = if (state.pendingApproval != null) AttentionNotice else null,
        )
    }
}

/** 手机本地识别用不了时的备选：把录音传给电脑，用电脑上的 SenseVoice 识别。 */
class DesktopRecognizer(private val chat: RemoteChatController) : RecognizerEngine {
    override val name = "电脑上的语音识别"
    override suspend fun recognize(pcm: ByteArray): String {
        if (!chat.state.value.isPeerOnline) throw EngineUnavailable("设备不在线")
        chat.sendVoice(pcm, speakReply = false)
        val done = withTimeoutOrNull(40_000) {
            while (chat.state.value.voicePhase != "idle") delay(50)
            true
        } ?: throw EngineUnavailable("电脑没有回应")
        check(done)
        val state = chat.state.value
        state.voiceText?.let { chat.consumeVoiceText(); return it }
        val error = state.error.orEmpty()
        if (error.contains("语音识别") && error.contains("开启")) throw EngineUnavailable("电脑上没有开启语音识别")
        return ""
    }
}
