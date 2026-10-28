package xyz.xiangban.xiaoban.common.community

/** 社区与账号服务的地址。账号中心要先登记 [clientId] 这个客户端，令牌的 audience 才会对得上论坛的校验。 */
data class CommunityConfig(
    val forumBase: String = "https://chat.xiangban.xyz",
    val accountBase: String = "https://account.xiangban.xyz",
    val clientId: String = "xiaoban-android",
    val deviceName: String = "小伴手机应用",
)
