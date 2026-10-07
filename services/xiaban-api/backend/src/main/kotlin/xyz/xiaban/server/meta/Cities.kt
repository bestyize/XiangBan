package xyz.xiaban.server.meta

import kotlinx.serialization.Serializable
import xyz.xiaban.contracts.account.profile.CityView
import xyz.xiaban.server.http.AppJson

@Serializable
data class City(
    val code: String,
    val name: String,
    val province: String,
    val lon: Double? = null,
    val lat: Double? = null,
) {
    val view: CityView get() = CityView(code, name, province)
}

class Cities(val all: List<City>) {
    private val byCode = all.associateBy { it.code }

    fun find(code: String?): City? = code?.let(byCode::get)

    companion object {
        fun load(): Cities {
            val text = Cities::class.java.getResourceAsStream("/cities.json")!!.bufferedReader().use { it.readText() }
            return Cities(AppJson.decodeFromString<List<City>>(text))
        }
    }
}
