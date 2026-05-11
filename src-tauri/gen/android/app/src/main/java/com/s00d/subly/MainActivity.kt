package com.s00d.subly

import android.os.Bundle
import androidx.activity.enableEdgeToEdge
import androidx.core.splashscreen.SplashScreen.Companion.installSplashScreen

class MainActivity : TauriActivity() {
  override fun onCreate(savedInstanceState: Bundle?) {
    // Install the Android 12+ splash screen API. Without this call the system
    // would immediately dismiss the splash theme as soon as the Activity is ready,
    // but installSplashScreen() lets WebView take over with a clean fade-out.
    installSplashScreen()
    enableEdgeToEdge()
    super.onCreate(savedInstanceState)
  }
}
