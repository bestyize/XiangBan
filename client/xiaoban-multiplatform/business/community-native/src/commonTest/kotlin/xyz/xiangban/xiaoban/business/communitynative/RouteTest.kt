package xyz.xiangban.xiaoban.business.communitynative

import kotlin.test.Test
import kotlin.test.assertEquals
import kotlin.test.assertNull

class RouteTest {
    @Test
    fun everyRouteSurvivesEncoding() {
        val routes = listOf(Route.Pack(42), Route.Topic(7), Route.Weekly("2026-W43"), Route.Project("xiaoban"), Route.Login, Route.Devices,
            Route.Search, Route.NewPost, Route.Settings, Route.Models, Route.AiChat, Route.RemoteChat("dsk_abc123"))
        routes.forEach { assertEquals(it, Route.decode(it.encode())) }
        assertEquals(routes, Route.decodeStack(Route.encodeStack(routes)))
    }

    @Test
    fun garbageIsDroppedInsteadOfCrashing() {
        assertNull(Route.decode("pack:abc"))
        assertNull(Route.decode("weekly:"))
        assertNull(Route.decode("rlist:dsk_1"), "旧版本存的会话列表页不再有")
        assertNull(Route.decode("rchat:"))
        assertNull(Route.decode("rchat"))
        assertNull(Route.decode("unknown:1"))
        assertEquals(listOf(Route.Topic(3)), Route.decodeStack("\nbad\ntopic:3\n"))
        assertEquals(emptyList(), Route.decodeStack(""))
    }

    @Test
    fun valuesMayContainColons() {
        assertEquals(Route.Weekly("a:b"), Route.decode(Route.Weekly("a:b").encode()))
    }
}
