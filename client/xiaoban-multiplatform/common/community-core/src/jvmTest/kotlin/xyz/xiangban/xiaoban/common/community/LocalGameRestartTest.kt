package xyz.xiangban.xiaoban.common.community

import java.io.File
import java.nio.file.Files
import java.util.concurrent.TimeUnit
import kotlin.test.Test
import kotlin.test.assertEquals
import kotlin.test.assertFalse
import kotlin.test.assertTrue
import kotlinx.coroutines.Job
import kotlinx.coroutines.runBlocking
import kotlinx.serialization.KSerializer
import kotlinx.serialization.json.Json

class LocalGameRestartTest {
    @Test
    fun interruptedOverwriteRestoresThePreviousFilesOnRestart() = runBlocking {
        val directory = Files.createTempDirectory("xb-game-restart").toFile().canonicalFile
        try {
            val workspace = directory.resolve("workspace")
            val store = MobileGameStore(JvmMobileWorkspace(workspace))
            store.create(LocalGameRestartProcess.game, LocalGameRestartProcess.files("old"))
            val installed = workspace.resolve("games/restart-game")

            val types = listOf(
                LocalGameRestartProcess::class.java, MobileGameStore::class.java, Unit::class.java,
                Job::class.java, KSerializer::class.java, Json::class.java,
            )
            val classpath = types.map { File(it.protectionDomain.codeSource.location.toURI()).path }
                .distinct().joinToString(File.pathSeparator)
            val java = File(System.getProperty("java.home"), "bin/java")
            fun process(phase: String): Int {
                val running = ProcessBuilder(java.path, "-cp", classpath, LocalGameRestartProcess::class.java.name,
                    phase, workspace.path).redirectErrorStream(true).start()
                if (!running.waitFor(15, TimeUnit.SECONDS)) {
                    running.destroyForcibly()
                    error("作品测试子进程未退出")
                }
                val output = running.inputStream.bufferedReader().use { it.readText() }
                assertEquals(if (phase == "overwrite") 73 else 0, running.exitValue(), output)
                return running.exitValue()
            }
            process("overwrite")
            assertFalse(installed.exists(), "子进程应在旧版移出且新版换入前结束")
            process("read")
            assertTrue(installed.resolve("manifest.json").isFile)
            assertEquals("old", installed.resolve("index.html").readText().substringAfter("<main>").substringBefore("</main>"))
        } finally {
            directory.deleteRecursively()
        }
    }
}

object LocalGameRestartProcess {
    val game = MobileGame("restart-game", "旧版", "测试本地作品")
    fun files(content: String): List<MobileGameFile> = listOf(
        MobileGameFile("index.html", "<!doctype html><main>$content</main>"),
        MobileGameFile("icon.svg", "<svg xmlns='http://www.w3.org/2000/svg'/>"),
        MobileGameFile("cover.svg", "<svg xmlns='http://www.w3.org/2000/svg'/>"),
    )

    @JvmStatic
    fun main(args: Array<String>) = runBlocking {
        val root = File(args[1])
        val disk = JvmMobileWorkspace(root)
        if (args[0] == "overwrite") {
            val workspace = object : MobileWorkspace by disk {
                override suspend fun installGameFiles(id: String, files: List<MobileGameFile>): Boolean {
                    val installation = LocalGameInstallation(root, root::resolve) { from, to ->
                        val moved = from.renameTo(to)
                        if (moved && to.name == "previous") Runtime.getRuntime().halt(73)
                        moved
                    }
                    installation.install(id, files, disk.list(recursive = true))
                    return true
                }
            }
            MobileGameStore(workspace).create(game.copy(name = "新版"), files("new"), overwrite = true)
            error("覆盖进程应在换入新版前结束")
        }
        val store = MobileGameStore(disk)
        check(store.list().single().name == "旧版")
        check(store.readFile(game.id, "index.html").contains("<main>old</main>"))
    }
}
