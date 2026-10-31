package xyz.xiangban.xiaoban.common.community

import io.ktor.http.HttpHeaders
import io.ktor.http.HttpStatusCode
import kotlinx.coroutines.test.runTest
import kotlin.test.*

class WorkDiscussionTest {
    @Test fun `reading a work is anonymous and never creates a topic`() = runTest {
        val s = FakeServer(); val api = ContentSocialApi(s.client, CommunityConfig()) { null }
        s.on("/api/content-social/pack/7/discussion", body = """{"topic_id":null,"like_count":0,"reply_count":0,"liked":false,"can_reply":false,"locked":false}""")
        assertNull(api.discussion("pack",7).topicId)
        assertEquals("GET",s.requests.single().method.value)
        assertNull(s.requests.single().headers[HttpHeaders.Authorization])
        assertEquals(401,assertFailsWith<ApiException> { api.discussion("pack",7,create=true) }.status)
        assertEquals(1,s.requests.size); s.client.close()
    }
    @Test fun `first interaction explicitly creates or reuses a thread with current credentials`() = runTest {
        val s = FakeServer(); var token = "first"; val api = ContentSocialApi(s.client, CommunityConfig()) { token }
        s.on("/api/content-social/scheme/8/discussion", body = """{"topic_id":45,"like_count":9,"reply_count":2,"liked":true,"can_reply":true,"locked":false}""")
        assertEquals(9,api.discussion("scheme",8).likeCount)
        token = "refreshed"
        assertEquals(45,api.discussion("scheme",8,true).topicId)
        assertEquals(listOf("GET","POST"),s.requests.map { it.method.value })
        assertEquals("Bearer refreshed",s.requests.last().headers[HttpHeaders.Authorization]); s.client.close()
    }
    @Test fun `unavailable discussion preserves server errors and invalid targets send nothing`() = runTest {
        val s=FakeServer(); val api=ContentSocialApi(s.client,CommunityConfig()) { "access" }
        assertEquals(400,assertFailsWith<ApiException> { api.discussion("topic",7,true) }.status)
        assertEquals(400,assertFailsWith<ApiException> { api.discussion("pack",0) }.status)
        assertTrue(s.requests.isEmpty())
        s.on("/api/content-social/pack/7/discussion",HttpStatusCode.NotFound,"""{"message":"内容已不可用"}""")
        assertEquals(404,assertFailsWith<ApiException> { api.discussion("pack",7) }.status); s.client.close()
    }
    @Test fun `comment likes use the existing post endpoint`() = runTest {
        val s=FakeServer(); val api=ForumApi(s.client,CommunityConfig()) { "access" }
        s.on("/api/posts/18/like",body="""{"liked":true,"like_count":1}""")
        api.setPostLiked(18,true); api.setPostLiked(18,false)
        assertEquals(listOf("PUT","DELETE"),s.requests.map { it.method.value })
        val post=CommunityJson.decodeFromString<Post>("""{"id":18,"floor":1,"author":{"uid":"u","nickname":"作者"},"liked":true,"like_count":3}""")
        assertTrue(post.liked); assertEquals(3,post.likeCount); s.client.close()
    }
    @Test fun `work counters remain backward compatible and notifications identify the creator work`() {
        val old=CommunityJson.decodeFromString<Pack>("""{"id":7,"kind":"game","title":"作品"}""")
        assertEquals(0,old.likeCount); assertEquals(0,old.replyCount)
        val actor=NotificationActor("u","读者")
        assertEquals("读者 赞了你的作品「作品」",notificationHeadline(CommunityNotification(1,"work_like","作品",actor=actor,packId=7)))
        assertEquals("读者 评论了你的作品「作品」",notificationHeadline(CommunityNotification(2,"reply","作品",actor=actor,packId=7)))
    }
}
