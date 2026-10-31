package xyz.xiangban.xiaoban.common.community

/** 每个非空行是一个选项；与服务端相同，按 Unicode 字符而不是 UTF-16 长度限制。 */
fun parsePollOptions(text: String): List<String> = text.lines().map { it.trim() }.filter { it.isNotEmpty() }

fun pollDraftError(options: List<String>, days: Int?): String? = when {
    options.size !in 2..6 -> "投票需要 2～6 个选项，每行填写一个"
    options.any { it.isBlank() || it.count { c -> !c.isLowSurrogate() } > 40 } -> "每个选项需要 1～40 个字"
    options.map { it.trim() }.distinct().size != options.size -> "投票选项不能重复"
    days != null && days !in 1..30 -> "投票时长为 1～30 天"
    else -> null
}
