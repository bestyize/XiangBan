package xyz.xiaban.server.admin

import io.ktor.http.Cookie
import io.ktor.http.HttpStatusCode
import io.ktor.server.application.ApplicationCall
import io.ktor.server.plugins.origin
import io.ktor.server.request.receive
import io.ktor.server.request.httpMethod
import io.ktor.server.response.respond
import io.ktor.server.routing.*
import kotlinx.serialization.Serializable
import xyz.xiaban.server.Services
import xyz.xiaban.server.http.ApiException
import xyz.xiaban.server.http.badRequest
import xyz.xiaban.server.http.tooMany
import xyz.xiaban.server.mail.Mail
import xyz.xiaban.shared.FlagsUpdate
import xyz.xiaban.shared.OkResponse
import kotlinx.coroutines.future.await
import java.security.MessageDigest
import java.net.URI

@Serializable data class AdminLogin(val username: String, val password: String)
@Serializable data class AdminSession(val username: String)
@Serializable data class AdminPassword(val oldPassword: String, val newPassword: String)
@Serializable data class TestEmail(val email: String)

private const val COOKIE = "xb_admin_session"

/** Rows on the dashboard, in display order; ids match web/public/game-packs and web/src/quizzes/catalog.ts. */
private val METRIC_GAMES = linkedMapOf(
    "falling-blocks" to "俄罗斯方块", "merge-2048" to "2048", "block-puzzle" to "小伴消方块", "water-sort" to "调饮料大师",
)
private val METRIC_QUIZZES = linkedMapOf(
    "mbti" to "MBTI", "mbanti" to "M班TI", "animal" to "精神小动物", "afterhours" to "下班第二人格",
)

fun Route.adminRoutes(services: Services) {
    route("/api/admin") {
        post("/login") {
            call.checkOrigin(services)
            val req = call.receive<AdminLogin>()
            val token = services.administrators.login(req.username.trim(), req.password, call.request.origin.remoteHost)
            call.adminCookie(services, token, 8 * 3600)
            call.respond(AdminSession(req.username.trim()))
        }
        get("/session") { call.respond(AdminSession(call.requireAdmin(services))) }
        get("/overview") { call.requireAdmin(services); call.respond(services.adminUsers.overview()) }
        get("/metrics") {
            call.requireAdmin(services)
            val days = call.request.queryParameters["days"]?.toIntOrNull() ?: 7
            val end = call.request.queryParameters["end"]?.let { runCatching { java.time.LocalDate.parse(it) }.getOrNull() ?: badRequest("日期格式不对") }
            call.respond(services.metrics.daily(days, end, METRIC_GAMES, METRIC_QUIZZES))
        }
        get("/users") {
            call.requireAdmin(services)
            val query = call.request.queryParameters
            call.respond(services.adminUsers.list(call.pageParameter("page", 1), call.pageParameter("pageSize", 20),
                query["q"].orEmpty(), query["type"] ?: "all", query["onboarding"] ?: "all", query["sort"] ?: "newest",
                query["from"].orEmpty(), query["to"].orEmpty()))
        }
        get("/users/{id}") { call.requireAdmin(services); call.respond(services.adminUsers.detail(call.userParameter())) }
        put("/users/{id}") {
            val actor = call.requireAdmin(services)
            val id = call.userParameter()
            services.adminUsers.update(id, call.receive<AdminUserUpdate>(), actor)
            services.guardian.reschedule(id)
            call.respond(services.adminUsers.detail(id))
        }
        delete("/users/{id}") {
            val actor = call.requireAdmin(services)
            val id = call.userParameter()
            services.adminUsers.delete(id, call.receive<AdminUserDelete>().confirmUserId, actor)
            services.guardian.reschedule(id)
            call.respond(OkResponse())
        }
        put("/users/{id}/note") {
            val actor = call.requireAdmin(services)
            services.adminUsers.saveNote(call.userParameter(), call.receive<AdminNoteUpdate>().note, actor)
            call.respond(OkResponse())
        }
        get("/audit") {
            call.requireAdmin(services)
            call.respond(services.adminUsers.auditPage(call.pageParameter("page", 1), call.pageParameter("pageSize", 20)))
        }
        post("/logout") {
            call.checkOrigin(services)
            services.administrators.logout(call.request.cookies[COOKIE])
            call.adminCookie(services, "", 0)
            call.respond(OkResponse())
        }
        put("/password") {
            val username = call.requireAdmin(services)
            val req = call.receive<AdminPassword>()
            services.administrators.changePassword(username, req.oldPassword, req.newPassword)
            call.adminCookie(services, "", 0)
            call.respond(OkResponse())
        }
        get("/email") {
            call.requireAdmin(services)
            call.respond(services.emailSettings.current().redacted())
        }
        put("/email") {
            val actor = call.requireAdmin(services)
            val saved = services.emailSettings.save(call.receive<EmailSettings>(), actor)
            call.respond(saved)
        }
        post("/email/test") {
            val username = call.requireAdmin(services)
            val req = call.receive<TestEmail>()
            val email = xyz.xiaban.server.account.EmailBinding.normalize(req.email)
            if (services.redis.cmd.set("xb:admin:testmail:$username", "1", io.lettuce.core.SetArgs.Builder.nx().ex(30)).await() != "OK") {
                tooMany("请等 30 秒再发送测试邮件")
            }
            try { services.mailer.send(Mail(email, "下班了么 · 邮件服务测试", "这是一封由管理后台请求发送的测试邮件。收到它说明 SMTP 发信链路已接通。")) }
            catch (_: Exception) { throw ApiException(HttpStatusCode.ServiceUnavailable, "mail_send_failed", "测试邮件发送失败，请检查主机、端口、授权码与 TLS 设置") }
            call.respond(OkResponse())
        }
        get("/llm") {
            call.requireAdmin(services)
            call.respond(services.llmSettings.current().redacted())
        }
        put("/llm") {
            val actor = call.requireAdmin(services)
            val saved = services.llmSettings.save(call.receive<LlmSettings>(), actor)
            call.respond(saved)
        }
        post("/llm/test") {
            call.requireAdmin(services)
            val req = call.receive<LlmTestRequest>()
            val reply = services.llmSettings.test(req)
            call.respond(LlmTestResponse(reply))
        }
        post("/llm/test-search") {
            call.requireAdmin(services)
            val req = call.receive<SearchTestRequest>()
            val res = services.searchService.test(req.baseUrl)
            call.respond(res)
        }
        get("/flags") { call.requireAdmin(services); call.respond(services.flags.view()) }
        put("/flags") { call.requireAdmin(services); call.respond(services.flags.update(call.receive<FlagsUpdate>())) }
        get("/reports") {
            call.requireAdmin(services)
            val limit = call.request.queryParameters["limit"]?.toIntOrNull()?.coerceIn(1, 200) ?: 50
            call.respond(services.moderation.reports(limit))
        }
        delete("/copies/{id}/disabled") {
            call.requireAdmin(services)
            services.moderation.restore(call.parameters["id"]!!)
            call.respond(OkResponse())
        }
    }
}

