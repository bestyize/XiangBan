package xyz.xiaban.wallpaper.ui

import android.app.Application
import androidx.lifecycle.AndroidViewModel
import androidx.lifecycle.viewModelScope
import kotlinx.coroutines.CancellationException
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.Job
import kotlinx.coroutines.currentCoroutineContext
import kotlinx.coroutines.flow.MutableSharedFlow
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.SharedFlow
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.update
import kotlinx.coroutines.isActive
import kotlinx.coroutines.launch
import kotlinx.coroutines.withContext
import xyz.xiaban.wallpaper.data.Category
import xyz.xiaban.wallpaper.data.DownloadStore
import xyz.xiaban.wallpaper.data.Downloaded
import xyz.xiaban.wallpaper.data.Downloader
import xyz.xiaban.wallpaper.data.Favorite
import xyz.xiaban.wallpaper.data.MediaKind
import xyz.xiaban.wallpaper.data.FavoriteStore
import xyz.xiaban.wallpaper.data.GalleryWriter
import xyz.xiaban.wallpaper.data.SiteClient
import xyz.xiaban.wallpaper.data.SiteItem
import xyz.xiaban.wallpaper.data.Stage
import java.io.File

/** What a save-to-gallery request is doing right now. */
sealed interface Transfer {
    class Active(val stage: Stage, val progress: Float, val totalBytes: Long = 0) : Transfer
    class Failed(val message: String) : Transfer
}

class AppViewModel(app: Application) : AndroidViewModel(app) {
    val site = SiteClient()

    private val gallery = GalleryWriter(app.contentResolver)
    private val favoriteStore = FavoriteStore(File(app.filesDir, "favorites.json"))
    private val downloadStore = DownloadStore(File(app.filesDir, "downloads.json")) { gallery.exists(it) }
    private val downloader = Downloader(site, gallery, downloadStore, File(app.cacheDir, "downloads"))
    private val jobs = HashMap<String, Job>()

    private val _favorites = MutableStateFlow<List<Favorite>>(emptyList())
    val favorites: StateFlow<List<Favorite>> = _favorites

    private val _saved = MutableStateFlow<List<Downloaded>>(emptyList())
    val saved: StateFlow<List<Downloaded>> = _saved

    private val _categories = MutableStateFlow<List<Category>>(emptyList())
    val categories: StateFlow<List<Category>> = _categories

    private val _transfers = MutableStateFlow<Map<String, Transfer>>(emptyMap())
    val transfers: StateFlow<Map<String, Transfer>> = _transfers

    /** The wallpaper whose detail page is open, if any. */
    private val _detail = MutableStateFlow<SiteItem?>(null)
    val detail: StateFlow<SiteItem?> = _detail

    /** What is open in the full-screen viewer, if anything. */
    private val _viewer = MutableStateFlow<Viewable?>(null)
    val viewer: StateFlow<Viewable?> = _viewer

    private val _messages = MutableSharedFlow<String>(extraBufferCapacity = 8)
    val messages: SharedFlow<String> = _messages

    init {
        viewModelScope.launch(Dispatchers.IO) {
            _favorites.value = favoriteStore.list()
            _saved.value = downloadStore.list()
        }
        loadCategories()
    }

    fun loadCategories() {
        if (_categories.value.isNotEmpty()) return
        viewModelScope.launch {
            runCatching { withContext(Dispatchers.IO) { site.categories() } }.onSuccess { _categories.value = it }
        }
    }

    fun open(item: SiteItem) {
        _detail.value = item
    }

    fun close() {
        _detail.value = null
    }

    /** Opens the viewer; a video the phone cannot stream says so instead of opening a player that cannot play. */
    fun view(item: Viewable) {
        if (item.kind == MediaKind.VIDEO && !item.playable) {
            _messages.tryEmit("这个视频暂不支持在线播放，保存到相册后在相册里观看")
            return
        }
        _viewer.value = item
    }

    fun closeViewer() {
        _viewer.value = null
    }

    fun isFavorite(id: String) = _favorites.value.any { it.item.id == id }

    fun toggleFavorite(item: SiteItem) {
        viewModelScope.launch {
            val nowFavorite = withContext(Dispatchers.IO) { favoriteStore.toggle(item) }
            _favorites.value = withContext(Dispatchers.IO) { favoriteStore.list() }
            _messages.tryEmit(if (nowFavorite) "已收藏，仅保存在这台手机上" else "已取消收藏")
        }
    }

    fun save(item: SiteItem) {
        if (jobs[item.id]?.isActive == true) return
        setTransfer(item.id, Transfer.Active(Stage.REQUESTING, 0f))
        jobs[item.id] = viewModelScope.launch {
            val job = currentCoroutineContext()[Job]
            var size = 0L
            try {
                val result = withContext(Dispatchers.IO) {
                    downloader.run(
                        item,
                        onStage = { setTransfer(item.id, Transfer.Active(it, 0f, size)) },
                        onSize = { size = it },
                        onProgress = { setTransfer(item.id, Transfer.Active(Stage.DOWNLOADING, it, size)) },
                        cancelled = { job?.isActive == false },
                    )
                }
                _saved.value = withContext(Dispatchers.IO) { downloadStore.list() }
                setTransfer(item.id, null)
                _messages.tryEmit(if (result.alreadySaved) "相册里已经有这张了" else "已保存到相册「小伴壁纸」")
            } catch (e: CancellationException) {
                setTransfer(item.id, null)
                throw e
            } catch (e: Exception) {
                setTransfer(item.id, Transfer.Failed(e.message ?: "保存失败"))
            }
        }
    }

    fun cancel(id: String) {
        jobs.remove(id)?.cancel()
        setTransfer(id, null)
    }

    /** Forgets a saved wallpaper and removes its gallery file when the system allows it. */
    fun deleteSaved(entry: Downloaded) {
        viewModelScope.launch {
            val removed = withContext(Dispatchers.IO) {
                val gone = gallery.delete(entry.uri)
                downloadStore.remove(entry.id)
                gone
            }
            _saved.value = withContext(Dispatchers.IO) { downloadStore.list() }
            _messages.tryEmit(if (removed) "已删除" else "已从列表移除。这张图不是本次安装保存的，请在相册里删除文件")
        }
    }

    fun say(message: String) {
        _messages.tryEmit(message)
    }

    private fun setTransfer(id: String, transfer: Transfer?) {
        _transfers.update { if (transfer == null) it - id else it + (id to transfer) }
    }
}
