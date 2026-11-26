package xyz.xiangban.xiaoban

import org.junit.Assert.assertEquals
import org.junit.Assert.assertNull
import org.junit.Test

class WebWorkLinkTest {
    private val prefix = "xiaoban://community/run/"

    @Test fun preservesRuntimeIdentityAndVersion() {
        assertEquals(WebWorkLink.Builtin("mbti"), WebWorkLink.parse(prefix + "builtin/mbti"))
        assertEquals(WebWorkLink.Idea("jiangshan-map"), WebWorkLink.parse(prefix + "idea/jiangshan-map"))
        assertEquals(WebWorkLink.Pack(38L, 7), WebWorkLink.parse(prefix + "pack/38/7"))
    }

    @Test fun ignoresOtherActionsAndInvalidTargets() {
        assertNull(WebWorkLink.parse(null))
        assertNull(WebWorkLink.parse("xiaoban://community/open/pack/38"))
        for (path in listOf("idea/", "idea/Bad", "builtin/-foo", "idea/mbti?x=1", "pack/2", "pack/2/0",
            "pack/2/01", "pack/2/2147483648", "pack/9007199254740992/1", "pack/2/3/4")) {
            assertNull(path, WebWorkLink.parse(prefix + path))
        }
    }
}
