/* Offline memory wallet; shipped copy of the host local policy. No account, storage, or API access. */
(() => {
"use strict";
"use strict";
function snakeEconomyDay(now = new Date()) {
    // Fixed product day, matching the backend and independent of the phone's timezone.
    return new Date(now.getTime() + 8 * 3600_000).toISOString().slice(0, 10);
}
function snakeCoinsForScore(score) { return score < 100 ? 0 : Math.min(80, 5 + Math.floor(score / 120)); }
function newSnakeEconomyState(day = snakeEconomyDay()) {
    return { version: 1, coins: 40, shields: 0, startShieldSeconds: 0, day, earnedToday: 0, sharedDay: null, sessions: {}, requests: {} };
}
function parseSnakeEconomyState(raw) {
    if (!raw)
        return null;
    try {
        const state = JSON.parse(raw);
        if (state.version !== 1 || !Number.isSafeInteger(state.coins) || state.coins < 0
            || !Number.isSafeInteger(state.shields) || state.shields < 0 || ![0, 5].includes(state.startShieldSeconds)
            || typeof state.day !== 'string' || !/^\d{4}-\d{2}-\d{2}$/.test(state.day)
            || !Number.isInteger(state.earnedToday) || state.earnedToday < 0 || state.earnedToday > 300
            || !(state.sharedDay === null || typeof state.sharedDay === 'string')
            || !state.sessions || typeof state.sessions !== 'object' || Array.isArray(state.sessions)
            || !state.requests || typeof state.requests !== 'object' || Array.isArray(state.requests))
            return null;
        for (const session of Object.values(state.sessions)) {
            if (!session || !['revived', 'consumed', 'settled', 'finished'].every((key) => typeof session[key] === 'boolean')
                || !Number.isInteger(session.score) || session.score < 0 || session.score > 1e9)
                return null;
        }
        for (const request of Object.values(state.requests)) {
            if (!request || !snakeWalletActions.includes(request.action) || !(request.sessionId === null || typeof request.sessionId === 'string')
                || !request.response || !request.response.wallet || !Number.isInteger(request.response.coinsEarned)
                || request.response.coinsEarned < 0 || request.response.coinsEarned > 80 || ![0, 3, 5].includes(request.response.protectionSeconds))
                return null;
        }
        return state;
    }
    catch {
        return null;
    }
}
const snakeWalletActions = ['buy_shield', 'use_shield', 'revive', 'consume_start_shield', 'settle', 'share_reward'];
/** Pure local policy. Host supplies verified session state; it never receives an account token. */
function createSnakeMemoryWallet(initial, today = snakeEconomyDay) {
    let state = initial ? structuredClone(initial) : newSnakeEconomyState(today());
    function rollover() {
        const day = today();
        if (state.day !== day) {
            state.day = day;
            state.earnedToday = 0;
        }
    }
    function read() {
        rollover();
        return { coins: state.coins, shields: state.shields, startShieldSeconds: state.startShieldSeconds,
            shareAvailable: state.sharedDay !== state.day, prices: { shield: 20, revive: 30 } };
    }
    function act(action, options) {
        if (!snakeWalletActions.includes(action))
            throw new Error('没有这个糖果道具操作');
        if (!options || typeof options.requestId !== 'string' || !/^[A-Za-z0-9_.:-]{1,120}$/.test(options.requestId))
            throw new Error('道具请求编号不正确');
        const sessionId = options.sessionId ?? null;
        if (sessionId !== null && (typeof sessionId !== 'string' || !/^[A-Za-z0-9_.:-]{1,120}$/.test(sessionId)))
            throw new Error('游戏记录编号不正确');
        const prior = Object.hasOwn(state.requests, options.requestId) ? state.requests[options.requestId] : null;
        if (prior) {
            if (prior.action !== action || prior.sessionId !== sessionId)
                throw new Error('请求编号已用于其他操作');
            return structuredClone(prior.response);
        }
        rollover();
        // Work on a copy so rejected purchases do not partially mutate local state.
        const next = structuredClone(state);
        let round = null;
        if (action !== 'buy_shield' && sessionId === null)
            throw new Error('请先开始一局贪吃蛇');
        if (sessionId !== null) {
            round = Object.hasOwn(next.sessions, sessionId) ? next.sessions[sessionId] : { revived: false, consumed: false, settled: false, finished: false, score: 0 };
            if (options.finished === true)
                round.finished = true;
            if (options.score !== undefined) {
                if (!Number.isInteger(options.score) || options.score < 0 || options.score > 1e9)
                    throw new Error('游戏成绩不正确');
                round.score = Math.max(round.score, options.score);
            }
            Object.defineProperty(next.sessions, sessionId, { value: round, writable: true, enumerable: true, configurable: true });
        }
        if (['use_shield', 'revive', 'consume_start_shield'].includes(action) && round.finished)
            throw new Error('这一局已经结束啦');
        if (['settle', 'share_reward'].includes(action) && !round.finished)
            throw new Error('本局结束后才能领取奖励');
        let coinsEarned = 0;
        let protectionSeconds = 0;
        if (action === 'buy_shield') {
            if (next.coins < 20)
                throw new Error('糖果币不够啦');
            next.coins -= 20;
            next.shields += 1;
        }
        else if (action === 'use_shield') {
            if (next.shields < 1)
                throw new Error('先兑换一张保护卡吧');
            next.shields -= 1;
            protectionSeconds = 3;
        }
        else if (action === 'revive') {
            if (round.revived)
                throw new Error('每局只能续命一次');
            if (next.coins < 30)
                throw new Error('糖果币不够啦');
            next.coins -= 30;
            round.revived = true;
            protectionSeconds = 5;
        }
        else if (action === 'consume_start_shield' && !round.consumed) {
            protectionSeconds = next.startShieldSeconds;
            next.startShieldSeconds = 0;
            round.consumed = true;
        }
        else if (action === 'settle' && !round.settled) {
            coinsEarned = Math.min(300 - next.earnedToday, snakeCoinsForScore(round.score));
            next.coins += coinsEarned;
            next.earnedToday += coinsEarned;
            round.settled = true;
        }
        else if (action === 'share_reward' && next.sharedDay !== next.day) {
            next.sharedDay = next.day;
            next.startShieldSeconds = 5;
        }
        state = next;
        const response = { wallet: read(), coinsEarned, protectionSeconds };
        Object.defineProperty(state.requests, options.requestId, { value: { action, sessionId, response: structuredClone(response) },
            writable: true, enumerable: true, configurable: true });
        return response;
    }
    return { read, act, snapshot: () => structuredClone(state) };
}

globalThis.XiabanSnakeEconomy = { createMemoryWallet() { const wallet = createSnakeMemoryWallet(); return { read: async () => wallet.read(), act: async (action, options) => wallet.act(action, options) }; } };
})();
