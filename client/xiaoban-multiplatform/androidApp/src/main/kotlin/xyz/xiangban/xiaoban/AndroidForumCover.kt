package xyz.xiangban.xiaoban

import android.graphics.Bitmap
import android.graphics.BitmapFactory
import android.graphics.Canvas
import android.graphics.Matrix
import android.media.ExifInterface
import android.graphics.Path
import android.graphics.RectF
import android.graphics.Paint
import android.graphics.Typeface
import android.text.SpannableString
import android.text.Spanned
import android.text.style.ForegroundColorSpan
import android.text.Layout
import android.text.StaticLayout
import android.text.TextPaint
import androidx.compose.ui.graphics.asImageBitmap
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.withContext
import xyz.xiangban.xiaoban.business.communitynative.ForumCoverImage
import xyz.xiangban.xiaoban.business.communitynative.ForumCoverSpec
import xyz.xiangban.xiaoban.business.communitynative.ForumCoverTemplate
import xyz.xiangban.xiaoban.common.community.FORUM_IMAGE_MAX_BYTES
import xyz.xiangban.xiaoban.common.community.ForumUploadFile
import xyz.xiangban.xiaoban.common.community.validateForumUpload
import java.io.ByteArrayInputStream
import java.io.ByteArrayOutputStream

/** Code-drawn typography, paper and doodles. Preview and export use the same renderer. */
internal suspend fun renderLocalForumCover(spec: ForumCoverSpec, thumbnail: Boolean = false): ForumCoverImage = withContext(Dispatchers.Default) {
    require(spec.title.isNotBlank() && spec.title.length <= 80 && spec.subtitle.length <= 100 && spec.highlight.length <= 16) { "标题最多 80 字，副标题最多 100 字，重点词最多 16 字" }
    require(spec.highlight.isEmpty() || spec.title.contains(spec.highlight)) { "重点词需要是标题中的文字" }
    val h = if (spec.portrait) 1200f else 900f
    val scale = if (thumbnail) .25f else 1f
    val bitmap = Bitmap.createBitmap((900 * scale).toInt(), (h * scale).toInt(), Bitmap.Config.ARGB_8888)
    try {
        val canvas = Canvas(bitmap)
        canvas.scale(scale, scale)
        drawCover(canvas, h, spec)
        val bytes = ByteArrayOutputStream().use { out -> check(bitmap.compress(Bitmap.CompressFormat.PNG, 100, out)); out.toByteArray() }
        ForumCoverImage(ForumUploadFile("文字封面.png", "image/png", bytes), bitmap.asImageBitmap())
    } catch (e: Throwable) { bitmap.recycle(); throw e }
}

private val CoverInk = 0xff20201e.toInt()
private val CoverRed = 0xffdd443b.toInt()
private val CoverYellow = 0xffffdb43.toInt()
private val CoverBlue = 0xff3067bb.toInt()

