package xyz.xiangban.xiaoban

import org.junit.Assert.assertEquals
import org.junit.Assert.assertNull
import org.junit.Test

class RuntimeV2MbtiShareTest {
    @Test fun onlyPublicResultSnapshotsAreAllowed() {
        val url = "https://chat.xiangban.xyz/builtin-games/mbti/share.html?t=INFJ&p=25%2C25%2C25%2C75&n=12%2C12%2C12%2C12&d=2026-11-08T13%3A46%3A00.000Z"
        assertEquals(url, RuntimeV2MbtiShare.validatedLink(url))
        listOf(
            url.replace("chat.xiangban.xyz", "chat.xiangban.xyz.evil.example"),
            url.replace("https://", "http://"),
            url.replace("share.html", "%73hare.html"),
            url + "&answers=secret", url + "&t=ESTJ", url + "#private",
            url.replace("25%2C25%2C25%2C75", "101%2C25%2C25%2C75"),
            url.replace("25%2C25%2C25%2C75", "75%2C25%2C25%2C75"),
            url.replace("12%2C12%2C12%2C12", "12%2C0%2C12%2C12"),
        ).forEach { assertNull(it, RuntimeV2MbtiShare.validatedLink(it)) }
    }
}
