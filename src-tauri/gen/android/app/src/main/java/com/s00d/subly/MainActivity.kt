package com.s00d.subly

import android.os.Bundle
import androidx.activity.enableEdgeToEdge
import androidx.core.view.ViewCompat
import androidx.core.view.WindowInsetsCompat
import androidx.core.splashscreen.SplashScreen.Companion.installSplashScreen
import com.s00d.subly.widget.WidgetSnapshotNotifier

class MainActivity : TauriActivity() {
  companion object {
    init {
      System.loadLibrary("subly_lib")
    }
  }

  /** Stores JVM + app Context for Rust (widget reload, keyring); see widget_snapshot.rs. */
  private external fun nativeInitWidgetBridge(): Int

  override fun onCreate(savedInstanceState: Bundle?) {
    installSplashScreen()
    enableEdgeToEdge()
    // Before super.onCreate: Tauri setup may touch the keyring, which needs ndk-context.
    nativeInitWidgetBridge()
    super.onCreate(savedInstanceState)
    WidgetSnapshotNotifier.init(this)
    // Older System WebViews report env(safe-area-inset-*) as 0 under edge-to-edge,
    // so the header slid under the status bar. Keep content inside the system bars.
    ViewCompat.setOnApplyWindowInsetsListener(findViewById(android.R.id.content)) { view, insets ->
      val bars = insets.getInsets(WindowInsetsCompat.Type.systemBars() or WindowInsetsCompat.Type.displayCutout())
      val ime = insets.getInsets(WindowInsetsCompat.Type.ime())
      view.setPadding(bars.left, bars.top, bars.right, maxOf(bars.bottom, ime.bottom))
      WindowInsetsCompat.CONSUMED
    }
  }

  override fun onResume() {
    super.onResume()
    WidgetSnapshotNotifier.requestRefresh(this)
  }
}
