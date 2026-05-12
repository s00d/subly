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
    }
}

rust {
    rootDirRel = "../../../"
}

dependencies {
    implementation("androidx.webkit:webkit:1.14.0")
    implementation("androidx.appcompat:appcompat:1.7.1")
    implementation("androidx.activity:activity-ktx:1.10.1")
    implementation("androidx.core:core-splashscreen:1.0.1")
    implementation("com.google.android.material:material:1.12.0")
    testImplementation("junit:junit:4.13.2")
    androidTestImplementation("androidx.test.ext:junit:1.1.4")
    androidTestImplementation("androidx.test.espresso:espresso-core:3.5.0")
}

apply(from = "tauri.build.gradle.kts")