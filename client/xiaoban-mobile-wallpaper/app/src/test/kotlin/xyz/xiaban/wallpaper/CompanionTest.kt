package xyz.xiaban.wallpaper

import org.junit.Assert.assertEquals
import org.junit.Assert.assertFalse
import org.junit.Assert.assertTrue
import xyz.xiaban.wallpaper.companion.Animator
import xyz.xiaban.wallpaper.companion.Expression
import xyz.xiaban.wallpaper.companion.Frame
import xyz.xiaban.wallpaper.companion.Glyph
import xyz.xiaban.wallpaper.companion.Mind
import xyz.xiaban.wallpaper.companion.Symbol
import org.junit.Test

private fun run(animator: Animator, seconds: Double, onFrame: (Frame) -> Unit = {}): Frame {
    var frame = animator.step(0.0)
    var t = 0.0
    while (t < seconds) {
        frame = animator.step(1.0 / 60)
        onFrame(frame)
        t += 1.0 / 60
    }
    return frame
}

class CompanionTest {
    @Test
    fun hasAllTwentyFiveExpressionsWithFiniteTargets() {
        assertEquals(25, Expression.entries.size)
        for (expression in Expression.entries) {
            assertTrue(expression.name, expression.targetFace.a.all { it.isFinite() })
        }
    }

    @Test
    fun springsIntoNewExpressionAndSettles() {
        val animator = Animator(seed = 1, reducedMotion = true)
        animator.setTarget(Expression.SURPRISED)
        assertTrue(animator.step(1.0 / 60).face.eyeL.w < 25)
        val frame = run(animator, 2.0)
        assertEquals(26.0, frame.face.eyeL.w, 0.1)
        assertEquals(4.0, frame.face.mouth.open, 0.1)
        assertEquals(Symbol.EXCLAIM, frame.symbol)
        assertEquals(1.0, frame.symbolP, 0.0)
    }

    @Test
    fun squashesEyesShutWhileSwappingGlyph() {
        val animator = Animator(seed = 1, reducedMotion = true)
        animator.setTarget(Expression.LOVE)
        var minSquash = 1.0
        val frame = run(animator, 1.0) { minSquash = minOf(minSquash, it.squashL) }
        assertTrue(minSquash < 0.2)
        assertEquals(Glyph.HEART, frame.glyphL)
        assertEquals(1.0, frame.squashL, 0.0)
    }

    @Test
    fun blinksOnItsOwnButNotWithReducedMotion() {
        var blinks = 0
        var closed = false
        run(Animator(seed = 7), 20.0) {
            if (it.squashL < 0.3 && !closed) blinks++
            closed = it.squashL < 0.3
        }
        assertTrue(blinks >= 3)
        var calmBlinks = 0
        run(Animator(seed = 7, reducedMotion = true), 20.0) { if (it.squashL < 0.3) calmBlinks++ }
        assertEquals(0, calmBlinks)
    }

    @Test
    fun movesMouthWhileSpeaking() {
        val animator = Animator(seed = 3, reducedMotion = true)
        animator.setTarget(Expression.SPEAKING)
        run(animator, 1.0)
        val opens = mutableListOf<Double>()
        run(animator, 1.0) { opens += it.face.mouth.open }
        assertTrue(opens.max() - opens.min() > 3)
    }

    @Test
    fun sameSeedGivesSameMotion() {
        val a = Animator(seed = 42)
        val b = Animator(seed = 42)
        val framesA = mutableListOf<Double>()
        val framesB = mutableListOf<Double>()
        run(a, 10.0) { framesA += it.face.gazeX + it.face.earL + it.squashL }
        run(b, 10.0) { framesB += it.face.gazeX + it.face.earL + it.squashL }
        assertEquals(framesA, framesB)
    }

    @Test
    fun neverProducesNaNOverLongBusyRun() {
        val animator = Animator(seed = 11)
        var i = 0
        run(animator, 30.0) {
            if (++i % 90 == 0) animator.setTarget(Expression.entries[(i / 90) % Expression.entries.size])
            assertFalse(it.face.a.any { v -> v.isNaN() })
        }
    }

    private fun mind(): Mind =
        Mind(Animator(seed = 5, reducedMotion = true), autoSleep = true, sleepyAfter = 10.0, sleepAfter = 20.0, rand = { 0.0 })

    @Test
    fun dozesOffWhenLeftAloneAndWakesUpSurprised() {
        val mind = mind()
        mind.tick(11.0)
        assertEquals(Expression.SLEEPY, mind.current())
        mind.tick(10.0)
        assertEquals(Expression.SLEEPING, mind.current())
        mind.activity()
        assertEquals(Expression.SURPRISED, mind.current())
        mind.tick(0.5)
        mind.tick(0.8)
        assertEquals(Expression.IDLE, mind.current())
    }

    @Test
    fun reactsToPokesAndGetsAnnoyedWhenPokedTooOften() {
        val mind = mind()
        mind.poke()
        assertEquals(Expression.SURPRISED, mind.current())
        repeat(4) {
            mind.tick(0.1)
            mind.poke()
        }
        assertEquals(Expression.ANNOYED, mind.current())
    }

    @Test
    fun showsTheChosenBaseExpressionWhenIdle() {
        val mind = mind()
        mind.setBase(Expression.WORRIED)
        assertEquals(Expression.WORRIED, mind.current())
    }
}
