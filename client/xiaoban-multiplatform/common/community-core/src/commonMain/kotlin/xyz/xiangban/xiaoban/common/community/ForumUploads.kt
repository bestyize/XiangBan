package xyz.xiangban.xiaoban.common.community

import io.ktor.client.HttpClient
import io.ktor.client.request.forms.MultiPartFormDataContent
import io.ktor.client.request.forms.formData
import io.ktor.client.request.header
import io.ktor.client.request.post
import io.ktor.client.request.setBody
import io.ktor.client.statement.bodyAsText
import io.ktor.http.Headers
import io.ktor.http.HttpHeaders
import io.ktor.http.ContentType
import io.ktor.http.Url
import io.ktor.http.contentType
import kotlinx.coroutines.CancellationException
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.withContext
import kotlinx.serialization.SerialName
import kotlinx.serialization.Serializable
import kotlinx.serialization.decodeFromString
import kotlinx.serialization.json.buildJsonObject
import kotlinx.serialization.json.put

const val FORUM_IMAGE_MAX_BYTES = 5L * 1024 * 1024
/** 现行服务端普通用户 5 MiB、守护者 30 MiB；内存上传另有 32 MiB 硬上限。 */
const val FORUM_MEMORY_MAX_BYTES = 32L * 1024 * 1024

/** 系统选择器读出的受限内容，不向论坛传本地 URI 或文件路径。 */
data class ForumUploadFile(val name: String, val mime: String, val bytes: ByteArray)

@Serializable
data class ForumUploadSession(val user: ForumUser, val limits: ForumUploadLimits)
@Serializable
data class ForumUploadLimits(
    @SerialName("image_max_bytes") val imageMaxBytes: Long,
    @SerialName("upload_max_bytes") val uploadMaxBytes: Long,
    @SerialName("comment_images") val commentImages: Boolean = false,
)
@Serializable
data class ForumImage(val id: String, val url: String, val size: Long)
@Serializable
data class ForumAttachment(val id: String, val name: String, val size: Long, val price: Int = 0, val purchased: Boolean = false)

@Serializable
data class AttachmentPurchase(val purchased: Boolean, val charged: Int, val balance: Int)

@Serializable
private data class DirectForumTicket(val id: String, @SerialName("upload_url") val uploadUrl: String, val headers: Map<String, String>)


internal val ForumAssetId = Regex("[0-9a-fA-F]{8}-[0-9a-fA-F]{4}-[0-9a-fA-F]{4}-[0-9a-fA-F]{4}-[0-9a-fA-F]{12}")

fun forumImageId(path: String): String? =
    path.removePrefix("/api/images/").takeIf { path.startsWith("/api/images/") && ForumAssetId.matches(it) }

fun forumUploadName(raw: String): String = raw.substringAfterLast('/').substringAfterLast('\\')
    .filter { !it.isISOControl() && it != '"' && it != ';' }.trim().take(120).ifEmpty { "file" }

fun validateForumUpload(file: ForumUploadFile, maxBytes: Long, image: Boolean) {
    val limit = minOf(maxBytes, FORUM_MEMORY_MAX_BYTES, if (image) FORUM_IMAGE_MAX_BYTES else Long.MAX_VALUE)
    if (limit <= 0) throw ApiException(403, if (image) "当前账号暂不能插图" else "当前账号暂不能上传附件")
    if (file.bytes.isEmpty()) throw ApiException(400, "文件是空的")
    if (file.bytes.size > limit) throw ApiException(400, "文件超过上传上限（${limit / 1024 / 1024} MB）")
    if (image && file.mime !in listOf("image/png", "image/jpeg", "image/gif", "image/webp"))
        throw ApiException(400, "只支持 PNG、JPEG、GIF、WebP 图片")
}

/** 服务端响应必须是同一条站内图片资源，拒绝 file://、content://、第三方链接及 Markdown 注入。 */
fun forumImageMarkdown(image: ForumImage, filename: String): String {
    if (forumImageId(image.url) != image.id || image.size <= 0)
        throw ApiException(200, "图片上传结果无效，请重试")
    val clean = forumUploadName(filename)
    val alt = clean.substringBeforeLast('.', clean)
        .filter { it !in "[]()!\\" }.take(100).ifBlank { "图片" }
    return "\n![$alt](${image.url})\n"
}

fun validateTopicAttachments(attachments: List<ForumAttachment>) {
    if (attachments.size > 5 || attachments.map { it.id }.distinct().size != attachments.size)
        throw ApiException(400, "一个帖子最多 5 个不同附件")
    if (attachments.any { !ForumAssetId.matches(it.id) || it.price !in 0..10 })
        throw ApiException(400, "附件信息无效，请重新上传")
}

/** 不自动重放上传：先取会话提供的最新令牌，网络失败保留选中文件，由用户明确重试。 */
class ForumUploads(private val client: HttpClient, config: CommunityConfig, private val token: suspend () -> String?) {
    private val base = config.forumBase.trimEnd('/')
    private val resourceBase = config.resourceBase.trimEnd('/')
    private val directUploads = config.directForumUploads

    suspend fun image(file: ForumUploadFile, maxBytes: Long): ForumImage {
        validateForumUpload(file, maxBytes, true)
        val uploaded = CommunityJson.decodeFromString<ForumImage>(if (directUploads) direct("image", file) else upload("/api/images", file))
        if (uploaded.size != file.bytes.size.toLong()) throw ApiException(200, "图片上传结果无效，请重试")
        forumImageMarkdown(uploaded, file.name)
        return uploaded
    }

