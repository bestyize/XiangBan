package xyz.xiangban.xiaoban.business.communitynative

import kotlin.math.abs
import kotlin.test.Test
import kotlin.test.assertEquals
import kotlin.test.assertFalse
import kotlin.test.assertTrue

/** 和网页端 tests/avatarCrop.test.ts 同一组用例，两端裁剪结果一致。 */
class AvatarCropTest {
    private val d = 300f
    private fun near(a: Float, b: Float) = assertTrue(abs(a - b) < 1e-3f, "$a ≈ $b")

    @Test
    fun startsCentredWithTheShortSideFillingTheCircle() {
        val (sx, sy, size) = AvatarCrop(640, 480).source(d)
        near(size, 480f); near(sx, 80f); near(sy, 0f)
    }

    @Test
    fun panningNeverUncoversTheCircle() {
        val moved = AvatarCrop(640, 480).panned(d, 10_000f, 10_000f)
        near(moved.ty, 0f)
        near(moved.source(d).first, 0f)
        val (left, _, size) = AvatarCrop(640, 480).panned(d, -10_000f, 0f).source(d)
        near(left + size, 640f)
    }

    @Test
    fun zoomKeepsTheChosenPointStillAndStaysWithinOneToFour() {
        val zoomed = AvatarCrop(400, 400).zoomed(d, 2f, 50f, 0f)
        near(zoomed.zoom, 2f); near(zoomed.tx, -50f)
        assertEquals(4f, zoomed.zoomed(d, 9f).zoom)
        assertEquals(1f, zoomed.zoomed(d, 0.1f).zoom)
        near(zoomed.zoomed(d, 0.1f).source(d).third, 400f)
    }

    @Test
    fun rotatingSwapsTheSidesAndKeepsTheCropValid() {
        val rotated = AvatarCrop(640, 480).panned(d, 40f, 0f).rotated(d)
        assertEquals(480, rotated.iw); assertEquals(640, rotated.ih)
        near(rotated.tx, 0f); near(rotated.ty, 40f)
        assertEquals(rotated, rotated.clamped(d))
    }

    @Test
    fun smallSourcesAndDoubleTap() {
        assertTrue(AvatarCrop(200, 300).small)
        assertFalse(AvatarCrop(1024, 768).small)
        assertTrue(AvatarCrop(1024, 768, zoom = 4f).small)
        assertEquals(2.2f, avatarDoubleTapZoom(1f))
        assertEquals(1f, avatarDoubleTapZoom(2.2f))
    }

    @Test
    fun rescalingFollowsTheStageSize() {
        val crop = AvatarCrop(640, 480, tx = 30f, ty = -10f).rescaled(300f, 600f)
        near(crop.tx, 60f); near(crop.ty, -20f)
        assertEquals(crop, crop.rescaled(0f, 300f))
    }
}