private fun drawCover(c: Canvas, h: Float, spec: ForumCoverSpec) {
    val p = Paint(Paint.ANTI_ALIAS_FLAG)
    fun rect(l: Float, t: Float, r: Float, b: Float, color: Int, radius: Float = 0f) {
        p.color = color; p.style = Paint.Style.FILL
        c.drawRoundRect(l, t, r, b, radius, radius, p)
    }
    fun line(x: Float, y: Float, x2: Float, y2: Float, color: Int, width: Float = 2f) {
        p.color = color; p.style = Paint.Style.STROKE; p.strokeWidth = width
        c.drawLine(x, y, x2, y2, p); p.style = Paint.Style.FILL
    }
    var textX = 95f
    var textWidth = 710
    var centered = false
    c.save()
    when (spec.template) {
        ForumCoverTemplate.Marker -> {
            c.drawColor(0xfffffdf5.toInt())
            rect(70f, 75f, 270f, 130f, CoverInk, 28f)
            coverLabel(c, "灵 感 笔 记", 96f, 112f, 24f, -1)
            doodleStar(c, 780f, 120f, 35f, CoverRed)
            line(70f, h - 105f, 550f, h - 105f, CoverInk)
            doodleArrow(c, 720f, h - 150f, CoverRed)
        }
        ForumCoverTemplate.Memo -> {
            c.drawColor(0xffefe4c7.toInt())
            rect(78f, 100f, 844f, h - 62f, 0x24000000)
            val paper = Path().apply {
                moveTo(60f, 80f); lineTo(820f, 80f); lineTo(820f, h - 90f)
                for (x in 820 downTo 60 step 20) lineTo(x.toFloat(), h - if ((x / 20) % 2 == 0) 74f else 90f)
                close()
            }
            p.color = 0xfffffcf1.toInt(); c.drawPath(paper, p)
            for (y in 244 until (h - 110).toInt() step 62) line(90f, y.toFloat(), 790f, y.toFloat(), 0xffddd9cd.toInt())
            line(145f, 175f, 145f, h - 115f, 0xffedb3ad.toInt(), 3f)
            c.save(); c.rotate(-5f, 450f, 90f); rect(330f, 55f, 570f, 120f, 0xccddbd74.toInt()); c.restore()
            coverLabel(c, "MEMO", 176f, 193f, 32f, 0xff82755b.toInt())
            textX = 178f; textWidth = 585
        }
        ForumCoverTemplate.Notebook -> {
            c.drawColor(0xffbcdaf0.toInt())
            rect(80f, 116f, 834f, h - 64f, 0x18244260)
            rect(65f, 98f, 820f, h - 82f, 0xfffffefa.toInt(), 14f)
            for (x in 95..800 step 42) line(x.toFloat(), 180f, x.toFloat(), h - 105f, 0xffe6ecef.toInt(), 1f)
            for (y in 180 until (h - 100).toInt() step 42) line(85f, y.toFloat(), 802f, y.toFloat(), 0xffe6ecef.toInt(), 1f)
            for (x in 140..760 step 103) {
                rect(x - 13f, 134f, x + 13f, 158f, 0xffb5d2e5.toInt(), 4f)
                p.color = 0xff809aae.toInt(); p.style = Paint.Style.STROKE; p.strokeWidth = 7f
                c.drawRoundRect(x - 5f, 63f, x + 12f, 148f, 9f, 9f, p)
                p.style = Paint.Style.FILL
            }
            doodleStar(c, 740f, h - 155f, 29f, CoverYellow)
            textX = 132f; textWidth = 620; centered = true
        }
        ForumCoverTemplate.Pop -> {
            c.drawColor(0xffddf56b.toInt())
            p.color = CoverInk; p.style = Paint.Style.STROKE; p.strokeWidth = 4f
            c.drawRect(34f, 34f, 866f, h - 34f, p); p.style = Paint.Style.FILL
            rect(70f, 78f, 352f, 135f, CoverInk)
            coverLabel(c, "IDEA / SHARE", 92f, 117f, 29f, -1)
            doodleStar(c, 770f, 120f, 44f, CoverInk)
            for (i in 0..17) if (i % 2 == 0) rect(70f + i * 42, h - 105f, 112f + i * 42, h - 77f, CoverInk)
            textX = 90f; textWidth = 720
        }
        ForumCoverTemplate.Collage -> {
            c.drawColor(0xfff3bdad.toInt())
            for (x in 0..900 step 60) for (y in 0..h.toInt() step 60) {
                p.color = 0x30ffffff; c.drawCircle(x.toFloat(), y.toFloat(), 3f, p)
            }
            c.rotate(-2.5f, 450f, h / 2)
            rect(82f, 102f, 842f, h - 70f, 0x20000000)
            rect(62f, 80f, 822f, h - 90f, 0xfffff9e9.toInt())
            c.save(); c.rotate(7f, 450f, 100f); rect(330f, 48f, 568f, 124f, 0xbbaec8a1.toInt()); c.restore()
            coverLabel(c, "把灵感留在这里", 116f, 190f, 26f, 0xff8d6c57.toInt())
            doodleStar(c, 735f, h - 158f, 32f, CoverRed)
            textX = 116f; textWidth = 650
        }
        ForumCoverTemplate.Editorial -> {
            c.drawColor(0xfff6f0e5.toInt())
            line(65f, 70f, 835f, 70f, CoverInk, 7f)
            coverLabel(c, "CREATIVE NOTES", 70f, 123f, 35f, CoverInk)
            line(65f, 147f, 835f, 147f, CoverInk, 2f)
            rect(70f, 184f, 146f, 203f, CoverRed)
            line(65f, h - 145f, 835f, h - 145f, CoverInk, 2f)
            coverLabel(c, "IDEAS WORTH SHARING", 70f, h - 89f, 23f, CoverInk)
            rect(783f, h - 119f, 830f, h - 72f, CoverRed)
            textX = 76f; textWidth = 748
        }
    }
    val align = if (centered) Layout.Alignment.ALIGN_CENTER else Layout.Alignment.ALIGN_NORMAL
    val subtitle = spec.subtitle.takeIf { it.isNotBlank() }?.let { fitCoverText(it, textWidth, 148, 33f, CoverInk, false, align) }
    val regionTop = 245f
    val regionBottom = h - 205f
    val titleMax = (regionBottom - regionTop - (subtitle?.height?.plus(50) ?: 0)).toInt()
    val raw = spec.title.trim()
    val initial = fitCoverText(raw, textWidth, titleMax, 146f, CoverInk, true, align)
    val start = if (spec.highlight.isNotEmpty()) raw.indexOf(spec.highlight) else initial.getLineStart(initial.lineCount - 1)
    val end = if (spec.highlight.isNotEmpty()) start + spec.highlight.length else raw.length
    val accent = when (spec.template) {
        ForumCoverTemplate.Notebook -> CoverBlue
        ForumCoverTemplate.Pop -> -1
        ForumCoverTemplate.Collage, ForumCoverTemplate.Editorial -> CoverRed
        else -> CoverInk
    }
    val decorated = SpannableString(raw).apply { if (start >= 0 && end > start) setSpan(ForegroundColorSpan(accent), start, end, Spanned.SPAN_EXCLUSIVE_EXCLUSIVE) }
    val title = fitCoverText(decorated, textWidth, titleMax, initial.paint.textSize, CoverInk, true, align)
    val blockHeight = title.height + (subtitle?.height?.plus(50) ?: 0)
    val top = regionTop + (regionBottom - regionTop - blockHeight) / 2
    c.save(); c.translate(textX, top)
    for (i in 0 until title.lineCount) {
        val from = maxOf(start, title.getLineStart(i))
        val to = minOf(end, title.getLineEnd(i))
        if (from >= to) continue
        val left = title.getPrimaryHorizontal(from)
        val right = if (to >= title.getLineEnd(i)) title.getLineRight(i) else title.getPrimaryHorizontal(to)
        val baseline = title.getLineBaseline(i).toFloat()
        val box = RectF(left - 10f, baseline + title.paint.fontMetrics.ascent - 4f, right + 10f, baseline + title.paint.fontMetrics.descent + 7f)
        when (spec.template) {
            ForumCoverTemplate.Marker, ForumCoverTemplate.Memo -> {
                p.color = CoverYellow; p.style = Paint.Style.FILL
                c.save(); c.rotate(-1.3f, box.centerX(), box.centerY()); c.drawRect(box.left, box.top + box.height() * .15f, box.right, box.bottom, p); c.restore()
            }
            ForumCoverTemplate.Pop -> { p.color = CoverInk; p.style = Paint.Style.FILL; c.drawRect(box, p) }
            ForumCoverTemplate.Notebook -> {
                p.color = 0xffe98ebc.toInt(); p.style = Paint.Style.STROKE; p.strokeWidth = 6f
                c.save(); c.rotate(-3f, box.centerX(), box.centerY()); c.drawOval(box, p); c.restore(); p.style = Paint.Style.FILL
            }
            ForumCoverTemplate.Collage -> {
                p.color = CoverRed; p.style = Paint.Style.STROKE; p.strokeWidth = 5f
                val path = Path().apply { moveTo(box.left, box.bottom + 4); quadTo(box.centerX(), box.bottom - 4, box.right, box.bottom + 5) }
                c.drawPath(path, p); p.style = Paint.Style.FILL
            }
            ForumCoverTemplate.Editorial -> Unit
        }
    }
    title.draw(c)
    subtitle?.let { c.translate(0f, title.height + 50f); it.draw(c) }
    c.restore(); c.restore()
}

