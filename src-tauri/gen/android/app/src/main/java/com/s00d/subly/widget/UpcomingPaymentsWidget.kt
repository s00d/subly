package com.s00d.subly.widget

import android.appwidget.AppWidgetManager
import android.content.ComponentName
import android.content.Context
import android.graphics.BitmapFactory
import androidx.compose.runtime.Composable
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.unit.Dp
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import androidx.glance.GlanceId
import androidx.glance.GlanceModifier
import androidx.glance.Image
import androidx.glance.ImageProvider
import androidx.glance.LocalContext
import androidx.glance.action.actionStartActivity
import androidx.glance.action.clickable
import androidx.glance.appwidget.GlanceAppWidget
import androidx.glance.appwidget.GlanceAppWidgetReceiver
import androidx.glance.appwidget.provideContent
import androidx.glance.background
import androidx.glance.color.ColorProvider
import androidx.glance.layout.Alignment
import androidx.glance.layout.Box
import androidx.glance.layout.Column
import androidx.glance.layout.Row
import androidx.glance.layout.Spacer
import androidx.glance.layout.fillMaxSize
import androidx.glance.layout.fillMaxWidth
import androidx.glance.layout.height
import androidx.glance.layout.padding
import androidx.glance.layout.size
import androidx.glance.layout.width
import androidx.glance.text.FontWeight
import androidx.glance.text.Text
import androidx.glance.text.TextStyle
import com.s00d.subly.MainActivity
import com.s00d.subly.R

class SublyWidgetReceiver : GlanceAppWidgetReceiver() {
    override val glanceAppWidget: GlanceAppWidget = UpcomingPaymentsWidget()
}

class UpcomingPaymentsWidget : GlanceAppWidget() {
    override suspend fun provideGlance(context: Context, id: GlanceId) {
        val minWidthDp = AppWidgetManager.getInstance(context)
            .getAppWidgetOptions(id)
            .getInt(AppWidgetManager.OPTION_APPWIDGET_MIN_WIDTH, 250)
        provideContent {
            WidgetChrome(minWidthDp = minWidthDp) {
                WidgetBody(compact = minWidthDp < 180)
            }
        }
    }
}

private fun paddingForMinWidth(minWidthDp: Int): Dp = when {
    minWidthDp < 180 -> 14.dp
    minWidthDp < 280 -> 10.dp
    else -> 12.dp
}

@Composable
private fun WidgetChrome(minWidthDp: Int, content: @Composable () -> Unit) {
    Box(
        modifier = GlanceModifier
            .fillMaxSize()
            .background(SublyWidgetTheme.backgroundProvider)
            .padding(paddingForMinWidth(minWidthDp)),
        contentAlignment = Alignment.TopStart,
    ) {
        content()
    }
}

@Composable
private fun WidgetBody(compact: Boolean) {
    val context = LocalContext.current
    val snapshot = WidgetSnapshotLoader.load(context)
    val open = actionStartActivity(ComponentName(context, MainActivity::class.java))
    val tap = GlanceModifier.clickable(open)

    when {
        snapshot.errorKey != null -> {
            Box(modifier = tap.fillMaxSize(), contentAlignment = Alignment.Center) {
                ErrorState(snapshot.errorKey!!)
            }
        }
        snapshot.items.isEmpty() -> {
            Box(modifier = tap.fillMaxSize(), contentAlignment = Alignment.Center) {
                EmptyState()
            }
        }
        compact -> {
            SmallBody(snapshot.items.first())
        }
        else -> {
            LargeBody(snapshot.items)
        }
    }
}

@Composable
private fun ErrorState(errorKey: String) {
    val context = LocalContext.current
    Text(
        text = WidgetSnapshotLoader.errorMessage(context, errorKey),
        style = TextStyle(
            color = SublyWidgetTheme.secondaryTextProvider,
            fontSize = 11.sp,
            fontWeight = FontWeight.Medium,
        ),
        maxLines = 4,
    )
}

@Composable
private fun EmptyState() {
    val context = LocalContext.current
    Column(horizontalAlignment = Alignment.CenterHorizontally) {
        Box(
            modifier = GlanceModifier
                .size(52.dp)
                .background(ColorProvider(day = Color(0x263878FA), night = Color(0x263878FA))),
            contentAlignment = Alignment.Center,
        ) {
            Text(
                text = "✓",
                style = TextStyle(
                    color = SublyWidgetTheme.accentProvider,
                    fontSize = 28.sp,
                    fontWeight = FontWeight.Bold,
                ),
            )
        }
        Spacer(GlanceModifier.height(8.dp))
        Text(
            text = context.getString(R.string.widget_no_upcoming),
            style = TextStyle(
                color = SublyWidgetTheme.secondaryTextProvider,
                fontSize = 12.sp,
                fontWeight = FontWeight.Medium,
            ),
            maxLines = 3,
        )
    }
}

