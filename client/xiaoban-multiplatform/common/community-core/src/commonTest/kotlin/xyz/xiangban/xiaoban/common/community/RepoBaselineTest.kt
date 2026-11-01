package xyz.xiangban.xiaoban.common.community

import kotlin.test.*
import kotlinx.coroutines.test.runTest

class RepoBaselineTest {
    @Test fun `first observation has exact total stars and absent changes rather than zero growth`() {
        val issue = CommunityJson.decodeFromString<RepoIssue>("""{"source":"GitHub","metric":"star total baseline","observed_days":0,"total_gain":null,"wow_pct":null,"daily_gain":[],"ranks":[{"full_name":"owner/project","star_total":132386,"star_gain":null,"spark":[132386],"observation_end":"2026-10-31T13:45:33.689317Z"}]}""")
        assertTrue(issue.isBaseline)
        assertNull(issue.totalGain); assertNull(issue.wowPct)
        assertNull(issue.ranks.single().starGain)
        assertEquals(132386L, issue.ranks.single().starTotal)
        assertEquals(1, issue.ranks.single().spark.size)
        assertTrue(issue.dailyGain.isEmpty())
    }
    @Test fun `a measured zero is retained when two actual observations exist`() {
        val issue = CommunityJson.decodeFromString<RepoIssue>("""{"source":"GitHub","metric":"observed net star change","observed_days":1,"total_gain":0,"ranks":[{"full_name":"owner/project","star_total":100,"star_gain":0,"spark":[100,100]}]}""")
        assertFalse(issue.isBaseline)
        assertEquals(0L, issue.totalGain); assertEquals(0L, issue.ranks.single().starGain)
        assertEquals(2, issue.ranks.single().spark.size)
    }
    @Test fun `trending weekly issue is recognised and carries GitHub published gains without a spark`() {
        val issue = CommunityJson.decodeFromString<RepoIssue>("""{"source":"GitHub","metric":"github trending weekly stars","observed_days":7,"complete":true,"total_gain":12062,"ranks":[{"full_name":"owner/project","star_total":110001,"star_gain":12062,"spark":[],"language":"Rust","language_color":"#dea584"}]}""")
        assertTrue(issue.isTrending)
        assertFalse(issue.isBaseline)
        assertEquals(12062L, issue.totalGain); assertEquals(12062L, issue.ranks.single().starGain)
        assertTrue(issue.ranks.single().spark.isEmpty())
        assertNull(issue.ranks.single().observedDays)
    }
    @Test fun `observation labels normalize offsets to UTC and hide malformed times`() {
        assertEquals("2026-10-31 13:45:33 UTC", repoObservationLabel("2026-10-31T21:45:33+08:00"))
        assertNull(repoObservationLabel(null)); assertNull(repoObservationLabel("not a timestamp"))
    }
    @Test fun `current mobile uses v2 discovery paths and keeps issue queries`() = runTest {
        val s=FakeServer()
        s.on({it.url.encodedPath=="/api/discover/home/v2"},body="""{"repos":null}""")
        s.on({it.url.encodedPath=="/api/discover/repos/v2"},body="""{"source":"GitHub","metric":"star total baseline","total_gain":null,"wow_pct":null,"ranks":[]}""")
        s.on({it.url.encodedPath=="/api/discover/repos/issues/v2"},body="""{"items":[]}""")
        s.on({it.url.encodedPath=="/api/discover/tracker/v2"},body="""{"promos":[]}""")
        val api=ForumApi(s.client,CommunityConfig()){null}
        api.home();api.repoIssue("baseline");api.repoIssues();api.tracker()
        assertEquals(listOf("/api/discover/home/v2","/api/discover/repos/v2","/api/discover/repos/issues/v2","/api/discover/tracker/v2"),s.requests.map{it.url.encodedPath})
        assertEquals("baseline",s.requests[1].url.parameters["issue"])
        s.client.close()
    }

}
