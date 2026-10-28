package xyz.xiangban.xiaoban.common.community

import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.Job
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.asStateFlow
import kotlinx.coroutines.launch

/** 单项内容（详情页）的加载状态。 */
sealed interface Load<out T> {
    data object Loading : Load<Nothing>
    data class Ready<T>(val value: T) : Load<T>
    data class Failed(val message: String, val notFound: Boolean = false) : Load<Nothing>
}

fun messageOf(e: Throwable): String = (e as? ApiException)?.message ?: "加载失败，请稍后重试"

/** 加载一项内容并保持状态；[reload] 可重试。同一时间只认最后一次请求的结果。 */
class Loader<T>(private val scope: CoroutineScope, private val fetch: suspend () -> T) {
    private val _state = MutableStateFlow<Load<T>>(Load.Loading)
    val state: StateFlow<Load<T>> = _state.asStateFlow()
    private var job: Job? = null

    fun reload() {
        job?.cancel()
        _state.value = Load.Loading
        job = scope.launch {
            _state.value = try {
                Load.Ready(fetch())
            } catch (e: ApiException) {
                Load.Failed(e.message, notFound = e.status == 404)
            } catch (e: Exception) {
                if (e is kotlinx.coroutines.CancellationException) throw e
                Load.Failed(messageOf(e))
            }
        }
    }
}

/** 社区首页信息流：二级 Tab（全部、周刊、项目、各内容类型），分页加载，切换 Tab 时丢弃旧请求的结果。 */
data class FeedState(
    val tab: String = "",
    val packs: List<Pack> = emptyList(),
    val weekly: List<WeeklyIssue> = emptyList(),
    val projects: List<Project> = emptyList(),
    val loading: Boolean = false,
    val error: String? = null,
    val hasMore: Boolean = false,
) {
    val isEmpty: Boolean get() = !loading && error == null && packs.isEmpty() && weekly.isEmpty() && projects.isEmpty()
}

class FeedController(private val api: ForumApi, private val scope: CoroutineScope) {
    private val _state = MutableStateFlow(FeedState())
    val state: StateFlow<FeedState> = _state.asStateFlow()
    private var page = 1
    private var generation = 0

    fun select(tab: String) {
        if (tab == _state.value.tab && _state.value.packs.isNotEmpty()) return
        _state.value = FeedState(tab = tab)
        refresh()
    }

    fun refresh() {
        val tab = _state.value.tab
        page = 1
        _state.value = FeedState(tab = tab)
        load(replace = true)
    }

    fun loadMore() {
        val s = _state.value
        if (s.loading || !s.hasMore) return
        page += 1
        load(replace = false)
    }

    private fun load(replace: Boolean) {
        val mine = ++generation
        val tab = _state.value.tab
        val requested = page
        _state.value = _state.value.copy(loading = true, error = null)
        scope.launch {
            try {
                val next: FeedState = when (tab) {
                    "weekly" -> api.weeklyIssues(requested).let { p -> _state.value.copy(weekly = (if (replace) emptyList() else _state.value.weekly) + p.items, hasMore = p.hasMore) }
                    "projects" -> _state.value.copy(projects = api.projects().items, hasMore = false)
                    else -> api.packs(kind = tab.ifEmpty { null }, page = requested).let { p -> _state.value.copy(packs = (if (replace) emptyList() else _state.value.packs) + p.items, hasMore = p.hasMore) }
                }
                if (mine == generation) _state.value = next.copy(loading = false, error = null)
            } catch (e: kotlinx.coroutines.CancellationException) {
                throw e
            } catch (e: Exception) {
                if (mine == generation) {
                    if (!replace) page -= 1
                    _state.value = _state.value.copy(loading = false, error = messageOf(e))
                }
            }
        }
    }
}

/** 讨论列表。 */
data class DiscussState(val topics: List<Topic> = emptyList(), val loading: Boolean = false, val error: String? = null, val hasMore: Boolean = false, val sort: String = "latest")

class DiscussController(private val api: ForumApi, private val scope: CoroutineScope) {
    private val _state = MutableStateFlow(DiscussState())
    val state: StateFlow<DiscussState> = _state.asStateFlow()
    private var page = 1
    private var generation = 0

    fun select(sort: String) {
        _state.value = DiscussState(sort = sort)
        page = 1
        load(true)
    }

    fun refresh() = select(_state.value.sort)

    fun loadMore() {
        val s = _state.value
        if (s.loading || !s.hasMore) return
        page += 1
        load(false)
    }

    private fun load(replace: Boolean) {
        val mine = ++generation
        val requested = page
        val sort = _state.value.sort
        _state.value = _state.value.copy(loading = true, error = null)
        scope.launch {
            try {
                val p = api.topics(sort = sort, page = requested)
                if (mine == generation) _state.value = _state.value.copy(topics = (if (replace) emptyList() else _state.value.topics) + p.items, hasMore = p.hasMore, loading = false)
            } catch (e: kotlinx.coroutines.CancellationException) {
                throw e
            } catch (e: Exception) {
                if (mine == generation) {
                    if (!replace) page -= 1
                    _state.value = _state.value.copy(loading = false, error = messageOf(e))
                }
            }
        }
    }
}
