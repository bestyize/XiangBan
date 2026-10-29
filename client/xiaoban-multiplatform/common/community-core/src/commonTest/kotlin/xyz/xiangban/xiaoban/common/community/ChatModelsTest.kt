package xyz.xiangban.xiaoban.common.community

import kotlin.test.Test
import kotlin.test.assertEquals
import kotlin.test.assertNotNull
import kotlin.test.assertNull
import kotlin.test.assertTrue

class ChatModelsTest {
    private fun todo(args: String, status: ToolStatus = ToolStatus.Done, output: String = "Todo list updated: 3 items") =
        ToolCallInfo("t", "todo", args, output, status)

    private val plan = """{"todos":[{"content":"读取需求","status":"completed","evidence":"已确认"},{"content":"写草稿","status":"in_progress"},{"content":"安装","status":"pending"}],"handoff":"接着写"}"""

    @Test
    fun `plan comes only from acknowledged todo calls`() {
        val p = assertNotNull(taskPlan(listOf(todo(plan))))
        assertEquals(3, p.steps.size)
        assertEquals(1, p.completed)
        assertEquals(1, p.current)
        assertEquals("接着写", p.handoff)
        assertNull(taskPlan(listOf(todo(plan, status = ToolStatus.Error))))
        assertNull(taskPlan(listOf(todo(plan, output = "rejected"))))
        assertNull(taskPlan(listOf(todo("""{"todos":[{"content":"a","status":"in_progress"},{"content":"b","status":"in_progress"}]}"""))))
        assertNull(taskPlan(listOf(todo("""{"todos":[{"content":"","status":"pending"}]}"""))))
        assertNull(taskPlan(listOf(todo("{half"))))
        // 后一次更新覆盖前一次
        val newer = todo("""{"todos":[{"content":"读取需求","status":"completed"}]}""")
        assertEquals(1, taskPlan(listOf(todo(plan), newer))!!.steps.size)
    }

    @Test
    fun `task state follows what is happening`() {
        val calls = listOf(todo(plan))
        assertEquals(TaskState.Running, taskProgress(calls, running = true)!!.state)
        assertEquals(TaskState.Approval, taskProgress(calls, running = true, waiting = "approval")!!.state)
        assertEquals(TaskState.Waiting, taskProgress(calls, running = true, waiting = "question")!!.state)
        assertEquals(TaskState.Paused, taskProgress(calls, running = false, stopped = true)!!.state)
        assertEquals(TaskState.Blocked, taskProgress(calls, running = false, error = "额度不足")!!.state)
        val failed = calls + ToolCallInfo("c", "shell", """{"command":"npm test"}""", "第 42 行有语法错误\n  at game.js", ToolStatus.Error)
        val blocked = taskProgress(failed, running = false)!!
        assertEquals(TaskState.Blocked, blocked.state)
        assertEquals("第 42 行有语法错误", blocked.message)
        // 同样的调用后来成功了，就不算受阻
        val fixed = failed + ToolCallInfo("c2", "shell", """{"command":"npm test"}""", "ok", ToolStatus.Done)
        assertEquals(TaskState.Paused, taskProgress(fixed, running = false)!!.state)
        val allDone = listOf(todo("""{"todos":[{"content":"a","status":"completed"}]}"""))
        assertEquals(TaskState.Completed, taskProgress(allDone, running = true)!!.state)
        assertNull(taskProgress(listOf(ToolCallInfo("c", "shell")), running = true))
    }

    @Test
    fun `labels, descriptions and summaries`() {
        assertEquals("运行命令", toolLabel("shell"))
        assertEquals("some_tool", toolLabel("some_tool"))
        assertEquals("npm test", describeCall("shell", """{"command":"npm test\nsecond line"}"""))
        assertEquals("喝水 · 10 分钟后", describeCall("set_reminder", """{"title":"喝水","minutes":10}"""))
        assertEquals("", describeCall("shell", "{half"))
        assertEquals("a".repeat(90) + "…", describeCall("shell", """{"command":"${"a".repeat(120)}"}"""))
        val calls = listOf(
            ToolCallInfo("1", "read_file", status = ToolStatus.Done, startedAtMs = 1_000, endedAtMs = 3_000),
            ToolCallInfo("2", "shell", status = ToolStatus.Error, startedAtMs = 3_000, endedAtMs = 19_000),
            todo(plan),
        )
        val s = activitySummary(calls)
        assertEquals(2, s.steps)
        assertEquals(1, s.errors)
        assertEquals(18_000, s.durationMs)
        assertEquals("42 秒", formatDuration(42_000))
        assertEquals("1 分 05 秒", formatDuration(65_000))
        assertTrue(activitySummary(listOf(ToolCallInfo("1", "x"))).durationMs == 0L)
    }

