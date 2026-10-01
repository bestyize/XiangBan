package xyz.xiaban.server.checkin

import kotlinx.coroutines.async
import kotlinx.coroutines.coroutineScope
import xyz.xiaban.server.calendar.HolidayCalendar
import xyz.xiaban.server.http.conflict
import xyz.xiaban.server.memory.MemoryKind
import xyz.xiaban.server.memory.MemoryService
import xyz.xiaban.server.meta.Cities
import xyz.xiaban.server.stats.PercentileService
import xyz.xiaban.server.stats.StatsService
import xyz.xiaban.server.stats.TipsService
import xyz.xiaban.server.unlocks.UnlockContext
import xyz.xiaban.server.unlocks.UnlockRules
import xyz.xiaban.server.unlocks.UnlockService
import xyz.xiaban.server.users.ProfileService
import xyz.xiaban.server.users.User
import xyz.xiaban.server.weather.WeatherInsights
import xyz.xiaban.server.weather.WeatherService
import xyz.xiaban.shared.CheckinView
import xyz.xiaban.shared.TodayResponse
import java.time.Clock
import java.time.Duration
import java.time.Instant
import java.time.LocalDateTime
import java.time.temporal.ChronoUnit

class CheckinService(
    private val profiles: ProfileService,
    private val checkins: CheckinRepository,
    private val calendar: HolidayCalendar,
    private val cities: Cities,
    private val planner: ReplyPlanner,
    private val memories: MemoryService,
    private val polisher: Polisher,
    private val weather: WeatherService,
    private val stats: StatsService,
    private val tips: TipsService,
    private val percentiles: PercentileService,
    private val views: CheckinViews,
    private val unlocks: UnlockService,
    private val clock: Clock,
) {
    suspend fun checkIn(userId: Long): CheckinView {
        val user = profiles.user(userId)
        val now = clock.instant()
        val local = LocalDateTime.ofInstant(now, user.timezone)
        val workDay = WorkDay.of(local)

        // 当天重复点击直接返回已有结果，不重新计算
        checkins.active(userId, workDay)?.let { return view(it, user, now) }

        // Only cached weather: check-in must not wait on the weather API (PRD 9.5).
        val city = cities.find(user.cityCode)
        val weatherTag = city?.let { weather.cached(it) }?.let { WeatherInsights.tag(it, now, user.timezone, workDay) }
        val title = TitleRules.compute(local, workDay, user.workEnd, calendar)
        val history = checkins.since(userId, workDay.minusDays(HISTORY_DAYS))
        val recentCopies = checkins.copyIdsSince(userId, workDay.minusDays(NO_REPEAT_DAYS))
        val plan = planner.plan(user, workDay, title, history, recentCopies, weatherTag)
        val created = checkins.insert(NewCheckin(userId, workDay, now, title, plan.copyId, plan.text, plan.careMode))
            ?: return view(checkins.active(userId, workDay)!!, user, now)

        percentiles.record(workDay, WorkDay.minuteOf(local), user.cityCode)
        val previous = history.firstOrNull()
        remember(created, plan, enteredCare = plan.careMode && previous?.careMode != true)
        city?.let { weather.prefetch(it) }

        val snapshot = stats.snapshot(user, now)
        val newUnlocks = unlocks.grant(
            userId,
            UnlockRules.earned(
                UnlockContext(
                    workDay = workDay,
                    title = title,
                    totalDays = snapshot.totalDays,
                    streak = snapshot.currentStreak,
                    weatherTag = weatherTag,
                    holidayCode = calendar.holidayOn(workDay)?.code,
                    daysSincePrevious = previous?.let { ChronoUnit.DAYS.between(it.workDay, workDay) },
                    previousWasCare = previous?.careMode == true,
                ),
            ),
        )
        return views.view(created, user, now, snapshot, pending = polisher.request(created)).copy(newUnlocks = newUnlocks)
    }

    /** With [waitMs] the call holds until the model rewrite lands or the wait runs out. */
    suspend fun today(userId: Long, waitMs: Long = 0): TodayResponse = coroutineScope {
        val user = profiles.user(userId)
        val now = clock.instant()
        val local = LocalDateTime.ofInstant(now, user.timezone)
        val workDay = WorkDay.of(local)
        val active = checkins.active(userId, workDay)
            ?: return@coroutineScope TodayResponse(
                workDay.toString(), null, planner.greeting(user, workDay, local, checkins.lastDayBefore(userId, workDay)),
            )

        val snapshot = stats.snapshot(user, now)
        val tipList = async { tips.tips(user, active, snapshot, now) }
        if (waitMs > 0 && polisher.isPending(active.id)) polisher.await(active.id, Duration.ofMillis(waitMs.coerceAtMost(MAX_WAIT_MS)))
        val row = checkins.find(active.id) ?: active
        val view = views.view(row, user, now, snapshot, pending = polisher.isPending(row.id))
        TodayResponse(workDay.toString(), view, tips = tipList.await())
    }

    suspend fun undo(userId: Long): TodayResponse {
        val user = profiles.user(userId)
        val now = clock.instant()
        val workDay = WorkDay.of(LocalDateTime.ofInstant(now, user.timezone))
        val active = checkins.active(userId, workDay) ?: conflict("今天还没打卡，没什么可撤销的", "nothing_to_undo")
        if (checkins.undoUsed(userId, workDay)) conflict("今天已经撤销过一次了", "undo_used")
        if (now.isAfter(active.checkedAt.plus(CheckinViews.UNDO_WINDOW))) conflict("打卡超过 10 分钟，撤销不了了", "undo_expired")
        checkins.revoke(active.id, now)
        percentiles.record(workDay, WorkDay.minuteOf(LocalDateTime.ofInstant(active.checkedAt, user.timezone)), user.cityCode, -1)
        memories.forget(userId, sourceRef(active))
        return today(userId)
    }

    private suspend fun view(row: CheckinRow, user: User, now: Instant, pending: Boolean = false): CheckinView =
        views.view(row, user, now, stats.snapshot(user, now), pending)

    private suspend fun remember(row: CheckinRow, plan: ReplyPlan, enteredCare: Boolean) {
        val ref = sourceRef(row)
        if (checkins.isFirst(row.userId, row.id)) {
            memories.record(row.userId, MemoryKind.Event, "第一次在下班了么打卡，那天是「${row.title.label}」", SOURCE, ref)
        }
        if (row.title.rare) memories.record(row.userId, MemoryKind.Event, "拿到了稀有称号「${row.title.label}」", SOURCE, ref)
        when {
            enteredCare -> memories.record(row.userId, MemoryKind.Emotion, "连续 3 天很晚才下班，有点累", SOURCE, ref)
            plan.overtimeStreak == 3 -> memories.record(row.userId, MemoryKind.Emotion, "连续 3 天加班", SOURCE, ref)
        }
    }

    private fun sourceRef(row: CheckinRow) = "checkin:${row.id}"

    private companion object {
        const val SOURCE = "checkin"
        const val HISTORY_DAYS = 45L
        const val NO_REPEAT_DAYS = 30L
        const val MAX_WAIT_MS = 3_500L
    }
}
