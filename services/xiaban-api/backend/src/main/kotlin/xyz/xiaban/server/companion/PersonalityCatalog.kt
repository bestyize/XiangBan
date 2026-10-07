package xyz.xiaban.server.companion

import xyz.xiaban.contracts.companion.personality.Personality

/** Display names belong to the service's content policy, not the wire enum. */
val Personality.label: String
    get() = when (this) {
        Personality.Toxic -> "毒舌心软"
        Personality.Gentle -> "温柔治愈"
        Personality.Chatty -> "元气话痨"
        Personality.Tsundere -> "高冷傲娇"
    }