private fun coverLabel(c: Canvas, text: String, x: Float, y: Float, size: Float, color: Int) {
    c.drawText(text, x, y, Paint(Paint.ANTI_ALIAS_FLAG).apply { this.color = color; textSize = size; typeface = Typeface.create("sans-serif", Typeface.BOLD) })
}

private fun doodleStar(c: Canvas, x: Float, y: Float, size: Float, color: Int) {
    val path = Path().apply {
        moveTo(x, y - size); lineTo(x + size * .24f, y - size * .24f); lineTo(x + size, y)
        lineTo(x + size * .24f, y + size * .24f); lineTo(x, y + size)
        lineTo(x - size * .24f, y + size * .24f); lineTo(x - size, y); lineTo(x - size * .24f, y - size * .24f); close()
    }
    c.drawPath(path, Paint(Paint.ANTI_ALIAS_FLAG).apply { this.color = color })
}

private fun doodleArrow(c: Canvas, x: Float, y: Float, color: Int) {
    val path = Path().apply { moveTo(x - 52, y + 48); cubicTo(x + 65, y + 80, x + 70, y - 5, x + 10, y - 15); moveTo(x + 37, y - 35); lineTo(x + 7, y - 15); lineTo(x + 31, y + 5) }
    c.drawPath(path, Paint(Paint.ANTI_ALIAS_FLAG).apply { this.color = color; style = Paint.Style.STROKE; strokeWidth = 7f; strokeCap = Paint.Cap.ROUND; strokeJoin = Paint.Join.ROUND })
}

