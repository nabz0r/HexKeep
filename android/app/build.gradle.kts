plugins { id("com.android.application"); id("org.jetbrains.kotlin.android") }
android {
    namespace = "game.hexkeep"
    compileSdk = 36
    defaultConfig {
        applicationId = "game.hexkeep"
        minSdk = 26
        targetSdk = 36
        versionCode = 8
        versionName = "0.8.0-lanternes"
        ndk { abiFilters += listOf("arm64-v8a", "armeabi-v7a", "x86_64") }
        testInstrumentationRunner = "androidx.test.runner.AndroidJUnitRunner"
    }
    // A release is never silently signed with the debug certificate.
    val uploadNames = listOf("HK_UPLOAD_STORE", "HK_UPLOAD_STORE_PASSWORD", "HK_UPLOAD_ALIAS", "HK_UPLOAD_KEY_PASSWORD")
    val upload = uploadNames.map { System.getenv(it) }
    require(upload.all { it.isNullOrBlank() } || upload.all { !it.isNullOrBlank() }) { "Configure all four HK_UPLOAD_* signing variables, or none for an unsigned candidate." }
    if (upload.all { !it.isNullOrBlank() }) signingConfigs.create("upload") {
        storeFile = file(upload[0]!!)
        storePassword = upload[1]
        keyAlias = upload[2]
        keyPassword = upload[3]
    }
    defaultConfig { buildConfigField("boolean", "OFFLINE_EDITION", "false") }
    flavorDimensions += "network"
    productFlavors {
        create("dev") { signingConfig = signingConfigs.getByName("debug"); dimension = "network"; applicationIdSuffix = ".dev"; buildConfigField("boolean", "DEV_NETWORK", "true"); resValue("string", "app_name", "HEXKEEP · DEV") }
        create("prod") { dimension = "network"; buildConfigField("boolean", "DEV_NETWORK", "false"); resValue("string", "app_name", "HEXKEEP") }
        create("play") {
            dimension = "network"
            buildConfigField("boolean", "DEV_NETWORK", "false")
            buildConfigField("boolean", "OFFLINE_EDITION", "true")
            resValue("string", "app_name", "HEXKEEP")
            if (upload.all { !it.isNullOrBlank() }) signingConfig = signingConfigs.getByName("upload")
        }
    }
    sourceSets.getByName("main").assets.setSrcDirs(listOf(layout.buildDirectory.dir("generated/gameAssets")))
    sourceSets.getByName("main").assets.srcDir(layout.buildDirectory.dir("generated/licensesAssets"))
    sourceSets.getByName("dev").java.srcDir("src/network/java")
    sourceSets.getByName("prod").java.srcDir("src/network/java")
    buildTypes { getByName("release") { isMinifyEnabled = false } }
    buildFeatures { buildConfig = true }
    compileOptions { sourceCompatibility = JavaVersion.VERSION_17; targetCompatibility = JavaVersion.VERSION_17 }
    kotlinOptions { jvmTarget = "17" }
    packaging { jniLibs { useLegacyPackaging = true } }
}
dependencies { implementation("androidx.annotation:annotation:1.9.1"); "devImplementation"("com.android.billingclient:billing:8.0.0"); "prodImplementation"("com.android.billingclient:billing:8.0.0"); implementation("net.java.dev.jna:jna:5.17.0@aar"); androidTestImplementation("androidx.test:runner:1.6.2"); androidTestImplementation("androidx.test.ext:junit:1.2.1"); androidTestImplementation("androidx.test:core:1.6.1") }

// Ship the exact repository notices without keeping a second copy in source control.
val bundleLicenses by tasks.registering(Copy::class) {
    from(rootProject.file("../LICENSES"))
    into(layout.buildDirectory.dir("generated/licensesAssets/licenses"))
    eachFile { path = name }
    includeEmptyDirs = false
}
tasks.named("preBuild") { dependsOn(bundleLicenses) }

// Keep the historical source paintings in Git without shipping superseded sprite sheets.
val stageGameAssets by tasks.registering(Sync::class) {
    from("src/main/assets") {
        exclude("art/v05/aurelon.png", "art/v05/skarn.png", "art/v05/vylde.png",
            "art/v05/wolf.png", "art/v05/wraith.png", "art/v05/golem.png", "art/v05/pilleur.png",
            "art/characters.png", "art/keep.png", "art/v06/refuge.png", "art/courtyard.png")
    }
    into(layout.buildDirectory.dir("generated/gameAssets"))
}
tasks.named("preBuild") { dependsOn(stageGameAssets) }
