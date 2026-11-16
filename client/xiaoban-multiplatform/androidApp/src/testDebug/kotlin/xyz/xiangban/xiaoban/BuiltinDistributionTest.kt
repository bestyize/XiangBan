package xyz.xiangban.xiaoban

import java.io.File
import kotlinx.coroutines.runBlocking
import kotlinx.serialization.json.Json
import kotlinx.serialization.json.jsonArray
import kotlinx.serialization.json.jsonObject
import kotlinx.serialization.json.jsonPrimitive
import org.junit.Assert.assertEquals
import org.junit.Assert.assertTrue
import org.junit.Rule
import org.junit.Test
import org.junit.rules.TemporaryFolder

class BuiltinDistributionTest {
    @get:Rule val temporary = TemporaryFolder()

    @Test fun officialBundlesDownloadVerifyExtractAndOpenOffline() = runBlocking {
        val manifestPath = System.getenv("XIAOBAN_BUILTIN_DISTRIBUTION") ?: return@runBlocking
        val games = Json.parseToJsonElement(File(manifestPath).readText()).jsonObject.getValue("games").jsonArray
        val repo = requireNotNull(File(manifestPath).absoluteFile.parentFile?.parentFile?.parentFile)
        val runtime = repo.resolve("client/xiaoban-multiplatform/androidApp/src/main/builtin-runtime-v2")
        val cache = temporary.newFolder("builtin-cache")
        assertEquals(11, games.size)
        for (value in games) {
            val entry = value.jsonObject
            fun text(key: String) = entry.getValue(key).jsonPrimitive.content
            val id = text("id")
            val game = AndroidBuiltinGames.Game(id, text("kind"), text("url"), text("sha256"), text("bytes").toLong(), text("sourceSha256"))
            fun load(root: File) = RuntimeV2Package.loadBuiltin(root, id) { path -> runtime.resolve(path.substringAfterLast('/')).inputStream() }
            val destination = AndroidBuiltinGames.ensureIn(cache, game, ::load)
            val pack = load(destination)
            assertEquals(id, pack.manifest.id)
            val html = pack.open(pack.manifest.entry)!!.bufferedReader().use { it.readText() }
            assertTrue(html.indexOf("./builtin-runtime.js") < html.indexOf("<script", html.indexOf("./builtin-runtime.js") + 1))
            assertTrue(html.contains("./builtin-runtime.css"))
            assertTrue(pack.open(pack.manifest.icon)!!.use { it.read() } >= 0)
            assertEquals(destination, AndroidBuiltinGames.ensureIn(cache, game.copy(url = "https://127.0.0.1:1/unavailable"), ::load))
            if (id == "falling-blocks") {
                destination.resolve(pack.manifest.entry).appendText("<!-- corrupted -->")
                val repaired = AndroidBuiltinGames.ensureIn(cache, game, ::load)
                assertTrue(!repaired.resolve(pack.manifest.entry).readText().contains("<!-- corrupted -->"))
            }
        }
    }
}