    suspend fun attachment(file: ForumUploadFile, maxBytes: Long): ForumAttachment {
        validateForumUpload(file, maxBytes, false)
        val uploaded = CommunityJson.decodeFromString<ForumAttachment>(if (directUploads) direct("attachment", file) else upload("/api/attachments", file))
        validateTopicAttachments(listOf(uploaded))
        if (uploaded.size <= 0 || uploaded.size != file.bytes.size.toLong()) throw ApiException(200, "附件上传结果无效，请重试")
        return uploaded
    }

    private suspend fun jsonPost(path: String, body: String, access: String): String {
        val response = try {
            client.post("$base$path") {
                header(HttpHeaders.Authorization, "Bearer $access")
                contentType(ContentType.Application.Json)
                setBody(body)
            }
        } catch (e: CancellationException) { throw e }
        catch (e: Exception) { throw ApiException(0, "上传连接失败，请重试") }
        val text = response.bodyAsText()
        if (response.status.value !in 200..299) throw parseApiError(response.status.value, text)
        return text
    }

    private suspend fun direct(kind: String, file: ForumUploadFile): String {
        val access = token() ?: throw ApiException(401, "无法取得登录凭据，请重试或重新登录")
        val hash = withContext(Dispatchers.Default) { Sha256().apply { update(file.bytes) }.hex() }
        val mime = if (kind == "image") file.mime.lowercase() else "application/octet-stream"
        val payload = buildJsonObject { put("kind", kind); put("name", forumUploadName(file.name)); put("mime", mime); put("size", file.bytes.size); put("sha256", hash) }.toString()
        val ticket = try { CommunityJson.decodeFromString<DirectForumTicket>(jsonPost("/api/resource-uploads/ticket", payload, access)) }
            catch (e: ApiException) { throw e }
            catch (e: CancellationException) { throw e }
            catch (e: Exception) { throw ApiException(0, "上传凭证无效") }
        if (!ForumAssetId.matches(ticket.id)) throw ApiException(0, "上传凭证无效")
        val target = runCatching { Url(ticket.uploadUrl) }.getOrElse { throw ApiException(0, "上传地址无效") }
        val expected = runCatching { Url(resourceBase) }.getOrElse { throw ApiException(0, "资源机地址无效") }
        val path = if (kind == "image") "/internal/forum-image/images/${ticket.id}" else "/internal/forum-file/${ticket.id}"
        if (target.protocol != expected.protocol || target.host != expected.host || target.port != expected.port ||
            target.user != null || target.password != null || target.fragment.isNotEmpty() || target.encodedPath != path || target.encodedQuery.isNotEmpty())
            throw ApiException(0, "上传地址无效")
        val allowed = setOf("Content-Type", "X-Content-Sha256", "X-Expiration", "X-Signature") + if (kind == "attachment") setOf("X-File-Ext") else emptySet()
        val headers = ticket.headers
        if (headers.keys != allowed || headers["Content-Type"] != mime || headers["X-Content-Sha256"] != hash ||
            headers["X-Expiration"]?.toLongOrNull() == null || headers["X-Signature"]?.matches(Regex("[a-f0-9]{64}")) != true ||
            (kind == "attachment" && headers["X-File-Ext"]?.matches(Regex("[a-z0-9]{1,8}")) != true))
            throw ApiException(0, "上传凭证无效")
        val resource = client.config { followRedirects = false }
        val receipt = try {
            val response = resource.post(ticket.uploadUrl) {
                this.headers.remove(HttpHeaders.Authorization)
                headers.forEach { (name, value) -> header(name, value) }
                setBody(file.bytes)
            }
            val text = response.bodyAsText()
            if (response.status.value !in 200..299) throw parseApiError(response.status.value, text)
            text
        } catch (e: CancellationException) { throw e }
        catch (e: ApiException) { throw e }
        catch (e: Exception) { throw ApiException(0, "资源机上传失败，请重试") }
        finally { resource.close() }
        val latest = token() ?: throw ApiException(401, "无法取得登录凭据，请重试或重新登录")
        return jsonPost("/api/resource-uploads/${ticket.id}/complete", receipt, latest)
    }

    private suspend fun upload(path: String, file: ForumUploadFile): String {
        val access = token() ?: throw ApiException(401, "无法取得登录凭据，请重试或重新登录")
        val name = forumUploadName(file.name)
        val mime = file.mime.takeIf { Regex("[a-zA-Z0-9.+-]+/[a-zA-Z0-9.+-]+").matches(it) } ?: "application/octet-stream"
        val response = try {
            client.post("$base$path") {
                header(HttpHeaders.Authorization, "Bearer $access")
                setBody(MultiPartFormDataContent(formData {
                    append("file", file.bytes, Headers.build {
                        append(HttpHeaders.ContentType, mime)
                        append(HttpHeaders.ContentDisposition, "filename=\"$name\"")
                    })
                }))
            }
        } catch (e: CancellationException) { throw e }
        catch (e: Exception) { throw ApiException(0, "上传连接失败，请重试") }
        val body = response.bodyAsText()
        if (response.status.value !in 200..299) throw parseApiError(response.status.value, body)
        return body
    }
}
