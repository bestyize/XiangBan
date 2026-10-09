package xyz.xiaban.wallpaper.companion

import xyz.xiaban.wallpaper.companion.Expression.*

class Beat(val expression: Expression, val hold: Double)

private val REACTIONS = listOf(
    listOf(Beat(SURPRISED, 0.35), Beat(LAUGHING, 1.1)),
    listOf(Beat(SHY, 1.4)),
    listOf(Beat(LOVE, 1.3)),
    listOf(Beat(EXCITED, 1.0)),
    listOf(Beat(HAPPY, 0.6), Beat(PROUD, 0.9)),
)
private val TOO_MANY_POKES = listOf(Beat(ANNOYED, 0.9), Beat(ANGRY, 1.2))
private val WAKE_UP = listOf(Beat(SURPRISED, 0.45), Beat(HAPPY, 0.7))

/**
 * Picks what the robot should be showing right now. Priority: a playing reaction, then dozing off,
 * then the base expression the user chose.
 */
class Mind(
    private val animator: Animator,
    private val autoSleep: Boolean = false,
    private val sleepyAfter: Double = 40.0,
    private val sleepAfter: Double = 90.0,
    private val rand: () -> Double = Math::random,
) {
    private var base = IDLE
    private val queue = ArrayDeque<Beat>()
    private var beatLeft = 0.0
    private var idle = 0.0
    private var pokes = listOf<Double>()
    private var clock = 0.0
    private var shown: Expression? = null

    val dozing get() = autoSleep && idle >= sleepyAfter

    fun setBase(expression: Expression) {
        base = expression
    }

    /** Something happened near the robot: reset the doze timer and wake it if needed. */
    fun activity() {
        val wasDozing = dozing
        idle = 0.0
        if (wasDozing && queue.isEmpty()) play(WAKE_UP)
    }

    fun poke() {
        val wasDozing = dozing
        idle = 0.0
        pokes = pokes.filter { clock - it < 3 } + clock
        animator.twitchEar(0, 0.8)
        if (wasDozing) return play(WAKE_UP)
        if (pokes.size >= 5) {
            pokes = emptyList()
            return play(TOO_MANY_POKES)
        }
        play(REACTIONS[(rand() * REACTIONS.size).toInt().coerceIn(0, REACTIONS.lastIndex)])
    }

    private fun play(beats: List<Beat>) {
        queue.clear()
        queue.addAll(beats)
        beatLeft = queue.firstOrNull()?.hold ?: 0.0
    }

    /** The expression that should be on screen now. */
    fun current(): Expression {
        queue.firstOrNull()?.let { return it.expression }
        if (autoSleep) {
            if (idle >= sleepAfter) return SLEEPING
            if (idle >= sleepyAfter) return SLEEPY
        }
        return base
    }

    fun tick(dt: Double) {
        clock += dt
        idle += dt
        if (queue.isNotEmpty()) {
            beatLeft -= dt
            if (beatLeft <= 0) {
                queue.removeFirst()
                beatLeft = queue.firstOrNull()?.hold ?: 0.0
            }
        }
        val next = current()
        if (next != shown) {
            shown = next
            animator.setTarget(next)
        }
    }
}
