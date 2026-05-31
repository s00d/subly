package com.s00d.subly.widget

import androidx.compose.ui.graphics.Color
import androidx.glance.color.ColorProvider

object SublyWidgetTheme {
    val accent = Color(0xFF3878FA)
    val accentDeep = Color(0xFF1F59E0)
    val accentProvider = ColorProvider(day = accent, night = accent)
    val accentDeepProvider = ColorProvider(day = accentDeep, night = accentDeep)

    val backgroundProvider = ColorProvider(
        day = Color(0xFFF2F2F7),
        night = Color(0xFF1C1C1E),
    )

    val primaryTextProvider = ColorProvider(
        day = Color(0xFF000000),
        night = Color(0xFFFFFFFF),
    )

    val secondaryTextProvider = ColorProvider(
        day = Color(0x99000000),
        night = Color(0x99FFFFFF),
    )

    val tileFillProvider = ColorProvider(
        day = Color(0x0A000000),
        night = Color(0x0FFFFFFF),
    )
}