    @Test
    fun `markdown blocks and inlines`() {
        val blocks = parseMarkdown("# 标题\n\n支持 **粗体**、`代码` 和 [链接](https://a.b/c)。\n\n- 一\n- 二\n\n1. 甲\n2. 乙\n\n> 引用\n\n```kotlin\nval x = 1\n```\n\n---\n\n| a | b |\n| - | - |\n| 1 | 2 |")
        assertEquals(MdBlock.Heading(1, listOf(MdInline.Text("标题"))), blocks[0])
        val para = blocks[1] as MdBlock.Paragraph
        assertTrue(para.inlines.contains(MdInline.Bold("粗体")))
        assertTrue(para.inlines.contains(MdInline.Code("代码")))
        assertTrue(para.inlines.contains(MdInline.Link("链接", "https://a.b/c")))
        assertEquals(2, (blocks[2] as MdBlock.Bullets).items.size)
        assertEquals(1, (blocks[3] as MdBlock.Numbered).start)
        assertTrue(blocks[4] is MdBlock.Quote)
        assertEquals(MdBlock.Code("kotlin", "val x = 1"), blocks[5])
        assertEquals(MdBlock.Rule, blocks[6])
        assertTrue((blocks[7] as MdBlock.Code).text.startsWith("| a | b |"))
    }

    @Test
    fun `half-written markdown never loses text`() {
        // 流式输出到一半：围栏没闭合、粗体没闭合、链接没写完
        val open = parseMarkdown("先这样：\n```js\nlet a = 1")
        assertEquals(MdBlock.Code("js", "let a = 1"), open.last())
        assertEquals(listOf(MdInline.Text("**没写完")), parseInlines("**没写完"))
        assertEquals(listOf(MdInline.Text("[文字](htt")), parseInlines("[文字](htt"))
        assertEquals(listOf(MdInline.Text("javascript:alert(1)是链接吗：[x](javascript:alert(1))")), parseInlines("javascript:alert(1)是链接吗：[x](javascript:alert(1))"))
        assertEquals(listOf(MdInline.Text("snake_case_name")), parseInlines("snake_case_name"))
        assertEquals(listOf(MdInline.Text("2*3*4")), parseInlines("2*3*4"))
    }

    @Test
    fun `changed files come from successful writes and edits once each`() {
        fun call(id: String, name: String, args: String, status: ToolStatus = ToolStatus.Done) = ToolCallInfo(id, name, args, status = status)
        val files = changedFiles(listOf(
            call("1", "write_file", """{"path":"game/index.html","content":"x"}"""),
            call("2", "edit_file", """{"path":"game/index.html","old":"a","new":"b"}"""),
            call("3", "edit_file", """{"path":"C:\\work\\game.js"}"""),
            call("4", "write_file", """{"path":"bad.txt"}""", ToolStatus.Error),
            call("5", "read_file", """{"path":"x.md"}"""),
            call("6", "write_file", "not json"),
        ))
        assertEquals(listOf("game/index.html", "C:\\work\\game.js"), files.map { it.path })
        assertEquals(listOf(true, false), files.map { it.created })
        assertEquals("index.html", files[0].name)
        assertEquals("HTML", files[0].extension)
        assertEquals("game.js", files[1].name)
    }

    @Test
    fun `token counts and speech text read naturally`() {
        assertEquals(listOf("987", "1.2k", "9.9k", "15k"), listOf(987, 1234, 9999, 15000).map(::formatTokens))
        assertEquals("标题\n第一点\n第二点\n请看 链接", speechText("# 标题\n\n- **第一点**\n- 第二点\n\n```kotlin\nval x = 1\n```\n\n请看 [链接](https://a.b)"))
    }
}
