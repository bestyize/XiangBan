package xyz.xiangban.xiaoban.common.community

/** 按住说话时，手指松开会怎样：发送、转成文字放进输入框，或取消。 */
enum class PttAction { Send, Text, Cancel }

/**
 * 按住说话的规则（设计规范第 8 节）：最长 20 秒，最后 10 秒倒计时；不到 0.5 秒算太短；
 * 手指在按钮上就是发送，往上滑超过一段距离是取消，滑到右边是转文字。
 */
object Ptt {
    const val MaxMillis = 20_000L
    const val CountdownFromMillis = 10_000L
    const val MinMillis = 500L

    /**
     * [x]、[y] 是手指在按钮里的位置，[width] 是按钮宽度，[startY] 是刚按下时的纵坐标，
     * [upThreshold] 是往上滑多远才算离开按钮（都是同一单位）。
     */
    fun action(x: Float, y: Float, width: Float, startY: Float, upThreshold: Float): PttAction = when {
        startY - y < upThreshold -> PttAction.Send
        width > 0f && x > width * 0.62f -> PttAction.Text
        else -> PttAction.Cancel
    }

    fun tooShort(elapsedMillis: Long): Boolean = elapsedMillis < MinMillis

    /** 还剩几秒（向上取整）；还没到倒计时的时候是 null。 */
    fun secondsLeft(elapsedMillis: Long): Int? =
        if (elapsedMillis < CountdownFromMillis) null else ((MaxMillis - elapsedMillis).coerceAtLeast(0L) + 999L).toInt() / 1000

    fun expired(elapsedMillis: Long): Boolean = elapsedMillis >= MaxMillis

    /** 录音时间显示成 0:06 的样子。 */
    fun clock(elapsedMillis: Long): String {
        val s = (elapsedMillis / 1000).toInt()
        return "${s / 60}:${(s % 60).toString().padStart(2, '0')}"
    }
}
