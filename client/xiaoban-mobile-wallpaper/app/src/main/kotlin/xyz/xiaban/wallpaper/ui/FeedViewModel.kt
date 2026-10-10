package xyz.xiaban.wallpaper.ui

import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.Job
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.update
import kotlinx.coroutines.launch
import kotlinx.coroutines.withContext
import xyz.xiaban.wallpaper.data.MediaKind
import xyz.xiaban.wallpaper.data.Orientation
import xyz.xiaban.wallpaper.data.SiteClient
import xyz.xiaban.wallpaper.data.SiteItem

data class FeedState(
    val items: List<SiteItem> = emptyList(),
    /** True while a page is being fetched. */
    val loading: Boolean = false,
    /** True only for a pull-to-refresh, which keeps the old items on screen meanwhile. */
    val refreshing: Boolean = false,
    val error: String? = null,
    val hasMore: Boolean = false,
    /** False until the first answer (or failure) arrives, so an empty list is not mistaken for "no results". */
    val answered: Boolean = false,
)

/** One online feed (pictures or videos) with search, category and paging. */
class FeedViewModel(private val site: SiteClient, val kind: MediaKind) : ViewModel() {
    private val _state = MutableStateFlow(FeedState())
    val state: StateFlow<FeedState> = _state

    var query = ""
        private set
    var category = ""
        private set
    var orientation = Orientation.ALL
        private set

    private var page = 0
    private var generation = 0
    private var job: Job? = null

    init {
        reload()
    }

    fun select(categoryId: String) {
        if (categoryId == category) return
        category = categoryId
        reload()
    }

    fun selectOrientation(value: Orientation) {
        if (value == orientation) return
        orientation = value
        reload()
    }

    fun search(text: String) {
        val cleaned = text.trim()
        if (cleaned == query) return
        query = cleaned
        reload()
    }

    fun refresh() = reload(pull = true)

    fun retry() {
        if (_state.value.items.isEmpty()) reload() else loadMore()
    }

    fun loadMore() {
        val s = _state.value
        if (s.loading || (!s.hasMore && s.error == null)) return
        fetch(page + 1, replace = false, pull = false)
    }

    private fun reload(pull: Boolean = false) = fetch(1, replace = true, pull = pull)

    private fun fetch(target: Int, replace: Boolean, pull: Boolean) {
        job?.cancel()
        val mine = ++generation
        _state.update {
            val base = if (replace && !pull) it.copy(items = emptyList(), hasMore = false, answered = false) else it
            base.copy(loading = true, refreshing = pull, error = null)
        }
        job = viewModelScope.launch {
            val result = runCatching { withContext(Dispatchers.IO) { site.search(query, category, kind, target, orientation) } }
            if (mine != generation) return@launch
            result.onSuccess { fetched ->
                page = fetched.page
                _state.update {
                    val merged = if (replace) fetched.items else (it.items + fetched.items).distinctBy { item -> item.id }
                    it.copy(items = merged, hasMore = fetched.hasMore, loading = false, refreshing = false, answered = true)
                }
            }.onFailure { error ->
                if (error is kotlinx.coroutines.CancellationException) throw error
                _state.update { it.copy(loading = false, refreshing = false, error = error.message ?: "加载失败", answered = true) }
            }
        }
    }
}
