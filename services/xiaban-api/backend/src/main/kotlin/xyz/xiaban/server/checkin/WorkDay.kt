package xyz.xiaban.server.checkin

import java.time.Duration
import java.time.LocalDate
import java.time.LocalDateTime
import java.time.LocalTime

/** A 下班日 runs from 05:00 to 05:00 the next morning, so leaving at 2 a.m. still counts for the day before. */
object WorkDay {
    val BOUNDARY: LocalTime = LocalTime.of(5, 0)

    fun of(local: LocalDateTime): LocalDate =
        if (local.toLocalTime() < BOUNDARY) local.toLocalDate().minusDays(1) else local.toLocalDate()

    /** Minutes since 05:00 of the 下班日, 0 until 1439. */
    fun minuteOf(local: LocalDateTime): Int = Duration.between(of(local).atTime(BOUNDARY), local).toMinutes().toInt()

    /** The configured off-work moment on the 下班日 timeline (an end time before 05:00 falls on the next morning). */
    fun endOfWork(workDay: LocalDate, workEnd: LocalTime): LocalDateTime =
        if (workEnd < BOUNDARY) workDay.plusDays(1).atTime(workEnd) else workDay.atTime(workEnd)

    fun startOfWork(workDay: LocalDate, workStart: LocalTime): LocalDateTime =
        if (workStart < BOUNDARY) workDay.plusDays(1).atTime(workStart) else workDay.atTime(workStart)
}