@Composable
private fun SmallBody(item: WidgetSnapshotItem) {
    val context = LocalContext.current
    Column(modifier = GlanceModifier.fillMaxSize()) {
        Box(
            modifier = GlanceModifier
                .fillMaxWidth()
                .height(4.dp)
                .background(SublyWidgetTheme.accentProvider),
        ) {}
        Spacer(GlanceModifier.height(10.dp))
        Row(verticalAlignment = Alignment.CenterVertically) {
            SubscriptionAvatar(item, 44.dp)
            Spacer(GlanceModifier.width(10.dp))
            Column(modifier = GlanceModifier.fillMaxWidth()) {
                Text(
                    text = WidgetSnapshotLoader.clippedTitle(item.name),
                    style = TextStyle(
                        color = SublyWidgetTheme.primaryTextProvider,
                        fontSize = 15.sp,
                        fontWeight = FontWeight.Bold,
                    ),
                    maxLines = 3,
                )
                Spacer(GlanceModifier.height(4.dp))
                Text(
                    text = WidgetSnapshotLoader.paymentDateLabel(context, item.nextPayment),
                    style = TextStyle(
                        color = SublyWidgetTheme.secondaryTextProvider,
                        fontSize = 11.sp,
                        fontWeight = FontWeight.Medium,
                    ),
                    maxLines = 1,
                )
            }
        }
        Spacer(GlanceModifier.height(8.dp))
        Text(
            text = WidgetSnapshotLoader.formatCurrency(item.price, item.currencyCode),
            style = TextStyle(
                color = SublyWidgetTheme.accentProvider,
                fontSize = 22.sp,
                fontWeight = FontWeight.Bold,
            ),
            maxLines = 1,
        )
    }
}

@Composable
private fun LargeBody(items: List<WidgetSnapshotItem>) {
    val context = LocalContext.current
    val (left, right, rest) = splitForGrid(items)

    Column(modifier = GlanceModifier.fillMaxSize()) {
        Text(
            text = context.getString(R.string.widget_upcoming_title),
            style = TextStyle(
                color = SublyWidgetTheme.primaryTextProvider,
                fontSize = 11.sp,
                fontWeight = FontWeight.Bold,
            ),
            maxLines = 1,
        )
        Spacer(GlanceModifier.height(4.dp))
        left.forEach { CompactRow(it, dense = true) }
        right.forEach { CompactRow(it, dense = true) }
        if (rest > 0) {
            Spacer(GlanceModifier.height(4.dp))
            Text(
                text = context.getString(R.string.widget_more_count, rest),
                style = TextStyle(
                    color = SublyWidgetTheme.secondaryTextProvider,
                    fontSize = 9.sp,
                    fontWeight = FontWeight.Bold,
                ),
                maxLines = 1,
            )
        }
    }
}

private fun splitForGrid(items: List<WidgetSnapshotItem>): Triple<List<WidgetSnapshotItem>, List<WidgetSnapshotItem>, Int> {
    val cap = minOf(12, items.size)
    val visible = items.take(cap)
    val left = visible.filterIndexed { i, _ -> i % 2 == 0 }
    val right = visible.filterIndexed { i, _ -> i % 2 == 1 }
    val rest = items.size - visible.size
    return Triple(left, right, rest)
}

@Composable
private fun CompactRow(item: WidgetSnapshotItem, dense: Boolean) {
    val context = LocalContext.current
    val avatar = if (dense) 20.dp else 24.dp
    Row(
        modifier = GlanceModifier
            .fillMaxWidth()
            .padding(vertical = if (dense) 3.dp else 4.dp),
        verticalAlignment = Alignment.CenterVertically,
    ) {
        SubscriptionAvatar(item, avatar)
        Spacer(GlanceModifier.width(if (dense) 5.dp else 6.dp))
        Column {
            Text(
                text = WidgetSnapshotLoader.clippedTitle(item.name),
                style = TextStyle(
                    color = SublyWidgetTheme.primaryTextProvider,
                    fontSize = if (dense) 10.sp else 11.sp,
                    fontWeight = FontWeight.Medium,
                ),
                maxLines = 1,
            )
            Text(
                text = WidgetSnapshotLoader.paymentDateLabel(context, item.nextPayment),
                style = TextStyle(
                    color = SublyWidgetTheme.secondaryTextProvider,
                    fontSize = if (dense) 8.sp else 9.sp,
                ),
                maxLines = 1,
            )
        }
        Spacer(GlanceModifier.width(4.dp))
        Text(
            text = WidgetSnapshotLoader.formatCurrency(item.price, item.currencyCode),
            style = TextStyle(
                color = SublyWidgetTheme.accentProvider,
                fontSize = if (dense) 10.sp else 11.sp,
                fontWeight = FontWeight.Bold,
            ),
            maxLines = 1,
        )
    }
}

@Composable
private fun SubscriptionAvatar(item: WidgetSnapshotItem, edge: androidx.compose.ui.unit.Dp) {
    val context = LocalContext.current
    val path = WidgetSnapshotLoader.iconPath(context, item.iconFile)
    val bitmap = path?.let { BitmapFactory.decodeFile(it.absolutePath) }
    if (bitmap != null) {
        Image(
            provider = ImageProvider(bitmap),
            contentDescription = null,
            modifier = GlanceModifier.size(edge),
        )
    } else {
        Box(
            modifier = GlanceModifier
                .size(edge)
                .background(SublyWidgetTheme.accentDeepProvider),
            contentAlignment = Alignment.Center,
        ) {
            Text(
                text = WidgetSnapshotLoader.avatarGlyph(item.name),
                style = TextStyle(
                    color = ColorProvider(day = Color.White, night = Color.White),
                    fontSize = (edge.value * 0.38f).sp,
                    fontWeight = FontWeight.Bold,
                ),
            )
        }
    }
}
