package xyz.xiaban.server.events

import kotlinx.serialization.Serializable
import xyz.xiaban.server.db.Db
import xyz.xiaban.server.db.query
import java.sql.Connection
import java.time.Clock
import java.time.LocalDate

/** People counted on one statistics day, split by account type. */
@Serializable
data class MetricPoint(val day: String, val total: Long, val registered: Long, val guest: Long)

@Serializable
data class MetricSeries(val key: String, val label: String, val points: List<MetricPoint>)

@Serializable
data class MetricsResponse(
    val days: List<String>,
    /** How the day is cut, shown on the dashboard */
    val dayNote: String,
    val visits: List<MetricSeries>,
    val growth: List<MetricSeries>,
    val product: List<MetricSeries>,
    val games: List<MetricSeries>,
    val quizzes: List<MetricSeries>,
)

/**
 * Daily figures for the admin dashboard, from the events table. A "day" is Beijing time cut at 05:00
 * ([EventService.metricsDay]); people are counted by actor, so one person doing something five times is one.
 */
class MetricsService(private val db: Db, private val clock: Clock) {

    suspend fun daily(rangeDays: Int, games: Map<String, String>, quizzes: Map<String, String>): MetricsResponse {
        val days = rangeDays.coerceIn(1, 90)
        val to = EventService.metricsDay(clock.instant())
        val from = to.minusDays(days - 1L)
        val dayList = (0 until days).map { from.plusDays(it.toLong()) }

        return db.read { c ->
            val people = distinctPeople(c, from, to)
            val pageViews = c.query(
                """select work_day, count(*), count(*) filter (where user_type = 'registered'), count(*) filter (where user_type = 'guest')
                   from events where name = 'page_view' and work_day between ? and ? group by work_day""",
                from, to,
            ) { it.getObject(1, LocalDate::class.java) to Triple(it.getLong(2), it.getLong(3), it.getLong(4)) }.toMap()
            val registrations = c.query(
                """select (((email_verified_at at time zone 'Asia/Shanghai') - interval '5 hours')::date) as d, count(*)
                   from users where email is not null and email_verified_at is not null
                     and email_verified_at >= ?::date at time zone 'Asia/Shanghai' + interval '5 hours'
                   group by d""",
                from,
            ) { it.getObject(1, LocalDate::class.java) to it.getLong(2) }.toMap()

            fun series(key: String, label: String, pick: (LocalDate) -> Triple<Long, Long, Long>?) =
                MetricSeries(key, label, dayList.map { d ->
                    val (t, r, g) = pick(d) ?: Triple(0L, 0L, 0L)
                    MetricPoint(d.toString(), t, r, g)
                })
            fun peopleSeries(key: String, label: String, bucket: String) = series(key, label) { people[bucket to it] }

            MetricsResponse(
                days = dayList.map(LocalDate::toString),
                dayNote = "按北京时间统计，每天从凌晨 05:00 开始，与打卡的下班日一致",
                visits = listOf(
                    peopleSeries("uv", "访客数（UV）", "visit"),
                    series("pv", "页面访问次数（PV）") { pageViews[it] },
                ),
                growth = listOf(
                    series("signup", "新增注册用户") { d -> registrations[d]?.let { Triple(it, it, 0L) } },
                ),
                product = listOf(
                    peopleSeries("checkin", "打卡人数", "checkin"),
                    peopleSeries("share", "分享人数（保存或分享下班卡）", "share"),
                    peopleSeries("card_save", "其中：保存图片", "card_save"),
                    peopleSeries("card_share", "其中：分享给朋友", "card_share"),
                    peopleSeries("card_generate", "生成下班卡人数", "card_generate"),
                ),
                games = games.map { (id, name) -> peopleSeries("game:$id", name, "game:$id") },
                quizzes = quizzes.map { (id, name) -> peopleSeries("quiz:$id", name, "quiz:$id") },
            )
        }
    }

    /**
     * One scan of the window: for every (bucket, day) the distinct actors, overall and per account type.
     * Undone check-ins still count: the person did check in that day.
     */
    private fun distinctPeople(c: Connection, from: LocalDate, to: LocalDate): Map<Pair<String, LocalDate>, Triple<Long, Long, Long>> =
        c.query(
            """
            with tagged as (
              select work_day, actor, user_type, case
                when name = 'page_view' then 'visit'
                when name = 'checkin' then 'checkin'
                when name = 'game_start' then 'game:' || coalesce(props->>'game', '')
                when name = 'quiz_start' then 'quiz:' || coalesce(props->>'quiz', '')
              end as bucket
              from events where work_day between ? and ? and actor is not null
              union all
              select work_day, actor, user_type, unnest(array['share', name]) from events
              where work_day between ? and ? and actor is not null and name in ('card_save', 'card_share')
              union all
              select work_day, actor, user_type, 'card_generate' from events
              where work_day between ? and ? and actor is not null and name = 'card_generate'
            )
            select bucket, work_day, count(distinct actor),
                   count(distinct actor) filter (where user_type = 'registered'),
                   count(distinct actor) filter (where user_type = 'guest')
            from tagged where bucket is not null group by bucket, work_day
            """,
            from, to, from, to, from, to,
        ) { (it.getString(1) to it.getObject(2, LocalDate::class.java)) to Triple(it.getLong(3), it.getLong(4), it.getLong(5)) }.toMap()
}
