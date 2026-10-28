import java.util.Properties

plugins {
    id("com.android.application")
    id("org.jetbrains.kotlin.android")
    id("rust")
}

val tauriProperties = Properties().apply {
    val propFile = file("tauri.properties")
    if (propFile.exists()) {
        propFile.inputStream().use { load(it) }
    }
}

// Release signing: prefer `keystore.properties` (local dev) but transparently
// fall back to env vars so CI can supply secrets without writing a file.
// When none of these are present the release variant stays unsigned — same
// behaviour as before this block was introduced.
val keystoreProperties = Properties().apply {
    val propFile = rootProject.file("keystore.properties")
    if (propFile.exists()) {
        propFile.inputStream().use { load(it) }
    }
}
fun keystoreField(key: String, env: String): String? {
    val v = keystoreProperties.getProperty(key) ?: System.getenv(env)
    return v?.takeIf { it.isNotBlank() }
}
val releaseStoreFile = keystoreField("storeFile", "ANDROID_KEYSTORE_PATH")
val releaseStorePassword = keystoreField("storePassword", "ANDROID_KEYSTORE_PASSWORD")
val releaseKeyAlias = keystoreField("keyAlias", "ANDROID_KEY_ALIAS")
val releaseKeyPassword = keystoreField("keyPassword", "ANDROID_KEY_PASSWORD")
val canSignRelease = releaseStoreFile != null
        && releaseStorePassword != null
        && releaseKeyAlias != null
        && releaseKeyPassword != null

android {
    compileSdk = 36
    namespace = "com.s00d.subly"
    defaultConfig {
        manifestPlaceholders["usesCleartextTraffic"] = "false"
        applicationId = "com.s00d.subly"
        minSdk = 24
        targetSdk = 36
        versionCode = tauriProperties.getProperty("tauri.android.versionCode", "1").toInt()
        versionName = tauriProperties.getProperty("tauri.android.versionName", "1.0")
    }
    if (canSignRelease) {
        signingConfigs {
            create("release") {
                storeFile = file(releaseStoreFile!!)
                storePassword = releaseStorePassword
                keyAlias = releaseKeyAlias
                keyPassword = releaseKeyPassword
            }
        }
    }
    buildTypes {
        getByName("debug") {
            manifestPlaceholders["usesCleartextTraffic"] = "true"
            isDebuggable = true
            isJniDebuggable = true
            isMinifyEnabled = false
            packaging {                jniLibs.keepDebugSymbols.add("*/arm64-v8a/*.so")
                jniLibs.keepDebugSymbols.add("*/armeabi-v7a/*.so")
                jniLibs.keepDebugSymbols.add("*/x86/*.so")
                jniLibs.keepDebugSymbols.add("*/x86_64/*.so")
            }
        }
        getByName("release") {
            isMinifyEnabled = true
            // Drops unused Material layouts, so R8 no longer keeps MaterialDatePicker
            // (its Window.setStatusBarColor call is flagged by Play for Android 15).
            isShrinkResources = true
            proguardFiles(
                *fileTree(".") { include("**/*.pro") }
                    .plus(getDefaultProguardFile("proguard-android-optimize.txt"))
                    .toList().toTypedArray()
            )
            if (canSignRelease) {
                signingConfig = signingConfigs.getByName("release")
            }
            // Ship a `BUNDLE-METADATA/com.android.tools.build.debugsymbols/`
            // entry inside the AAB so Play Console can deobfuscate Rust
            // native crash traces. SYMBOL_TABLE = lightweight (function
            // names only); use "FULL" if you also want source lines —
            // significantly larger AAB.
            ndk {
                debugSymbolLevel = "SYMBOL_TABLE"
            }
        }
    }
    kotlinOptions {
        jvmTarget = "1.8"
    }
    buildFeatures {
        buildConfig = true
        compose = true
    }
    composeOptions {
        kotlinCompilerExtensionVersion = "1.5.15"
    }
}

rust {
    rootDirRel = "../../../"
}

dependencies {
    implementation(project(":widget-core"))
    val composeBom = platform("androidx.compose:compose-bom:2024.10.00")
    implementation(composeBom)
    implementation("androidx.compose.runtime:runtime")
    implementation("androidx.compose.ui:ui")
    implementation("androidx.glance:glance-appwidget:1.1.1")
    implementation("androidx.glance:glance-material3:1.1.1")
    implementation("org.jetbrains.kotlinx:kotlinx-coroutines-android:1.8.1")
    implementation("androidx.webkit:webkit:1.14.0")
    implementation("androidx.appcompat:appcompat:1.7.1")
    implementation("androidx.activity:activity-ktx:1.12.4")
    implementation("androidx.core:core-splashscreen:1.2.0")
    implementation("com.google.android.material:material:1.13.0")
    androidTestImplementation("androidx.test.ext:junit:1.1.4")
    androidTestImplementation("androidx.test.espresso:espresso-core:3.5.0")
}

apply(from = "tauri.build.gradle.kts")