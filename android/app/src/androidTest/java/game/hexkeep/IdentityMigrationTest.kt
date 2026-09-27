package game.hexkeep
import android.app.Activity
import android.content.Intent
import android.os.SystemClock
import android.util.Base64
import androidx.test.platform.app.InstrumentationRegistry
import org.json.JSONObject
import org.junit.Test
import org.junit.Assert.*
import java.io.File
/** Reflection deliberately uses only the v0.2 API, so this test can run before an APK upgrade. */
class IdentityMigrationTest {
 private fun identity():JSONObject {
  val i=InstrumentationRegistry.getInstrumentation();val monitor=i.addMonitor("game.hexkeep.MainActivity",null,false)
  i.targetContext.startActivity(Intent().setClassName("game.hexkeep.dev","game.hexkeep.MainActivity").addFlags(Intent.FLAG_ACTIVITY_NEW_TASK))
  val activity=i.waitForMonitorWithTimeout(monitor,5000) as Activity? ?: error("Activity absent")
  SystemClock.sleep(1000)
  val engine=activity.javaClass.getMethod("getEngine").invoke(activity)
  val public=engine.javaClass.getMethod("identityPublic").invoke(engine) as ByteArray
  val saved=JSONObject(engine.javaClass.getMethod("snapshot").invoke(engine) as String)
  val result=JSONObject().put("public",Base64.encodeToString(public,Base64.NO_WRAP)).put("name",saved.getString("name")).put("created",saved.getBoolean("created")).put("xp",saved.getJSONObject("expansion").getJSONObject("campaign").getInt("xp"))
  i.runOnMainSync{activity.finish()};i.removeMonitor(monitor);return result
 }
 @Test fun captureBefore(){val result=identity();assertTrue(result.getBoolean("created"));File(InstrumentationRegistry.getInstrumentation().targetContext.getExternalFilesDir(null),"upgrade-identity-before.json").writeText(result.toString())}
 @Test fun verifyAfter(){val context=InstrumentationRegistry.getInstrumentation().targetContext;val before=JSONObject(File(context.getExternalFilesDir(null),"upgrade-identity-before.json").readText());val after=identity();assertEquals(before.getString("public"),after.getString("public"));assertEquals(before.getString("name"),after.getString("name"));assertEquals(before.getBoolean("created"),after.getBoolean("created"));assertTrue(after.getInt("xp")>=before.getInt("xp"));File(context.getExternalFilesDir(null),"upgrade-result.txt").writeText("PASS v0.3 → v0.4: same public identity, name, created flag; campaign XP retained.\n")}
}
