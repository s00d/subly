package com.s00d.subly.widget

import android.content.Context
import org.json.JSONArray
import org.json.JSONObject
import java.io.File
import java.math.BigDecimal
import java.text.DateFormat
import java.text.SimpleDateFormat
import java.util.Calendar
import java.util.Currency
import java.util.Locale
import java.util.TimeZone
import java.util.concurrent.TimeUnit

data class WidgetSnapshotItem(
    val id: String,
    val name: String,
    val nextPayment: String,
    val price: BigDecimal,
    val currencyId: String,
    val currencyCode: String,
    val iconFile: String?,
)

data class LoadedSnapshot(
    val items: List<WidgetSnapshotItem>,
    val errorKey: String?,
)

object WidgetSnapshotLoader {
    private val isoDay: SimpleDateFormat =
        SimpleDateFormat("yyyy-MM-dd", Locale.US).apply {
            timeZone = TimeZone.getTimeZone("UTC")
            isLenient = false
        }

    private fun widgetDir(context: Context): File = File(context.filesDir, "widget")

    fun iconPath(context: Context, iconFile: String?): File? {
        val rel = iconFile?.trim().orEmpty()
        if (rel.isEmpty() || rel.contains("..")) return null
        val file = File(widgetDir(context), rel)
        return if (file.isFile) file else null
    }

    fun load(context: Context): LoadedSnapshot =
        load(File(widgetDir(context), "upcoming.json"))

    /** Same JSON contract as Rust `widget_snapshot.rs` / iOS `WidgetSnapshotIO.swift`. */
    fun load(jsonFile: File): LoadedSnapshot {
        if (!jsonFile.isFile) {
            return LoadedSnapshot(emptyList(), "widget_err_open_app")
        }
        return try {
            val root = JSONObject(jsonFile.readText())
            decodeRoot(root)
        } catch (_: Exception) {
            LoadedSnapshot(emptyList(), "widget_err_bad_data")
        }
    }

    internal fun decodeRoot(root: JSONObject): LoadedSnapshot {
        val arr = root.optJSONArray("items") ?: JSONArray()
        val items = buildList {
            for (i in 0 until arr.length()) {
                val o = arr.optJSONObject(i) ?: continue
                add(parseItem(o))
            }
        }
        return LoadedSnapshot(items, null)
    }

    private fun parseItem(o: JSONObject): WidgetSnapshotItem {
        val price = when (val raw = o.opt("price")) {
            is String -> raw.toBigDecimalOrNull() ?: BigDecimal.ZERO
            is Number -> BigDecimal(raw.toString())
            else -> BigDecimal.ZERO
        }
        val code = o.optString("currencyCode", "").trim().uppercase(Locale.US)
        return WidgetSnapshotItem(
            id = o.optString("id", ""),
            name = o.optString("name", ""),
            nextPayment = o.optString("nextPayment", ""),
            price = price,
            currencyId = o.optString("currencyId", ""),
            currencyCode = if (code.length == 3) code else "USD",
            iconFile = o.optString("iconFile").takeIf { it.isNotBlank() },
        )
    }

    fun clippedTitle(raw: String, maxLen: Int = 200): String {
        val s = raw.replace('\n', ' ').replace('\t', ' ').trim()
        if (s.length <= maxLen) return s
        return s.take(maxLen) + "…"
    }

    fun avatarGlyph(name: String): String {
        val t = clippedTitle(name, 80)
        val ch = t.firstOrNull() ?: return "?"
        return ch.uppercaseChar().toString()
    }

    fun paymentDateLabel(context: Context, iso: String): String {
        val trimmed = iso.trim()
        val pay = parsePaymentDay(trimmed) ?: return formatDayShort(trimmed, null)
        val today = startOfDay(Calendar.getInstance())
        val payDay = startOfDay(pay)
        val days = TimeUnit.MILLISECONDS.toDays(payDay.timeInMillis - today.timeInMillis)
        return when (days) {
            0L -> context.getString(R.string.widget_date_today)
            1L -> context.getString(R.string.widget_date_tomorrow)
            else -> formatDayShort(trimmed, pay)
        }
    }

    private fun parsePaymentDay(iso: String): Calendar? {
        return try {
            val date = isoDay.parse(iso) ?: return null
            Calendar.getInstance().apply { time = date }
        } catch (_: Exception) {
            null
        }
    }

    private fun startOfDay(cal: Calendar): Calendar {
        return (cal.clone() as Calendar).apply {
            set(Calendar.HOUR_OF_DAY, 0)
            set(Calendar.MINUTE, 0)
            set(Calendar.SECOND, 0)
            set(Calendar.MILLISECOND, 0)
        }
    }

    private fun formatDayShort(fallback: String, date: Calendar?): String {
        if (date == null) {
            val parsed = parsePaymentDay(fallback.trim())
            if (parsed == null) {
                return if (fallback.length > 28) fallback.take(28) + "…" else fallback
            }
            return DateFormat.getDateInstance(DateFormat.SHORT, Locale.getDefault()).format(parsed.time)
        }
        return DateFormat.getDateInstance(DateFormat.SHORT, Locale.getDefault()).format(date.time)
    }

    fun formatCurrency(price: BigDecimal, isoCode: String): String {
        return try {
            val currency = Currency.getInstance(isoCode)
            val symbols = java.text.NumberFormat.getCurrencyInstance(Locale.getDefault()) as java.text.DecimalFormat
            symbols.currency = currency
            val frac = price.remainder(BigDecimal.ONE).abs()
            symbols.minimumFractionDigits = if (frac < BigDecimal("0.001")) 0 else 2
            symbols.maximumFractionDigits = 2
            symbols.format(price)
        } catch (_: Exception) {
            price.setScale(2).toPlainString()
        }
    }

    fun errorMessage(context: Context, key: String): String {
        val resId = when (key) {
            "widget_err_no_group" -> R.string.widget_err_no_group
            "widget_err_bad_data" -> R.string.widget_err_bad_data
            else -> R.string.widget_err_open_app
        }
        return context.getString(resId)
    }
}
