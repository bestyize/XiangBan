package xyz.xiangban.xiaoban.business.communitynative

import kotlin.test.Test
import kotlin.test.assertEquals
import kotlin.test.assertNull

class RouteTest {
    @Test
    fun everyRouteSurvivesEncoding() {
        val routes = listOf(Route.Pack(42), Route.Topic(7), Route.Weekly("2026-W43"), Route.Project("xiaoban"), Route.Login, Route.Devices,
            Route.Tool("cc-switch"), Route.Frontier, Route.FrontierArticle("中文:1"), Route.Articles("create-game", "图文分享"), Route.Daily, Route.News("news-article-1"), Route.Leaderboard, Route.Scatter, Route.Search, Route.NewPost, Route.NewHelp, Route.Settings, Route.Models, Route.AiChat, Route.RemoteChat("dsk_abc123"),
            Route.OpenSource("AppFlowy-IO", "AppFlowy"), Route.OpenSourceCollection("ideas-to-things"),
            Route.WebPage("https://github.com/example/project?tab=readme&x=a%20b#license"))
        routes.forEach { assertEquals(it, Route.decode(it.encode())) }
        assertEquals(routes, Route.decodeStack(Route.encodeStack(routes)))
    }

    @Test
    fun routeRemovedInLaterVersionsIsDroppedFromASavedStack() {
        // 「社区分享」页面已下线；旧版本存下的页面栈里可能还留着它，升级后要被跳过而不是崩溃。
        assertNull(Route.decode("opensource-shares"))
        assertEquals(listOf(Route.Daily, Route.Leaderboard), Route.decodeStack("daily\nopensource-shares\nleaderboard"))
    }

    @Test
    fun creationDraftsSurviveStackRestoreWithoutSplittingOnNewlines() {
        val routes = listOf(
            Route.PublishPack(localGameId = "pixel-adventure"),
            Route.CreationPost("分享：小猫 + 星星", "第一行\n第二行：100% & hello?"),
            Route.RemoteChat("dsk_1", "请制作壁纸：山丘\n保留 + % 与中文"),
        )
        assertEquals(routes, Route.decodeStack(Route.encodeStack(routes)))
        assertNull(Route.decode("publishgame:../private"))
        assertNull(Route.decode("publishgame:"))
    }

    @Test
    fun embeddedBrowserAcceptsOnlyWebLinksWithAHost() {
        assertEquals(Route.WebPage("http://example.org"), webPageRoute("http://example.org"))
        for (url in listOf("javascript:alert(1)", "file:///tmp/page", "https:///path", "https://user@example.org")) {
            assertNull(webPageRoute(url))
        }
    }

    @Test
    fun garbageIsDroppedInsteadOfCrashing() {
        assertNull(Route.decode("pack:abc"))
        assertNull(Route.decode("weekly:"))
        assertNull(Route.decode("rlist:dsk_1"), "旧版本存的会话列表页不再有")
        assertNull(Route.decode("rchat:"))
        assertNull(Route.decode("rchat"))
        assertNull(Route.decode("tool:"))
        assertNull(Route.decode("unknown:1"))
        assertEquals(listOf(Route.Topic(3)), Route.decodeStack("\nbad\ntopic:3\n"))
        assertEquals(emptyList(), Route.decodeStack(""))
    }

    @Test
    fun valuesMayContainColons() {
        assertEquals(Route.Weekly("a:b"), Route.decode(Route.Weekly("a:b").encode()))
    }

    @Test
    fun oldAccentRouteRestoresThemeWithoutDuplicatePage() {
        assertEquals(Route.Appearance, Route.decode("accent"))
        assertEquals(listOf(Route.Settings, Route.Appearance), Route.decodeStack("settings\naccent"))
        assertEquals(listOf(Route.Settings, Route.Appearance), Route.decodeStack("settings\nappearance\naccent"))
    }
}
