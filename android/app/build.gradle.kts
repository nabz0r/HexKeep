plugins { id("com.android.application"); id("org.jetbrains.kotlin.android") }
android {
    namespace = "game.hexkeep"
    compileSdk = 36
    defaultConfig {
        applicationId = "game.hexkeep"
        minSdk = 26
        targetSdk = 36
        versionCode = 4
        versionName = "0.4.0-les-chemins"
        ndk { abiFilters += listOf("arm64-v8a", "armeabi-v7a", "x86_64") }
        testInstrumentationRunner = "androidx.test.runner.AndroidJUnitRunner"
    }
    flavorDimensions += "network"
    productFlavors {
        create("dev") { signingConfig = signingConfigs.getByName("debug"); dimension = "network"; applicationIdSuffix = ".dev"; buildConfigField("boolean", "DEV_NETWORK", "true"); resValue("string", "app_name", "HEXKEEP · Chemins") }
        create("prod") { dimension = "network"; buildConfigField("boolean", "DEV_NETWORK", "false"); resValue("string", "app_name", "HEXKEEP") }
    }
    buildTypes { getByName("release") { isMinifyEnabled = false } }
    buildFeatures { buildConfig = true }
    compileOptions { sourceCompatibility = JavaVersion.VERSION_17; targetCompatibility = JavaVersion.VERSION_17 }
    kotlinOptions { jvmTarget = "17" }
    packaging { jniLibs { useLegacyPackaging = true } }
}
dependencies { implementation("com.android.billingclient:billing:8.0.0"); implementation("net.java.dev.jna:jna:5.17.0@aar"); androidTestImplementation("androidx.test:runner:1.6.2"); androidTestImplementation("androidx.test.ext:junit:1.2.1"); androidTestImplementation("androidx.test:core:1.6.1") }
