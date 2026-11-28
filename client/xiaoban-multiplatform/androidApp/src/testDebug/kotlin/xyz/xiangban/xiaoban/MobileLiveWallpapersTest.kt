package xyz.xiangban.xiaoban

import org.junit.Assert.assertEquals
import org.junit.Assert.assertFalse
import org.junit.Assert.assertNull
import org.junit.Assert.assertTrue
import org.junit.Test
import xyz.xiangban.xiaoban.MobileLiveWallpapers.Selection
import xyz.xiangban.xiaoban.MobileLiveWallpapers.Slot.PRIMARY
import xyz.xiangban.xiaoban.MobileLiveWallpapers.Slot.SECONDARY

class MobileLiveWallpapersTest {
    @Test fun legacyFilesUseTheirSlotDefaultsAndNewPresetsSurvive() {
        assertEquals("wallpaper-aquarium", PRIMARY.fileName)
        assertEquals("wallpaper-galaxy", SECONDARY.fileName)
        for (oldValue in listOf(null, "aquarium", "galaxy", "invalid")) {
            assertEquals("dusk", PRIMARY.presetFromStored(oldValue))
            assertEquals("neon", SECONDARY.presetFromStored(oldValue))
        }
        for (preset in listOf("dusk", "morning", "moss", "neon", "snow")) {
            assertEquals(preset, PRIMARY.presetFromStored(preset))
            assertEquals(preset, SECONDARY.presetFromStored(preset))
        }
        assertFalse(MobileLiveWallpapers.isPreset("aquarium"))
        assertFalse(MobileLiveWallpapers.isPreset("galaxy"))
    }

    @Test fun samePresetOnDifferentComponentsCanBeSwitchedIndependently() {
        val shared = Selection(PRIMARY, PRIMARY, "dusk", "dusk")
        val split = Selection(PRIMARY, SECONDARY, "dusk", "dusk")
        assertEquals("both", shared.switchTarget("wallpaper"))
        assertNull(shared.switchSlot("lock"))
        assertEquals(SECONDARY, shared.unusedSlot("dusk"))
        assertEquals(PRIMARY, split.switchSlot("home"))
        assertEquals(SECONDARY, split.switchSlot("lock"))
        assertTrue(split.hasNewWallpaper(shared))
        assertFalse(split.hasNewWallpaper(split))
    }
}