private fun fitCoverText(text: CharSequence, width: Int, maxHeight: Int, initialSize: Float, color: Int, bold: Boolean, alignment: Layout.Alignment): StaticLayout {
    var size = initialSize
    while (size >= 22f) {
        val paint = TextPaint(Paint.ANTI_ALIAS_FLAG).apply {
            this.color = color; textSize = size
            typeface = Typeface.create(if (bold) "sans-serif-black" else "sans-serif", if (bold) Typeface.BOLD else Typeface.NORMAL)
            if (bold) { style = Paint.Style.FILL_AND_STROKE; strokeWidth = size * .015f }
        }
        // Preserve intentional short headline lines instead of wrapping a final character alone.
        val explicitLines = text.toString().split('\n')
        if (bold && explicitLines.size > 1 && explicitLines.all { it.length <= 10 } && explicitLines.any { paint.measureText(it) > width - 8 }) {
            size -= 2f
            continue
        }
        val layout = StaticLayout.Builder.obtain(text, 0, text.length, paint, width)
            .setAlignment(alignment).setIncludePad(false).setLineSpacing(size * .17f, 1f).build()
        if (layout.height <= maxHeight) return layout
        size -= 2f
    }
    error("换行太多，请精简封面文字")
}

internal suspend fun previewLocalForumCover(file: ForumUploadFile): ForumCoverImage = withContext(Dispatchers.Default) {
    validateForumUpload(file, FORUM_IMAGE_MAX_BYTES, true)
    val options = BitmapFactory.Options().apply { inJustDecodeBounds = true }
    BitmapFactory.decodeByteArray(file.bytes, 0, file.bytes.size, options)
    require(options.outWidth > 0 && options.outHeight > 0) { "无法读取这张图片，请换一张" }
    options.inSampleSize = 1
    while (maxOf(options.outWidth, options.outHeight) / options.inSampleSize > 1600) options.inSampleSize *= 2
    options.inJustDecodeBounds = false
    val bitmap = BitmapFactory.decodeByteArray(file.bytes, 0, file.bytes.size, options) ?: error("无法读取图片")
    val matrix = Matrix()
    val orientation = runCatching { ExifInterface(ByteArrayInputStream(file.bytes)).getAttributeInt(ExifInterface.TAG_ORIENTATION, ExifInterface.ORIENTATION_NORMAL) }.getOrDefault(ExifInterface.ORIENTATION_NORMAL)
    when (orientation) {
        ExifInterface.ORIENTATION_FLIP_HORIZONTAL -> matrix.setScale(-1f, 1f)
        ExifInterface.ORIENTATION_ROTATE_180 -> matrix.setRotate(180f)
        ExifInterface.ORIENTATION_FLIP_VERTICAL -> matrix.setScale(1f, -1f)
        ExifInterface.ORIENTATION_TRANSPOSE -> { matrix.setRotate(90f); matrix.postScale(-1f, 1f) }
        ExifInterface.ORIENTATION_ROTATE_90 -> matrix.setRotate(90f)
        ExifInterface.ORIENTATION_TRANSVERSE -> { matrix.setRotate(-90f); matrix.postScale(-1f, 1f) }
        ExifInterface.ORIENTATION_ROTATE_270 -> matrix.setRotate(-90f)
    }
    val preview = if (matrix.isIdentity) bitmap else Bitmap.createBitmap(bitmap, 0, 0, bitmap.width, bitmap.height, matrix, true).also { if (it !== bitmap) bitmap.recycle() }
    ForumCoverImage(file, preview.asImageBitmap())
}
