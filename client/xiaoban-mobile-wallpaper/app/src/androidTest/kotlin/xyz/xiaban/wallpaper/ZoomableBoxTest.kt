package xyz.xiaban.wallpaper

import androidx.compose.foundation.background
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.ui.Modifier
import androidx.compose.ui.geometry.Offset
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.test.junit4.createComposeRule
import androidx.compose.ui.test.onNodeWithTag
import androidx.compose.ui.test.performTouchInput
import androidx.compose.ui.test.pinch
import androidx.compose.ui.test.doubleClick
import androidx.compose.ui.test.click
import androidx.compose.ui.test.swipe
import androidx.compose.ui.test.center
import androidx.compose.ui.test.down
import androidx.compose.ui.test.moveBy
import androidx.compose.ui.test.up
import androidx.compose.ui.test.width
import androidx.compose.ui.test.height
import androidx.test.ext.junit.runners.AndroidJUnit4
import org.junit.Assert.assertEquals
import org.junit.Assert.assertFalse
import org.junit.Assert.assertTrue
import org.junit.Rule
import org.junit.Test
import org.junit.runner.RunWith
import xyz.xiaban.wallpaper.ui.ZOOMABLE_TAG
import xyz.xiaban.wallpaper.ui.ZoomMath
import xyz.xiaban.wallpaper.ui.ZoomState
import xyz.xiaban.wallpaper.ui.ZoomableBox

/** Real touches on a real screen: the arithmetic is tested on the JVM, this proves it is wired to the fingers. */
@RunWith(AndroidJUnit4::class)
class ZoomableBoxTest {
    @get:Rule
    val rule = createComposeRule()

    private var taps = 0
    private var dismissed = 0

    private fun show(state: ZoomState, aspect: Float = 1f) {
        taps = 0
        dismissed = 0
        rule.setContent {
            ZoomableBox(aspect = aspect, state = state, onTap = { taps++ }, onDismiss = { dismissed++ }) {
                Box(Modifier.fillMaxSize().background(Color.Magenta))
            }
        }
    }

    @Test
    fun pinchingOutZoomsIn() {
        val state = ZoomState()
        show(state)
        rule.onNodeWithTag(ZOOMABLE_TAG).performTouchInput {
            val c = center
            pinch(c - Offset(40f, 0f), c - Offset(200f, 0f), c + Offset(40f, 0f), c + Offset(200f, 0f))
        }
        rule.waitForIdle()
        assertTrue("scale was ${state.zoom.scale}", state.zoom.scale > 2f)
        assertTrue(state.zoom.scale <= ZoomMath.MAX_SCALE)
        assertEquals(0, dismissed)
    }

    @Test
    fun pinchingInAfterZoomingGoesBackToFit() {
        val state = ZoomState()
        show(state)
        rule.onNodeWithTag(ZOOMABLE_TAG).performTouchInput {
            val c = center
            pinch(c - Offset(40f, 0f), c - Offset(240f, 0f), c + Offset(40f, 0f), c + Offset(240f, 0f))
        }
        rule.waitForIdle()
        assertTrue(state.zoom.scale > 2f)
        rule.onNodeWithTag(ZOOMABLE_TAG).performTouchInput {
            val c = center
            pinch(c - Offset(300f, 0f), c - Offset(20f, 0f), c + Offset(300f, 0f), c + Offset(20f, 0f))
        }
        rule.waitForIdle()
        assertEquals(1f, state.zoom.scale, 0.05f)
        assertEquals(0f, state.zoom.x, 1f)
    }

    @Test
    fun doubleTapZoomsInAndAgainZoomsOut() {
        val state = ZoomState()
        show(state)
        rule.onNodeWithTag(ZOOMABLE_TAG).performTouchInput { doubleClick() }
        rule.waitUntil(3_000) { kotlin.math.abs(state.zoom.scale - ZoomMath.DOUBLE_TAP_SCALE) < 0.01f }
        assertEquals(ZoomMath.DOUBLE_TAP_SCALE, state.zoom.scale, 0.05f)
        assertEquals("a double tap is not a tap", 0, taps)
        rule.onNodeWithTag(ZOOMABLE_TAG).performTouchInput { doubleClick() }
        rule.waitUntil(3_000) { kotlin.math.abs(state.zoom.scale - 1f) < 0.01f }
        assertEquals(1f, state.zoom.scale, 0.01f)
    }

    @Test
    fun aSingleTapIsReportedAsATap() {
        val state = ZoomState()
        show(state)
        rule.onNodeWithTag(ZOOMABLE_TAG).performTouchInput { click() }
        rule.waitUntil(3_000) { taps == 1 }
        assertEquals(0, dismissed)
        assertTrue(state.zoom.isFit)
    }

    @Test
    fun draggingAZoomedPictureMovesItAndStopsAtTheEdge() {
        val state = ZoomState()
        show(state)
        rule.onNodeWithTag(ZOOMABLE_TAG).performTouchInput { doubleClick() }
        rule.waitUntil(3_000) { kotlin.math.abs(state.zoom.scale - ZoomMath.DOUBLE_TAP_SCALE) < 0.01f }
        val before = state.zoom
        rule.onNodeWithTag(ZOOMABLE_TAG).performTouchInput {
            swipe(center, center + Offset(-width / 4f, 0f), durationMillis = 300)
        }
        rule.waitForIdle()
        assertTrue("moved from ${before.x} to ${state.zoom.x}", state.zoom.x < before.x - 20f)
        assertEquals("dragging a zoomed picture must not close the viewer", 0, dismissed)
        rule.onNodeWithTag(ZOOMABLE_TAG).performTouchInput {
            swipe(center, center + Offset(-width * 0.9f, 0f), durationMillis = 300)
            swipe(center, center + Offset(-width * 0.9f, 0f), durationMillis = 300)
        }
        rule.waitForIdle()
        val limit = (rule.onNodeWithTag(ZOOMABLE_TAG).fetchSemanticsNode().size.width * (state.zoom.scale - 1f)) / 2f
        assertTrue("x=${state.zoom.x} limit=$limit", state.zoom.x >= -limit - 1f)
    }

    @Test
    fun pullingAFitPictureFarEnoughClosesTheViewer() {
        val state = ZoomState()
        show(state)
        rule.onNodeWithTag(ZOOMABLE_TAG).performTouchInput {
            swipe(center, center + Offset(0f, height * 0.4f), durationMillis = 300)
        }
        rule.waitForIdle()
        assertEquals(1, dismissed)
    }

    @Test
    fun aShortPullSpringsBackAndKeepsTheViewerOpen() {
        val state = ZoomState()
        show(state)
        rule.onNodeWithTag(ZOOMABLE_TAG).performTouchInput {
            swipe(center, center + Offset(0f, height * 0.05f), durationMillis = 300)
        }
        rule.waitUntil(3_000) { state.dragY == 0f }
        assertEquals(0, dismissed)
        assertFalse(state.zoom.scale != 1f)
    }
}
