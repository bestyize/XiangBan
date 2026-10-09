package xyz.xiaban.wallpaper.ui

import kotlin.math.abs

/**
 * How a picture is scaled and moved inside the full-screen viewer. [scale] 1 means "fit the screen"; [x] and [y]
 * move the picture from the centre of the screen, in pixels, after scaling.
 */
data class Zoom(val scale: Float = 1f, val x: Float = 0f, val y: Float = 0f) {
    val isFit: Boolean get() = scale < 1.01f
}

/** The arithmetic of pinch, drag and double-tap, kept free of Android so it can be tested on the JVM. */
object ZoomMath {
    const val MAX_SCALE = 6f
    const val DOUBLE_TAP_SCALE = 2.5f
    private const val DISMISS_FRACTION = 0.15f
    private const val FADE_FRACTION = 0.6f
    private const val MIN_BACKDROP = 0.3f

    /** [contentW] and [contentH] are the picture's size at scale 1, i.e. already fitted into the view. */
    class Frame(val viewW: Float, val viewH: Float, val contentW: Float, val contentH: Float)

    /** The size a picture of [aspect] (width over height) takes when fitted into the view, without cropping. */
    fun fitted(viewW: Float, viewH: Float, aspect: Float): Pair<Float, Float> {
        if (aspect <= 0f || viewW <= 0f || viewH <= 0f) return viewW to viewH
        return if (aspect >= viewW / viewH) viewW to viewW / aspect else viewH * aspect to viewH
    }

    /** Keeps the scale in range and the picture from leaving the screen. A picture smaller than the screen stays centred. */
    fun limit(zoom: Zoom, frame: Frame): Zoom {
        val scale = zoom.scale.coerceIn(1f, MAX_SCALE)
        val maxX = ((frame.contentW * scale - frame.viewW) / 2f).coerceAtLeast(0f)
        val maxY = ((frame.contentH * scale - frame.viewH) / 2f).coerceAtLeast(0f)
        return Zoom(scale, zoom.x.coerceIn(-maxX, maxX), zoom.y.coerceIn(-maxY, maxY))
    }

    /**
     * One step of a pinch/drag: the fingers are at ([centroidX], [centroidY]) after moving by ([panX], [panY]),
     * and the spread changed by [zoomChange]. The point of the picture that was under the fingers stays under them.
     */
    fun transform(zoom: Zoom, centroidX: Float, centroidY: Float, panX: Float, panY: Float, zoomChange: Float, frame: Frame): Zoom {
        val newScale = (zoom.scale * zoomChange).coerceIn(1f, MAX_SCALE)
        val cx = frame.viewW / 2f
        val cy = frame.viewH / 2f
        val ratio = newScale / zoom.scale
        val x = centroidX + panX - cx - ratio * (centroidX - cx - zoom.x)
        val y = centroidY + panY - cy - ratio * (centroidY - cy - zoom.y)
        return limit(Zoom(newScale, x, y), frame)
    }

    /** Double tap: from fit, zoom in around the tap; from anywhere else, go back to fit. */
    fun doubleTap(zoom: Zoom, tapX: Float, tapY: Float, frame: Frame): Zoom =
        if (zoom.isFit) transform(Zoom(), tapX, tapY, 0f, 0f, DOUBLE_TAP_SCALE, frame) else Zoom()

    fun shouldDismiss(dragY: Float, viewH: Float): Boolean = abs(dragY) > viewH * DISMISS_FRACTION

    /** The black backdrop fades while the picture is pulled away, but never disappears completely. */
    fun backdropAlpha(dragY: Float, viewH: Float): Float =
        (1f - abs(dragY) / (viewH * FADE_FRACTION)).coerceIn(MIN_BACKDROP, 1f)
}
