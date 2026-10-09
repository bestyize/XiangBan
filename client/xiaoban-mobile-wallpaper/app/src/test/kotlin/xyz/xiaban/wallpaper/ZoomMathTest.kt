package xyz.xiaban.wallpaper

import org.junit.Assert.assertEquals
import org.junit.Assert.assertFalse
import org.junit.Assert.assertTrue
import org.junit.Test
import xyz.xiaban.wallpaper.ui.Zoom
import xyz.xiaban.wallpaper.ui.ZoomMath

private const val EPS = 0.01f

class ZoomMathTest {
    private val square = ZoomMath.Frame(viewW = 1000f, viewH = 1000f, contentW = 1000f, contentH = 1000f)

    @Test
    fun fitsAPictureInsideTheScreenKeepingItsShape() {
        val wide = ZoomMath.fitted(1080f, 2000f, 1.5f)
        assertEquals(1080f, wide.first, EPS)
        assertEquals(720f, wide.second, EPS)
        val tall = ZoomMath.fitted(1080f, 2000f, 0.5f)
        assertEquals(1000f, tall.first, EPS)
        assertEquals(2000f, tall.second, EPS)
        val unknown = ZoomMath.fitted(1080f, 2000f, 0f)
        assertEquals(1080f, unknown.first, EPS)
        assertEquals(2000f, unknown.second, EPS)
    }

    @Test
    fun scaleStaysBetweenFitAndTheMaximum() {
        assertEquals(1f, ZoomMath.limit(Zoom(0.4f, 50f, 50f), square).scale, EPS)
        assertEquals(ZoomMath.MAX_SCALE, ZoomMath.limit(Zoom(99f, 0f, 0f), square).scale, EPS)
    }

    @Test
    fun aPictureThatFitsCannotBeDraggedAway() {
        val z = ZoomMath.limit(Zoom(1f, 300f, -300f), square)
        assertEquals(0f, z.x, EPS)
        assertEquals(0f, z.y, EPS)
    }

    @Test
    fun zoomedPictureCannotBeDraggedPastItsEdges() {
        val z = ZoomMath.limit(Zoom(2f, 900f, -900f), square)
        assertEquals(500f, z.x, EPS)
        assertEquals(-500f, z.y, EPS)
    }

    @Test
    fun aLetterboxedPictureOnlyMovesAlongTheAxisThatOverflows() {
        val frame = ZoomMath.Frame(viewW = 1000f, viewH = 2000f, contentW = 1000f, contentH = 500f)
        val z = ZoomMath.limit(Zoom(2f, 400f, 400f), frame)
        assertEquals(400f, z.x.coerceAtMost(500f), EPS)
        assertEquals(0f, z.y, EPS)
    }

    @Test
    fun pinchingKeepsThePointUnderTheFingersInPlace() {
        val centroid = 600f to 500f
        val z = ZoomMath.transform(Zoom(), centroid.first, centroid.second, 0f, 0f, 2f, square)
        assertEquals(2f, z.scale, EPS)
        val cx = 500f
        val cy = 500f
        val p = cx + (centroid.first - cx - 0f) / 1f
        val onScreen = cx + z.scale * (p - cx) + z.x
        assertEquals(centroid.first, onScreen, EPS)
        assertEquals(500f, cy + z.scale * (centroid.second - cy) + z.y, EPS)
    }

    @Test
    fun panWhileZoomedMovesThePictureAndStopsAtTheEdge() {
        val start = Zoom(2f, 0f, 0f)
        val moved = ZoomMath.transform(start, 500f, 500f, 120f, -80f, 1f, square)
        assertEquals(120f, moved.x, EPS)
        assertEquals(-80f, moved.y, EPS)
        val far = ZoomMath.transform(start, 500f, 500f, 5000f, 0f, 1f, square)
        assertEquals(500f, far.x, EPS)
    }

    @Test
    fun pinchingOutToFitCentresThePictureAgain() {
        val zoomed = Zoom(2f, 300f, 200f)
        val back = ZoomMath.transform(zoomed, 500f, 500f, 0f, 0f, 0.1f, square)
        assertEquals(1f, back.scale, EPS)
        assertEquals(0f, back.x, EPS)
        assertEquals(0f, back.y, EPS)
    }

    @Test
    fun doubleTapZoomsInAroundTheTapAndThenBackOut() {
        val zoomedIn = ZoomMath.doubleTap(Zoom(), 700f, 500f, square)
        assertEquals(ZoomMath.DOUBLE_TAP_SCALE, zoomedIn.scale, EPS)
        val under = 500f + zoomedIn.scale * ((700f) - 500f) + zoomedIn.x
        assertEquals(700f, under, EPS)
        val out = ZoomMath.doubleTap(zoomedIn, 700f, 500f, square)
        assertEquals(Zoom(), out)
    }

    @Test
    fun doubleTapAtACornerStaysInsideThePicture() {
        val z = ZoomMath.doubleTap(Zoom(), 1000f, 1000f, square)
        val max = (1000f * ZoomMath.DOUBLE_TAP_SCALE - 1000f) / 2
        assertTrue(z.x >= -max - EPS && z.x <= max + EPS)
        assertTrue(z.y >= -max - EPS && z.y <= max + EPS)
    }

    @Test
    fun aSwipeClosesTheViewerOnlyWhenItGoesFarEnough() {
        assertFalse(ZoomMath.shouldDismiss(100f, 2000f))
        assertTrue(ZoomMath.shouldDismiss(400f, 2000f))
        assertTrue(ZoomMath.shouldDismiss(-400f, 2000f))
    }

    @Test
    fun theBackdropFadesAsThePictureIsPulledAway() {
        assertEquals(1f, ZoomMath.backdropAlpha(0f, 2000f), EPS)
        assertTrue(ZoomMath.backdropAlpha(500f, 2000f) < 1f)
        assertTrue(ZoomMath.backdropAlpha(5000f, 2000f) >= 0.3f)
    }

    @Test
    fun isFitOnlyWhenNotZoomedIn() {
        assertTrue(Zoom().isFit)
        assertTrue(Zoom(1.005f, 0f, 0f).isFit)
        assertFalse(Zoom(1.2f, 0f, 0f).isFit)
    }
}
