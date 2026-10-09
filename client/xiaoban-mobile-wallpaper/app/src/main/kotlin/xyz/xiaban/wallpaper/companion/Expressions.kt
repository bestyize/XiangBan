package xyz.xiaban.wallpaper.companion

import xyz.xiaban.wallpaper.companion.Glyph.CHEVRON
import xyz.xiaban.wallpaper.companion.Glyph.HEART
import xyz.xiaban.wallpaper.companion.Symbol.*

private fun Eye.smile() { span = 0.5; arcMid = -90.0; w = 22.0; h = 16.0; thick = 0.5; y = 5.0 }
private fun Eye.solid() { thick = 1.0 }
private fun Mouth.cat(width: Double, depth: Double) { w = width; cat = depth; curve = 0.0; open = 0.0; tongue = 0.0 }
private fun Mouth.frown(width: Double, depth: Double) { w = width; curve = -depth; cat = 0.0; open = 0.0 }
private fun Mouth.openMouth(width: Double, amount: Double) { w = width; cat = 2.6; curve = 0.0; open = amount; tongue = 1.0 }
private fun Eye.pupilAt(pupilSize: Double, px: Double, py: Double) { solid(); pupil = pupilSize; pupilX = px; pupilY = py }

/** The 25 base expressions, ordered as in the design sheet. */
enum class Expression(val label: String, private val look: Look = Look(), face: Face.() -> Unit = {}) {
    IDLE("平静"),
    HAPPY("开心", Look(blink = false), { both { smile() }; blush = 0.9 }),
    EXCITED(
        "兴奋", Look(CHEVRON, CHEVRON, SPARKS, blink = false, saccade = false),
        { both { w = 22.0; h = 21.0; thick = 0.38 }; mouth.openMouth(8.0, 9.0); blush = 0.9; bounce = 2.5; bounceHz = 2.2; earL = -6.0; earR = -6.0 },
    ),
    LAUGHING(
        "大笑", Look(CHEVRON, CHEVRON, SPARKS, blink = false, saccade = false),
        { both { w = 23.0; h = 22.0; thick = 0.38 }; mouth.openMouth(10.0, 13.0); blush = 1.0; bounce = 4.5; bounceHz = 3.2; earL = -8.0; earR = -8.0 },
    ),
    PROUD(
        "得意", Look(glyphR = CHEVRON, symbol = SPARKLE, blink = false),
        { eyeR.w = 22.0; eyeR.h = 21.0; eyeR.thick = 0.38; mouth.cat(8.0, 3.4); tilt = -4.0; earL = -4.0 },
    ),
    CURIOUS(
        "好奇", Look(symbol = QUESTION),
        {
            both { pupilAt(0.56, 0.44, -0.16) }
            mouth.w = 3.6; mouth.curve = 0.0; mouth.cat = 0.0; mouth.open = 3.4; mouth.tongue = 0.0; mouth.x = 2.0
            tilt = 6.0; gazeX = 3.0; earR = -8.0
        },
    ),
    THINKING(
        "思考", Look(symbol = DOTS, saccade = false),
        {
            both { pupilAt(0.34, 0.2, 0.42); w = 22.0; h = 19.0; lidTop = -0.12; y = 4.0 }
            mouth.w = 7.0; mouth.curve = 0.0; mouth.cat = 0.0; mouth.x = 3.0
            gazeX = 4.0; gazeY = -3.0; tilt = -3.0
        },
    ),
    CONFUSED(
        "困惑", Look(symbol = QUESTION),
        {
            eyeL.pupilAt(0.45, 0.3, 0.15); eyeL.lidTop = -0.35; eyeL.lidTilt = -0.3
            eyeR.pupilAt(0.5, -0.1, 0.15); eyeR.lidTop = -0.55; eyeR.lidTilt = 0.25
            mouth.frown(5.0, 3.0); tilt = -7.0; earL = 6.0
        },
    ),
    SURPRISED(
        "惊讶", Look(symbol = EXCLAIM, blink = false, saccade = false),
        {
            both { w = 26.0; h = 27.0; thick = 0.3 }
            mouth.w = 3.4; mouth.curve = 0.0; mouth.cat = 0.0; mouth.open = 4.0; mouth.tongue = 0.0
            lift = -3.0; earL = -10.0; earR = -10.0
        },
    ),
    SHY(
        "害羞", Look(blink = false),
        { both { smile(); w = 20.0; h = 14.0 }; mouth.cat(7.0, 3.4); blush = 0.85; blushLines = 1.0; gazeY = 3.0; tilt = 5.0; earL = 6.0; earR = 6.0 },
    ),
    LOVE(
        "喜爱", Look(HEART, HEART, blink = false),
        { both { w = 23.0 }; mouth.cat(10.0, 4.5); blush = 0.7; blushLines = 0.9; bounce = 1.4; bounceHz = 1.3 },
    ),
    SLEEPY(
        "困倦", Look(symbol = BUBBLE, blink = false, saccade = false),
        { both { solid(); w = 25.0; h = 15.0; lidTop = 0.05; y = 2.0 }; mouth.cat(5.0, 2.6); earL = 10.0; earR = 10.0; earGlow = 0.7; tilt = 3.0 },
    ),
    SLEEPING(
        "睡眠", Look(symbol = ZZZ, blink = false, saccade = false),
        { both { span = 0.5; arcMid = 90.0; w = 21.0; h = 14.0; thick = 0.5; y = -2.0 }; mouth.cat(8.0, 3.6); earL = 14.0; earR = 14.0; earGlow = 0.45; lift = 2.0 },
    ),
    SAD(
        "难过",
        face = {
            eyeL.pupilAt(0.5, 0.32, 0.2); eyeL.lidTop = -0.45; eyeL.lidTilt = -0.5
            eyeR.pupilAt(0.5, -0.32, 0.2); eyeR.lidTop = -0.45; eyeR.lidTilt = -0.5
            mouth.frown(6.0, 3.4); gazeY = 3.0; earL = 12.0; earR = 12.0; earGlow = 0.75
        },
    ),
    WORRIED(
        "担心",
        face = {
            eyeL.pupilAt(0.46, 0.26, 0.24); eyeL.lidTop = -0.3; eyeL.lidTilt = -0.7
            eyeR.pupilAt(0.46, -0.26, 0.24); eyeR.lidTop = -0.3; eyeR.lidTilt = -0.7
            mouth.frown(6.0, 3.0); tilt = -3.0; earL = 8.0; earR = 8.0
        },
    ),
    ANGRY(
        "生气", Look(symbol = ANGER, saccade = false),
        { both { solid(); w = 25.0; h = 21.0; lidTop = -0.2; lidTilt = 0.8 }; mouth.frown(7.0, 3.4); earL = -12.0; earR = -12.0 },
    ),
    ANNOYED(
        "不耐烦", Look(symbol = ANGER, saccade = false),
        { both { pupilAt(0.44, 0.34, 0.38); w = 24.0; h = 18.0; lidTop = -0.05; lidTilt = 0.12 }; mouth.frown(6.0, 2.6); gazeX = 3.0 },
    ),
    CRYING(
        "哭泣", Look(blink = false, saccade = false),
        { both { span = 0.42; arcMid = -90.0; w = 20.0; h = 13.0; thick = 0.55; y = 2.0 }; mouth.frown(6.0, 3.0); tears = 1.0; earL = 14.0; earR = 14.0; earGlow = 0.7 },
    ),
    EMBARRASSED(
        "尴尬", Look(symbol = SWEAT),
        { both { w = 21.0; h = 21.0; thick = 0.34 }; mouth.cat(9.0, 3.2); blush = 0.8; blushLines = 1.0; gazeX = -3.0; tilt = -3.0 },
    ),
    LISTENING(
        "倾听", Look(symbol = LISTEN, saccade = false),
        { both { pupilAt(0.3, 0.36, -0.38); w = 22.0; h = 23.0 }; mouth.cat(5.0, 2.4); tilt = 8.0; earR = -12.0; earL = 4.0 },
    ),
    SPEAKING(
        "说话", Look(symbol = SOUND, blink = false),
        { both { smile() }; mouth.openMouth(8.0, 6.0); talk = 0.7 },
    ),
    GREETING(
        "打招呼", Look(symbol = WAVE, blink = false),
        { both { smile() }; mouth.openMouth(8.0, 8.0); bounce = 1.6; bounceHz = 1.6; tilt = -5.0; earL = -10.0 },
    ),
    WAITING(
        "等待", Look(blink = false, saccade = false),
        { both { dash = 1.0; thick = 0.3 }; mouth.cat(7.0, 3.0); spin = 0.35 },
    ),
    OFFLINE(
        "离线", Look(symbol = BAN, blink = false, saccade = false),
        { both { w = 20.0; h = 0.0; thick = 0.36 }; mouth.cat(6.0, 2.6); screen = 0.5; earGlow = 0.22; earL = 10.0; earR = 10.0 },
    ),
    CHARGING(
        "充电", Look(symbol = BOLT, blink = false, saccade = false),
        { both { smile() }; mouth.cat(7.0, 3.0); glowPulse = 0.6; earGlow = 0.95 },
    );

    private val patch = Face.blank().also(face)

    /** What the face and discrete look should settle into. The face is a fresh copy. */
    val targetFace: Face get() = Face.base().also { it.apply(patch) }
    val targetLook: Look get() = look
}
