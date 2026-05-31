package com.s00d.subly

import android.os.Bundle
import androidx.activity.enableEdgeToEdge
import androidx.core.splashscreen.SplashScreen.Companion.installSplashScreen
import com.s00d.subly.widget.WidgetSnapshotNotifier

class MainActivity : TauriActivity() {
  companion object {
    init {
      System.loadLibrary("subly_lib")
    }
  }

  /** Stores JVM for Rust widget snapshot reload (see widget_snapshot.rs). */
  private external fun nativeInitWidgetBridge(): Int

  override fun onCreate(savedInstanceState: Bundle?) {
    installSplashScreen()
    enableEdgeToEdge()
    super.onCreate(savedInstanceState)
    WidgetSnapshotNotifier.init(this)
    nativeInitWidgetBridge()
  }

  override fun onResume() {
    super.onResume()
    WidgetSnapshotNotifier.requestRefresh(this)
  }
}
