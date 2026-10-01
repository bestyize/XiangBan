package xyz.xiaban.server.checkin

import xyz.xiaban.server.calendar.HolidayCalendar
import xyz.xiaban.server.http.conflict
import xyz.xiaban.server.users.HHMM
import xyz.xiaban.server.users.ProfileService
import xyz.xiaban.server.users.User
import xyz.xiaban.shared.CheckinView
import xyz.xiaban.shared.ReplyView
import xyz.xiaban.shared.TitleView
import xyz.xiaban.shared.TodayResponse
import java.time.Clock
import java.time.Duration
import java.time.Instant
import java.time.LocalDate
import java.time.LocalDateTime

class CheckinService(
    private val profiles: ProfileService,
    private val checkins: CheckinRepository,
    private val calendar: HolidayCalendar,
    private val clock: Clock,
) {
    suspend fun checkIn(userId: Long): CheckinView {
        val user = profiles.user(userId)
        val now = clock.instant()
        val local = LocalDateTime.ofInstant(now, user.timezone)
        val workDay = WorkDay.of(local)

        // 当天重复点击直接返回已有结果，不重新计算
        checkins.active(userId, workDay)?.let { return view(it, user, now) }

        val title = TitleRules.compute(local, workDay, user.workEnd, calendar)
        val created = checkins.insert(
            NewCheckin(
                userId = userId,
                workDay = workDay,
                checkedAt = now,
                title = title,
                copyId = "fallback:${title.code}",
                libraryReply = TitleRules.fallbackLine(title),
            ),
        ) ?: checkins.active(userId, workDay)!!
        return view(created, user, now)
    }

    suspend fun today(userId: Long): TodayResponse {
        val user = profiles.user(userId)
        val now = clock.instant()
        val workDay = currentWorkDay(user, now)
        return TodayResponse(workDay.toString(), checkins.active(userId, workDay)?.let { view(it, user, now) })
    }

    suspend fun undo(userId: Long): TodayResponse {
        val user = profiles.user(userId)
        val now = clock.instant()
        val workDay = currentWorkDay(user, now)
        val active = checkins.active(userId, workDay) ?: conflict("今天还没打卡，没什么可撤销的", "nothing_to_undo")
        if (checkins.undoUsed(userId, workDay)) conflict("今天已经撤销过一次了", "undo_used")
        if (now.isAfter(active.checkedAt.plus(UNDO_WINDOW))) conflict("打卡超过 10 分钟，撤销不了了", "undo_expired")
        checkins.revoke(active.id, now)
        return TodayResponse(workDay.toString(), null)
    }

    private fun currentWorkDay(user: User, now: Instant): LocalDate = WorkDay.of(LocalDateTime.ofInstant(now, user.timezone))

    private suspend fun view(row: CheckinRow, user: User, now: Instant): CheckinView {
        val undoDeadline = row.checkedAt.plus(UNDO_WINDOW)
        val canUndo = now.isBefore(undoDeadline) && !checkins.undoUsed(row.userId, row.workDay)
        return CheckinView(
            id = row.id.toString(),
            workDay = row.workDay.toString(),
            checkedAt = row.checkedAt.toEpochMilli(),
            localTime = LocalDateTime.ofInstant(row.checkedAt, user.timezone).format(HHMM),
            title = TitleView(row.title, row.title.label, row.title.rare, row.title.mood),
            mood = row.title.mood,
            reply = ReplyView(row.reply, row.replySource, pending = false),
            undoUntil = if (canUndo) undoDeadline.toEpochMilli() else null,
        )
    }

    companion object {
        val UNDO_WINDOW: Duration = Duration.ofMinutes(10)
    }
}