private fun ApplicationCall.pageParameter(name: String, default: Int): Int = request.queryParameters[name]?.let {
    it.toIntOrNull() ?: badRequest("分页参数必须为整数")
} ?: default

private fun ApplicationCall.userParameter(): Long = parameters["id"]?.toLongOrNull()?.takeIf { it > 0 }
    ?: badRequest("用户编号不正确")

/**
 * CSRF defence-in-depth: non-GET admin requests must come from our own pages.
 *
 * The Origin is compared against the Host header and the canonical site address. Comparing hostname only
 * (port ignored) keeps local development working: the Vite dev proxy rewrites Host to the backend port
 * (localhost:8080) while the browser's Origin is the app (localhost:5173). X-Forwarded-Host covers
 * reverse proxies. A browser on evil.com can never claim our hostname.
 */
private fun ApplicationCall.checkOrigin(services: Services) {
    val origin = request.headers["Origin"] ?: return
    val originHost = runCatching { URI(origin).host }?.getOrNull() ?: badRequest("请从本站管理后台提交请求", "invalid_origin")
    val hosts = listOfNotNull(
        request.headers["Host"],
        request.headers["X-Forwarded-Host"],
        runCatching { URI(services.config.publicBaseUrl).authority }.getOrNull(),
    )
    if (hosts.none { it.removeSuffix(":80").removeSuffix(":443").substringBefore(':').equals(originHost, ignoreCase = true) }) {
        badRequest("请从本站管理后台提交请求", "invalid_origin")
    }
}

private fun ApplicationCall.adminCookie(services: Services, token: String, age: Int) {
    response.cookies.append(Cookie(COOKIE, token, maxAge = age, path = "/api/admin", httpOnly = true,
        secure = services.config.publicBaseUrl.startsWith("https://"), extensions = mapOf("SameSite" to "Strict")))
}

private suspend fun ApplicationCall.requireAdmin(services: Services): String {
    if (request.httpMethod.value != "GET") checkOrigin(services)
    val expected = services.config.adminToken
    val given = request.headers["X-Admin-Token"]
    if (expected != null && given != null && MessageDigest.isEqual(expected.toByteArray(), given.toByteArray())) return "legacy-token"
    return services.administrators.username(request.cookies[COOKIE])
        ?: throw ApiException(HttpStatusCode.Unauthorized, "admin_unauthorized", "请先登录管理员账号")
}
