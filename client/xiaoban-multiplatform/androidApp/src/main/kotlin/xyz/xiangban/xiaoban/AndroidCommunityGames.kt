package xyz.xiangban.xiaoban

import android.content.Context
import kotlinx.coroutines.flow.MutableStateFlow
import xyz.xiangban.xiaoban.common.community.CommunityGameHistory
import xyz.xiangban.xiaoban.common.community.RecentCommunityGame

/** 最近试玩过的社区作品，只存在这台手机上，不保存下载凭证或作品内容。 */
internal object AndroidCommunityGames {
    private val history = MutableStateFlow<List<RecentCommunityGame>>(emptyList())
    private var initialized = false

    @Synchronized fun recent(context: Context): MutableStateFlow<List<RecentCommunityGame>> {
        if (!initialized) {
            history.value = CommunityGameHistory.decode(context.getSharedPreferences("community_game_history", Context.MODE_PRIVATE).getString("recent", null))
            initialized = true
        }
        return history
    }

    @Synchronized fun opened(context: Context, recentGame: RecentCommunityGame) {
        recent(context)
        history.value = CommunityGameHistory.opened(history.value, recentGame)
        context.getSharedPreferences("community_game_history", Context.MODE_PRIVATE).edit()
            .putString("recent", CommunityGameHistory.encode(history.value)).apply()
    }
}
