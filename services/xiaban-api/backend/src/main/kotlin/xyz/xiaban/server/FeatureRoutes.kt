package xyz.xiaban.server

import io.ktor.http.*
import io.ktor.server.auth.authenticate
import io.ktor.server.request.receive
import io.ktor.server.request.receiveChannel
import io.ktor.server.response.*
import io.ktor.server.routing.*
import io.ktor.utils.io.readBuffer
import kotlinx.io.readByteArray
import xyz.xiaban.contracts.common.OkResponse
import xyz.xiaban.server.afterwork.*
import xyz.xiaban.server.auth.AUTH_USER
import xyz.xiaban.server.auth.userId
import xyz.xiaban.server.games.*
import xyz.xiaban.server.http.ApiException
import xyz.xiaban.server.http.notFound
import xyz.xiaban.server.quizzes.QuizAttemptRecord
import xyz.xiaban.server.users.AvatarService

fun Route.featureRoutes(services: Services) {
    get("/api/display") {
        call.response.header(HttpHeaders.CacheControl, "no-store")
        call.respond(services.displaySettings.current())
    }
    get("/api/avatars/{key}") {
        val bytes = services.avatars.image(call.parameters["key"].orEmpty()) ?: notFound("没有找到头像")
        call.response.header(HttpHeaders.CacheControl, "private, max-age=300")
        call.respondBytes(bytes, ContentType.Image.JPEG)
    }
    authenticate(AUTH_USER) {
        put("/api/account/avatar") {
            val bytes = call.receiveChannel().readBuffer(AvatarService.MAX_BYTES.toLong() + 1).readByteArray()
            services.avatars.save(call.userId(), bytes)
            call.respond(services.profiles.view(call.userId()))
        }
        delete("/api/account/avatar") { services.avatars.remove(call.userId()); call.respond(services.profiles.view(call.userId())) }
        post("/api/games/sessions") { call.respond(services.games.start(call.userId(), call.receive<GameStart>().gameId)) }
        put("/api/games/sessions/{id}") { call.respond(services.games.progress(call.userId(), call.parameters["id"].orEmpty(), call.receive<GameProgress>())) }
        get("/api/quizzes/progress") {
            if (services.profiles.view(call.userId()).accountType != "registered")
                throw ApiException(HttpStatusCode.Forbidden, "login_required", "登录后可以保存测试进度")
            call.respond(services.quizzes.all(call.userId()))
        }
        put("/api/quizzes/progress") {
            if (services.profiles.view(call.userId()).accountType != "registered")
                throw ApiException(HttpStatusCode.Forbidden, "login_required", "登录后可以保存测试进度")
            call.respond(services.quizzes.save(call.userId(), call.receive<QuizAttemptRecord>()))
        }
        get("/api/games/summary") { call.respond(services.games.summaries(call.userId())) }
        get("/api/afterwork") { call.respond(services.afterwork.data(call.userId())) }
        put("/api/afterwork/choices") { call.respond(services.afterwork.choose(call.userId(), call.receive<AfterworkChoice>())) }
        put("/api/afterwork/preferences") { call.respond(services.afterwork.preference(call.userId(), call.receive<AfterworkPreference>())) }
        post("/api/afterwork/import") { call.respond(services.afterwork.importData(call.userId(), call.receive<AfterworkData>())) }
        delete("/api/afterwork") { services.afterwork.clear(call.userId()); call.respond(OkResponse()) }
    }
}
