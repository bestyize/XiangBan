package xyz.xiaban.server.db

import java.sql.Connection
import java.sql.PreparedStatement
import java.sql.ResultSet
import java.sql.Types
import java.time.Instant
import java.time.LocalDate
import java.time.LocalTime
import java.time.OffsetDateTime
import java.time.ZoneOffset

fun Connection.prepare(sql: String, params: Array<out Any?>): PreparedStatement {
    val ps = prepareStatement(sql)
    params.forEachIndexed { i, value -> ps.bind(i + 1, value) }
    return ps
}

private fun PreparedStatement.bind(index: Int, value: Any?) {
    when (value) {
        null -> setNull(index, Types.NULL)
        is Instant -> setObject(index, OffsetDateTime.ofInstant(value, ZoneOffset.UTC))
        else -> setObject(index, value)
    }
}

fun <T> Connection.query(sql: String, vararg params: Any?, map: (ResultSet) -> T): List<T> =
    prepare(sql, params).use { ps ->
        ps.executeQuery().use { rs -> buildList { while (rs.next()) add(map(rs)) } }
    }

fun <T> Connection.queryOne(sql: String, vararg params: Any?, map: (ResultSet) -> T): T? =
    query(sql, *params, map = map).firstOrNull()

fun Connection.execute(sql: String, vararg params: Any?): Int =
    prepare(sql, params).use { it.executeUpdate() }

fun ResultSet.instant(column: String): Instant? = getObject(column, OffsetDateTime::class.java)?.toInstant()

fun ResultSet.localDate(column: String): LocalDate = getObject(column, LocalDate::class.java)

fun ResultSet.localTime(column: String): LocalTime = getObject(column, LocalTime::class.java)

fun ResultSet.intOrNull(column: String): Int? = getInt(column).takeUnless { wasNull() }

fun ResultSet.longOrNull(column: String): Long? = getLong(column).takeUnless { wasNull() }
