package com.s00d.subly.widget

import android.content.Context
import android.os.Handler
import android.os.Looper
import androidx.glance.appwidget.updateAll
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.launch

/** Called from Rust (JNI) and MainActivity when widget snapshot data changes. */
object WidgetSnapshotNotifier {
    @Volatile
    private var appContext: Context? = null

    @JvmStatic
    fun init(context: Context) {
        appContext = context.applicationContext
    }

    @JvmStatic
    fun notifySnapshotUpdated() {
        val ctx = appContext ?: return
        Handler(Looper.getMainLooper()).post {
            CoroutineScope(Dispatchers.Main.immediate).launch {
                UpcomingPaymentsWidget().updateAll(ctx)
            }
        }
    }

    @JvmStatic
    fun requestRefresh(context: Context) {
        init(context)
        notifySnapshotUpdated()
    }
}
