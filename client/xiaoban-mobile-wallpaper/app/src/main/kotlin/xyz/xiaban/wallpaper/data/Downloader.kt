package xyz.xiaban.wallpaper.data

import java.io.File
import java.io.IOException
import java.util.concurrent.CancellationException

enum class Stage { REQUESTING, DOWNLOADING, SAVING }

/** The website, as far as downloading is concerned. */
interface Remote {
    /** [preferSource] asks the site to hand over the provider's link and not to build its own copy first. */
    fun delivery(id: String, preferSource: Boolean): Delivery

    /** Our stored copy: size and SHA-256 must match the manifest or the file is discarded. */
    fun downloadCopy(copy: CopyFile, target: File, progress: (Long, Long) -> Unit, cancelled: () -> Boolean)

    /** The provider's file: only the size limit applies; what it really is gets checked by the caller. */
    fun downloadSource(
        source: SourceFile,
        target: File,
        onTotal: (Long) -> Unit,
        progress: (Long, Long) -> Unit,
        cancelled: () -> Boolean,
    ): Verified.Result
}

/**
 * Brings one wallpaper into the gallery. It uses the provider's own link when the site offers one and falls back to
 * the site's verified copy when that fails, so a blocked or expired link never costs the user the download.
 * Blocking, so call it off the main thread.
 */
class Downloader(
    private val remote: Remote,
    private val gallery: GallerySink,
    private val downloads: DownloadStore,
    private val tempDir: File,
    private val probe: (File, MediaKind) -> MediaProbe.Size = MediaProbe::check,
) {
    class Result(val entry: Downloaded, val alreadySaved: Boolean)

    fun run(
        item: SiteItem,
        onStage: (Stage) -> Unit,
        onSize: (Long) -> Unit,
        onProgress: (Float) -> Unit,
        cancelled: () -> Boolean,
        now: () -> Long = System::currentTimeMillis,
    ): Result {
        downloads.list().firstOrNull { it.id == item.id }?.let { return Result(it, alreadySaved = true) }

        onStage(Stage.REQUESTING)
        val delivery = remote.delivery(item.id, preferSource = true)
        delivery.copy?.let { onSize(it.bytes) }

        var failure: Exception? = null
        for (attempt in DownloadPlan.attempts(delivery)) {
            try {
                onStage(Stage.DOWNLOADING)
                return when (attempt) {
                    is Attempt.Source -> fromSource(item, delivery, attempt.file, onStage, onSize, onProgress, cancelled, now)
                    is Attempt.Copy -> fromCopy(item, delivery, attempt.file, onStage, onProgress, cancelled, now)
                    Attempt.AskForCopy -> {
                        onStage(Stage.REQUESTING)
                        val copy = remote.delivery(item.id, preferSource = false).copy ?: throw SiteException("网站没有提供可下载的文件")
                        onSize(copy.bytes)
                        onStage(Stage.DOWNLOADING)
                        fromCopy(item, delivery, copy, onStage, onProgress, cancelled, now)
                    }
                }
            } catch (e: CancellationException) {
                throw e
            } catch (e: SiteException) {
                failure = e
            } catch (e: IOException) {
                failure = SiteException("网络出错：${e.message ?: "连接中断"}")
            }
        }
        // The last way to be tried is the site's own copy, so its reason is the one worth showing.
        throw failure ?: SiteException("网站没有提供可下载的文件")
    }

    private fun fromSource(
        item: SiteItem, delivery: Delivery, source: SourceFile,
        onStage: (Stage) -> Unit, onSize: (Long) -> Unit, onProgress: (Float) -> Unit, cancelled: () -> Boolean, now: () -> Long,
    ): Result = withTemp { temp ->
        val got = remote.downloadSource(source, temp, onSize, { done, total -> onProgress(fraction(done, total)) }, cancelled)
        downloads.findByHash(got.sha256)?.let { return@withTemp Result(it, alreadySaved = true) }
        finish(item, delivery, temp, got.sha256, onStage, now)
    }

    private fun fromCopy(
        item: SiteItem, delivery: Delivery, copy: CopyFile,
        onStage: (Stage) -> Unit, onProgress: (Float) -> Unit, cancelled: () -> Boolean, now: () -> Long,
    ): Result {
        downloads.findByHash(copy.sha256)?.let { return Result(it, alreadySaved = true) }
        return withTemp { temp ->
            remote.downloadCopy(copy, temp, { done, total -> onProgress(fraction(done, total)) }, cancelled)
            finish(item, delivery, temp, copy.sha256, onStage, now)
        }
    }

    /** The file's own bytes decide what it is: a link that answers with a web page or the wrong kind is not used. */
    private fun finish(item: SiteItem, delivery: Delivery, file: File, sha256: String, onStage: (Stage) -> Unit, now: () -> Long): Result {
        val mime = MediaSniffer.mime(file) ?: throw SiteException("下载到的不是图片或视频，已丢弃")
        val kind = MediaRules.kindOf(mime) ?: throw SiteException("不支持的文件格式")
        if (kind != delivery.kind) throw SiteException("下载到的文件类型与预期不符，已丢弃")
        val size = probe(file, kind)

        onStage(Stage.SAVING)
        val extension = MediaRules.extension(mime) ?: throw SiteException("不支持的文件格式")
        val title = delivery.title.ifBlank { item.title }
        val uri = gallery.save(file, kind, mime, GalleryNaming.fileName(title, sha256, extension))
        val entry = Downloaded(item.id, kind, title, uri, sha256, size.width, size.height, now())
        downloads.add(entry)
        return Result(entry, alreadySaved = false)
    }

    private fun fraction(done: Long, total: Long) = if (total > 0) (done.toFloat() / total).coerceIn(0f, 1f) else 0f

    private inline fun <T> withTemp(block: (File) -> T): T {
        tempDir.mkdirs()
        val temp = File.createTempFile("part", ".tmp", tempDir)
        try {
            return block(temp)
        } finally {
            temp.delete()
        }
    }
}
