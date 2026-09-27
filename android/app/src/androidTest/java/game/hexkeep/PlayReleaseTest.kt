package game.hexkeep

import android.content.Intent
import android.content.pm.PackageManager
import android.graphics.Bitmap
import android.os.SystemClock
import android.view.MotionEvent
import androidx.test.core.app.ActivityScenario
import androidx.test.platform.app.InstrumentationRegistry
import game.hexkeep.core.Engine
import org.json.JSONObject
import org.junit.Assert.*
import org.junit.Assume.assumeTrue
import org.junit.Test
import java.io.File

/** Exercises the installed Play variant, encrypted storage and real touch coordinates. */
class PlayReleaseTest {
    private val test = InstrumentationRegistry.getInstrumentation()
    private lateinit var activity: MainActivity
    private lateinit var engine: Engine
    private fun state() = JSONObject(engine.presentation(584))
    private fun launch(): ActivityScenario<MainActivity> {
        val scenario = ActivityScenario.launch<MainActivity>(Intent(test.targetContext, MainActivity::class.java))
        SystemClock.sleep(1200)
        scenario.onActivity { activity = it; engine = it.engine }
        return scenario
    }
    private fun tap(x: Float, y: Float) {
        val viewport = JSONObject(activity.renderMetrics()).getJSONObject("viewport")
        val px = viewport.getDouble("left").toFloat() + x * viewport.getDouble("scale").toFloat()
        val py = viewport.getDouble("top").toFloat() + y * viewport.getDouble("scale").toFloat()
        val now = SystemClock.uptimeMillis()
        for (action in listOf(MotionEvent.ACTION_DOWN, MotionEvent.ACTION_UP)) {
            val event = MotionEvent.obtain(now, SystemClock.uptimeMillis(), action, px, py, 0).apply { source=android.view.InputDevice.SOURCE_TOUCHSCREEN }
            assertTrue(test.uiAutomation.injectInputEvent(event, true)); event.recycle()
            SystemClock.sleep(60)
        }
        SystemClock.sleep(250)
    }
    private fun shot(name: String) {
        SystemClock.sleep(150)
        File(test.targetContext.getExternalFilesDir(null), "v06-$name.png").outputStream().use {
            test.uiAutomation.takeScreenshot().compress(Bitmap.CompressFormat.PNG, 100, it)
        }
    }
    @Test fun permissionsAndMenusMatchTheOfflinePromise() {
        assumeTrue(BuildConfig.OFFLINE_EDITION)
        val info = test.targetContext.packageManager.getPackageInfo(test.targetContext.packageName, PackageManager.GET_PERMISSIONS)
        val permissions = info.requestedPermissions.orEmpty().toSet()
        assertEquals(setOf("android.permission.VIBRATE"), permissions)
        launch().use {
            val icon=Bitmap.createBitmap(512,512,Bitmap.Config.ARGB_8888)
            val drawable=activity.getDrawable(R.drawable.icon)!!
            drawable.setBounds(0,0,512,512);drawable.draw(android.graphics.Canvas(icon))
            File(test.targetContext.getExternalFilesDir(null),"v06-store-icon.png").outputStream().use { icon.compress(Bitmap.CompressFormat.PNG,100,it) }
            assertTrue(state().getBoolean("offline")); assertFalse(state().getBoolean("blocked"))
            engine.uiAction("home"); SystemClock.sleep(300); shot("refuge")
            val width = JSONObject(activity.renderMetrics()).getJSONObject("viewport").getDouble("width").toFloat()
            tap(width - 119, 57f); assertEquals(10, state().getInt("screen")); shot("settings")
            val before = state().getBoolean("music")
            tap(200f, 155f); assertEquals(!before, state().getBoolean("music")); tap(200f,155f)
            tap(150f, 384f); shot("help"); tap(width-110,57f)
            tap(width/2,384f); assertTrue(test.uiAutomation.rootInActiveWindow.findAccessibilityNodeInfosByText("Aucune transmission par le jeu").isNotEmpty())
            test.uiAutomation.performGlobalAction(android.accessibilityservice.AccessibilityService.GLOBAL_ACTION_BACK)
            SystemClock.sleep(300)
            for (action in listOf("network", "connect", "gps", "shop", "identity")) engine.uiAction(action)
            assertEquals(10, state().getInt("screen")); assertEquals(0, engine.networkPeers().toInt())
            assertTrue(Engine.offline("broken save").canSave().not())
        }
    }
    @Test fun interruptedAdventureRestoresExactlyAndBackKeepsItsPause() {
        assumeTrue(BuildConfig.OFFLINE_EDITION)
        var expected = ""
        launch().use { scenario ->
            engine.uiAction("home"); engine.uiAction("prologue"); SystemClock.sleep(500)
            engine.uiAction("home"); engine.uiAction("contract:2"); SystemClock.sleep(1000)
            engine.uiAction("pause"); SystemClock.sleep(150)
            expected = state().getJSONObject("battle").toString()
            test.runOnMainSync { activity.persist() }
            shot("pause")
        }
        launch().use {
            assertTrue(state().getBoolean("resumable")); assertEquals(0, state().getInt("screen")); shot("continue")
            tap(200f,410f); assertEquals(14,state().getInt("screen"))
            assertEquals(expected,state().getJSONObject("battle").toString())
            val width=JSONObject(activity.renderMetrics()).getJSONObject("viewport").getDouble("width").toFloat()
            tap(width/2-90,300f); assertEquals(10,state().getInt("screen"))
            test.uiAutomation.performGlobalAction(android.accessibilityservice.AccessibilityService.GLOBAL_ACTION_BACK)
            SystemClock.sleep(350); assertEquals(14,state().getInt("screen"))
            assertEquals(expected,state().getJSONObject("battle").toString())
            tap(width/2,224f); assertEquals(6,state().getInt("screen"))
            SystemClock.sleep(800); assertNotEquals(expected,state().getJSONObject("battle").toString()); shot("adventure")
            engine.uiAction("home")
        }
    }
    @Test fun unreadableVaultIsNeverOverwrittenOnPause() {
        assumeTrue(BuildConfig.OFFLINE_EDITION)
        // Isolated preview app storage, preserved even if an assertion fails.
        val file=File(test.targetContext.filesDir,"state.hk")
        val original=if(file.exists())file.readBytes()else null
        val invalid="unreadable-test-ciphertext".toByteArray()
        try {
            file.writeBytes(invalid)
            launch().use { scenario -> assertFalse(engine.canSave()); test.runOnMainSync{activity.persist()} }
            assertArrayEquals(invalid,file.readBytes())
        } finally { if(original==null)file.delete()else file.writeBytes(original) }
    }
}
