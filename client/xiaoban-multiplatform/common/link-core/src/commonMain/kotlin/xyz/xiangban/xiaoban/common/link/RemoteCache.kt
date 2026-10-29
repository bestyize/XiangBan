package xyz.xiangban.xiaoban.common.link

/** 本地缓存只保存电脑列表、会话列表和消息，不保存待审批或待执行指令。 */
interface RemoteCache {
    fun peers(account: String): List<PeerSummary>
    fun savePeers(account: String, peers: List<PeerSummary>)
    fun sessions(account: String, peer: String): List<RemoteSessionSummary>
    fun saveSessions(account: String, peer: String, sessions: List<RemoteSessionSummary>)
    fun messages(account: String, peer: String, session: String): List<RemoteTurn>?
    fun saveMessages(account: String, peer: String, session: String, messages: List<RemoteTurn>)
    fun removePeer(account: String, peer: String)
    fun clearAccount(account: String)
}

class InMemoryRemoteCache : RemoteCache {
    private val peerLists = mutableMapOf<String, List<PeerSummary>>()
    private val sessionLists = mutableMapOf<Pair<String, String>, List<RemoteSessionSummary>>()
    private val conversations = mutableMapOf<Triple<String, String, String>, List<RemoteTurn>>()
    override fun peers(account: String) = peerLists[account].orEmpty()
    override fun savePeers(account: String, peers: List<PeerSummary>) { peerLists[account] = peers }
    override fun sessions(account: String, peer: String) = sessionLists[account to peer].orEmpty()
    override fun saveSessions(account: String, peer: String, sessions: List<RemoteSessionSummary>) { sessionLists[account to peer] = sessions }
    override fun messages(account: String, peer: String, session: String) = conversations[Triple(account, peer, session)]
    override fun saveMessages(account: String, peer: String, session: String, messages: List<RemoteTurn>) { conversations[Triple(account, peer, session)] = messages }
    override fun removePeer(account: String, peer: String) {
        peerLists[account] = peers(account).filterNot { it.id == peer }
        sessionLists.remove(account to peer)
        conversations.keys.removeAll { it.first == account && it.second == peer }
    }
    override fun clearAccount(account: String) {
        peerLists.remove(account)
        sessionLists.keys.removeAll { it.first == account }
        conversations.keys.removeAll { it.first == account }
    }
}
