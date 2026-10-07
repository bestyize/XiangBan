package xyz.xiaban.server.flags

import kotlin.test.Test
import kotlin.test.assertEquals
import kotlin.test.assertFalse
import kotlin.test.assertTrue
import kotlinx.coroutines.test.runTest
import xyz.xiaban.contracts.admin.settings.FlagsUpdate

class FeatureFlagsTest {
    private class Switch(var value: Boolean) : LlmSwitch {
        override suspend fun enabled() = value
        override suspend fun setEnabled(enabled: Boolean) { value = enabled }
    }

    private fun flags(stored: MutableMap<String, Boolean>, source: LlmSwitch? = null, environment: Boolean = false) = FeatureFlags(
        readStored = { stored[it] },
        writeStored = { field, value -> stored[field] = value },
        llmDefault = environment,
        goHomeDefault = false,
        appDownloadUrl = "https://example.com/app",
        llmSwitch = source,
    )

    @Test fun `admin switch overrides stale redis and environment values in both directions`() = runTest {
        val stored = mutableMapOf("llm" to false)
        val source = Switch(true)
        val flags = flags(stored, source)
        assertTrue(flags.llmEnabled())
        assertTrue(flags.view().llmEnabled)
        stored["llm"] = true
        source.value = false
        assertFalse(flags.llmEnabled())
        assertFalse(flags(stored, source, environment = true).view().llmEnabled)
    }

    @Test fun `legacy flags writes update the same switch used by admin settings`() = runTest {
        val stored = mutableMapOf("llm" to false)
        val source = Switch(false)
        val flags = flags(stored, source)
        assertTrue(flags.update(FlagsUpdate(llmEnabled = true)).llmEnabled)
        assertTrue(source.value)
        assertEquals(false, stored["llm"], "legacy Redis value is no longer a second switch")
        assertFalse(flags.update(FlagsUpdate(llmEnabled = false)).llmEnabled)
        assertFalse(source.value)
    }

    @Test fun `updating ordinary flags never overwrites the model switch`() = runTest {
        val stored = mutableMapOf<String, Boolean>()
        val source = Switch(true)
        val flags = flags(stored, source)
        val result = flags.update(FlagsUpdate(goHomeEnabled = true))
        assertTrue(result.llmEnabled)
        assertTrue(result.goHomeEnabled)
        assertTrue(source.value)
        assertEquals(true, stored["go_home"])
    }

    @Test fun `standalone flags retain redis and environment fallback behavior`() = runTest {
        val stored = mutableMapOf<String, Boolean>()
        val flags = flags(stored, environment = true)
        assertTrue(flags.llmEnabled())
        assertFalse(flags.update(FlagsUpdate(llmEnabled = false)).llmEnabled)
        assertEquals(false, stored["llm"])
    }
}
