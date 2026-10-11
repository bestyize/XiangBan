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
import xyz.xiaban.wallpaper.data.AccountClient
import xyz.xiaban.wallpaper.data.CodeLoginResult
import xyz.xiaban.wallpaper.data.LoginOutcome
import xyz.xiaban.wallpaper.data.UsernameCheck
import xyz.xiaban.wallpaper.data.AccountException
import xyz.xiaban.wallpaper.data.AccountRules
import xyz.xiaban.wallpaper.data.AccountSession
import xyz.xiaban.wallpaper.data.AccountStore
import xyz.xiaban.wallpaper.data.Announcement
import xyz.xiaban.wallpaper.data.AppUpdate
import xyz.xiaban.wallpaper.data.Category
import xyz.xiaban.wallpaper.data.DownloadStore
import xyz.xiaban.wallpaper.data.Downloaded
import xyz.xiaban.wallpaper.data.Downloader
import xyz.xiaban.wallpaper.data.Favorite
import xyz.xiaban.wallpaper.data.MediaKind
import xyz.xiaban.wallpaper.data.FavoriteStore
import xyz.xiaban.wallpaper.data.GalleryWriter
import xyz.xiaban.wallpaper.data.SeenAnnouncements
import xyz.xiaban.wallpaper.data.SiteClient
import xyz.xiaban.wallpaper.data.SiteItem
import xyz.xiaban.wallpaper.data.Stage
import xyz.xiaban.wallpaper.data.UpdateRules
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

    private val accountClient = AccountClient()
    private val accountStore = AccountStore(File(app.filesDir, "account.json"))
    private val seenAnnouncements = SeenAnnouncements(File(app.filesDir, "announcements-seen.json"))
    private val installedVersion = runCatching { app.packageManager.getPackageInfo(app.packageName, 0).versionName }.getOrNull().orEmpty()

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

    /** The signed-in user, or null. Restored from disk at start-up. */
    private val _account = MutableStateFlow<AccountSession?>(null)
    val account: StateFlow<AccountSession?> = _account

    private val _loginOpen = MutableStateFlow(false)
    val loginOpen: StateFlow<Boolean> = _loginOpen

    /** A release newer than the installed one, or null when this is the newest or the check has not answered. */
    private val _update = MutableStateFlow<AppUpdate?>(null)
    val update: StateFlow<AppUpdate?> = _update

    /** The website's page with the downloads, where an update is fetched. */
    val downloadsLink: String get() = UpdateRules.downloadsLink(site.origin)

    /** The notice open as a dialog right now, if any. The rest wait their turn in [waiting]. */
    private val _announcement = MutableStateFlow<Announcement?>(null)
    val announcement: StateFlow<Announcement?> = _announcement
    private val waiting = ArrayDeque<Announcement>()

    private val _messages = MutableSharedFlow<String>(extraBufferCapacity = 8)
    val messages: SharedFlow<String> = _messages

    init {
        viewModelScope.launch(Dispatchers.IO) {
            _favorites.value = favoriteStore.list()
            _saved.value = downloadStore.list()
        }
        loadCategories()
        restoreAccount()
        checkUpdate()
        loadAnnouncements()
    }

    /** Asked once per launch. A failed check says nothing: the red dot just does not appear. */
    private fun checkUpdate() {
        if (installedVersion.isEmpty()) return
        viewModelScope.launch {
            runCatching { withContext(Dispatchers.IO) { site.update(installedVersion) } }.onSuccess { _update.value = it }
        }
    }

    /** Fetches the notices once per launch and opens the first one that is due; the others follow as each is closed. */
    private fun loadAnnouncements() {
        viewModelScope.launch {
            val items = runCatching { withContext(Dispatchers.IO) { site.announcements() } }.getOrNull() ?: return@launch
            val due = withContext(Dispatchers.IO) { UpdateRules.toShow(items, seenAnnouncements::has) }
            waiting.addAll(due)
            if (_announcement.value == null) showNextAnnouncement()
        }
    }

    /** Closes the open notice, whichever button did it, and opens the next one waiting. */
    fun dismissAnnouncement() {
        showNextAnnouncement()
    }

    private fun showNextAnnouncement() {
        val next = waiting.removeFirstOrNull()
        _announcement.value = next
        // Counted as shown the moment it appears, so a skippable notice never comes twice even if the app is killed on it.
        if (next != null && next.skippable) viewModelScope.launch(Dispatchers.IO) { seenAnnouncements.add(next.id, next.revision) }
    }

    fun openLogin() {
        _loginOpen.value = true
    }

    fun closeLogin() {
        _loginOpen.value = false
    }

    /** Asks the account centre to send a code. Returns the seconds to wait before asking again. */
    suspend fun sendLoginCode(identifier: String, resetPassword: Boolean = false): Int =
        withContext(Dispatchers.IO) { accountClient.sendCode(identifier, resetPassword) }

    /**
     * Checks the code. An existing account is signed in right away; a new email or phone number comes back as
     * [CodeLoginResult.RegistrationRequired] and has to pick a username and password through [register].
     * Throws [AccountException] with a sentence to show.
     */
    suspend fun loginWithCode(identifier: String, code: String): CodeLoginResult {
        val result = withContext(Dispatchers.IO) { accountClient.loginWithCode(identifier, code) }
        if (result is CodeLoginResult.SignedIn) signedIn(result.outcome)
        return result
    }

    suspend fun loginWithPassword(identifier: String, password: String) {
        signedIn(withContext(Dispatchers.IO) { accountClient.loginWithPassword(identifier, password) })
    }

    suspend fun register(registrationToken: String, username: String, password: String) {
        signedIn(withContext(Dispatchers.IO) { accountClient.register(registrationToken, username, password) })
    }

    suspend fun resetPassword(identifier: String, code: String, newPassword: String) {
        withContext(Dispatchers.IO) { accountClient.resetPassword(identifier, code, newPassword) }
    }

    suspend fun checkUsername(username: String): UsernameCheck = withContext(Dispatchers.IO) { accountClient.checkUsername(username) }

    private suspend fun signedIn(outcome: LoginOutcome) {
        withContext(Dispatchers.IO) { accountStore.save(outcome.session) }
        _account.value = outcome.session
        _loginOpen.value = false
        val name = outcome.session.user.username
        _messages.tryEmit(if (outcome.isNewUser) "注册成功，欢迎你，$name" else "欢迎回来，$name")
    }

    /** Forgets the login on this phone right away, then asks the account centre to revoke it as a best effort. */
    fun logout() {
        val session = _account.value ?: return
        _account.value = null
        _messages.tryEmit("已退出登录")
        viewModelScope.launch(Dispatchers.IO) {
            accountStore.clear()
            runCatching { accountClient.logout(session.refreshToken) }
        }
    }

    /**
     * Brings the saved login back and renews it when it is about to lapse. Only an answer from the account centre
     * saying the login is no longer valid signs the user out; being offline keeps it.
     */
    private fun restoreAccount() {
        viewModelScope.launch {
            val saved = withContext(Dispatchers.IO) { accountStore.load() } ?: return@launch
            _account.value = saved
            if (!AccountRules.needsRefresh(saved, AccountClient.nowSeconds())) return@launch
            try {
                val renewed = withContext(Dispatchers.IO) { accountClient.refresh(saved).also { accountStore.save(it) } }
                _account.value = renewed
            } catch (e: AccountException) {
                if (e.sessionExpired) {
                    _account.value = null
                    withContext(Dispatchers.IO) { accountStore.clear() }
                    _messages.tryEmit(AccountClient.SESSION_EXPIRED)
                }
            }
        }
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
