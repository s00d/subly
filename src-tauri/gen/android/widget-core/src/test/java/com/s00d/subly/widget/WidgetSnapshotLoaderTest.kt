package com.s00d.subly.widget

import android.content.Context
import androidx.test.core.app.ApplicationProvider
import org.json.JSONObject
import org.junit.Assert.assertEquals
import org.junit.Assert.assertNotNull
import org.junit.Assert.assertNull
import org.junit.Assert.assertTrue
import org.junit.Test
import org.junit.runner.RunWith
import org.robolectric.RobolectricTestRunner
import org.robolectric.annotation.Config
import java.io.File
import java.math.BigDecimal

@RunWith(RobolectricTestRunner::class)
@Config(sdk = [28])
class WidgetSnapshotLoaderTest {
    private fun fixture(name: String): File {
        val url = checkNotNull(javaClass.classLoader?.getResource("widget_fixtures/$name")) {
            "missing fixture widget_fixtures/$name"
        }
        return File(url.toURI())
    }

    @Test
    fun decodeStringPriceMatchesRustContract() {
        val result = WidgetSnapshotLoader.load(fixture("upcoming_string_price.json"))
        assertNull(result.errorKey)
        assertEquals(1, result.items.size)
        val item = result.items.first()
        assertEquals("Streaming", item.name)
        assertEquals(0, item.price.compareTo(BigDecimal("12.50")))
        assertEquals("USD", item.currencyCode)
        assertEquals("icons/sub-widget-fixture-1.png", item.iconFile)
    }

    @Test
    fun decodeLegacyNumberPrice() {
        val result = WidgetSnapshotLoader.load(fixture("upcoming_legacy_number_price.json"))
        assertNull(result.errorKey)
        assertEquals(1, result.items.size)
        assertEquals(0, result.items.first().price.compareTo(BigDecimal("9.99")))
        assertEquals("EUR", result.items.first().currencyCode)
    }

    @Test
    fun missingFileReturnsOpenApp() {
        val missing = File(System.getProperty("java.io.tmpdir"), "subly-missing-${System.nanoTime()}.json")
        val result = WidgetSnapshotLoader.load(missing)
        assertEquals("widget_err_open_app", result.errorKey)
        assertTrue(result.items.isEmpty())
    }

    @Test
    fun invalidJsonReturnsBadData() {
        val bad = File.createTempFile("subly-widget-bad", ".json")
        bad.writeText("{ not valid json }")
        bad.deleteOnExit()
        val result = WidgetSnapshotLoader.load(bad)
        assertEquals("widget_err_bad_data", result.errorKey)
    }

    @Test
    fun stringPriceIsNotReadableAsJsonDoubleOnly() {
        val root = JSONObject(fixture("upcoming_string_price.json").readText())
        val price = root.getJSONArray("items").getJSONObject(0).get("price")
        assertTrue(price is String)
        assertEquals("12.50", price)
    }

    @Test
    fun iconPathRejectsTraversal() {
        val context = ApplicationProvider.getApplicationContext<Context>()
        val widgetDir = File(context.filesDir, "widget").apply { mkdirs() }
        val safe = File(widgetDir, "icons/safe.png").apply {
            parentFile?.mkdirs()
            writeText("x")
        }
        assertNotNull(WidgetSnapshotLoader.iconPath(context, "icons/safe.png"))
        assertEquals(safe.canonicalFile, WidgetSnapshotLoader.iconPath(context, "icons/safe.png")?.canonicalFile)
        assertNull(WidgetSnapshotLoader.iconPath(context, "../secrets.png"))
        assertNull(WidgetSnapshotLoader.iconPath(context, null))
    }

    @Test
    fun clippedTitleAndAvatarGlyph() {
        assertEquals("Hi", WidgetSnapshotLoader.clippedTitle("  Hi \n"))
        val long = "x".repeat(250)
        assertTrue(WidgetSnapshotLoader.clippedTitle(long).endsWith("…"))
        assertEquals("S", WidgetSnapshotLoader.avatarGlyph("streaming"))
        assertEquals("?", WidgetSnapshotLoader.avatarGlyph("   "))
    }

    @Test
    fun errorMessageMapsKeys() {
        val context = ApplicationProvider.getApplicationContext<Context>()
        assertEquals(
            context.getString(R.string.widget_err_open_app),
            WidgetSnapshotLoader.errorMessage(context, "widget_err_open_app"),
        )
        assertEquals(
            context.getString(R.string.widget_err_bad_data),
            WidgetSnapshotLoader.errorMessage(context, "widget_err_bad_data"),
        )
    }

    @Test
    fun paymentDateLabelUsesStringsForFarFuture() {
        val context = ApplicationProvider.getApplicationContext<Context>()
        val label = WidgetSnapshotLoader.paymentDateLabel(context, "2099-06-01")
        assertTrue(label.isNotBlank())
        assertTrue(label != context.getString(R.string.widget_date_today))
        assertTrue(label != context.getString(R.string.widget_date_tomorrow))
    }
}
