package xyz.xiangban.xiaoban.common.community

import kotlin.test.*

class TrackerOffersTest {
    @Test fun `ongoing verified offer has no artificial countdown and retains eligibility and source`() {
        val tracker=CommunityJson.decodeFromString<Tracker>("""{"promos":[{"key":"student","text":"学生试用","ends_at":null,"pct":null,"days_left":null,"source_url":"https://education.github.com/pack","eligibility":"须通过学生身份验证"}],"launches":[{"key":"old","title":"历史回顾｜Qwen3","summary":"历史原文","source_url":"https://qwenlm.github.io/blog/qwen3/","published_at":"2025-04-29T00:00:00Z"}]}""")
        val offer=tracker.promos.single()
        assertNull(offer.endsAt);assertNull(offer.pct);assertNull(offer.daysLeft)
        assertEquals("须通过学生身份验证",offer.eligibility)
        assertEquals("https://education.github.com/pack",trackerOfficialSource(offer.sourceUrl))
        assertEquals("2025-04-29",tracker.launches.single().publishedAt.take(10))
        assertEquals("历史原文",tracker.launches.single().summary)
    }
    @Test fun `legacy timed offers retain countdown values and absent links stay absent`() {
        val offer=CommunityJson.decodeFromString<PromoItem>("""{"key":"timed","text":"限时试用","ends_at":"2026-11-04T00:00:00Z","pct":60,"days_left":3}""")
        assertEquals(60,offer.pct);assertEquals(3,offer.daysLeft);assertNull(offer.sourceUrl)
        assertNull(trackerOfficialSource(offer.sourceUrl))
    }
    @Test fun `unsafe or malformed sources cannot reach platform URL actions`() {
        for (url in listOf("javascript:alert(1)","http://official.example/","https://user:secret@official.example/","https://official.example:8443/","https://official.example/\n")) assertNull(trackerOfficialSource(url),url)
        assertEquals("https://official.example/path",trackerOfficialSource("https://official.example/path"))
    }
}
