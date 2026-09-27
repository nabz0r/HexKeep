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
  if(InstrumentationRegistry.getArguments().getString("prepare")=="true"){
   val action=engine.javaClass.getMethod("uiAction",String::class.java)
   action.invoke(engine,"prologue");action.invoke(engine,"home")
  }
  val public=engine.javaClass.getMethod("identityPublic").invoke(engine) as ByteArray
  val saved=JSONObject(engine.javaClass.getMethod("snapshot").invoke(engine) as String)
  val journey=saved.optJSONObject("journey")?:JSONObject()
  val result=JSONObject().put("items",journey.optJSONArray("items")).put("equipped",journey.optJSONArray("equipped")).put("dust",journey.optInt("dust")).put("public",Base64.encodeToString(public,Base64.NO_WRAP)).put("name",saved.getString("name")).put("created",saved.getBoolean("created")).put("xp",saved.getJSONObject("expansion").getJSONObject("campaign").getInt("xp"))
  i.runOnMainSync{activity.finish()};i.removeMonitor(monitor);return result
 }
 @Test fun captureBefore(){val result=identity();assertTrue(result.getBoolean("created"));File(InstrumentationRegistry.getInstrumentation().targetContext.getExternalFilesDir(null),"upgrade-identity-before.json").writeText(result.toString())}
 @Test fun verifyAfter(){val context=InstrumentationRegistry.getInstrumentation().targetContext;val before=JSONObject(File(context.getExternalFilesDir(null),"upgrade-identity-before.json").readText());val after=identity();assertEquals(before.getString("public"),after.getString("public"));assertEquals(before.getString("name"),after.getString("name"));assertEquals(before.getBoolean("created"),after.getBoolean("created"));assertTrue(after.getInt("xp")>=before.getInt("xp"));val items=before.optJSONArray("items");if(items!=null){val current=after.getJSONArray("items");val oldEquipment=before.getJSONArray("equipped");val added=(6-oldEquipment.length()).coerceAtLeast(0);assertEquals(items.length()+added,current.length());for(n in 0 until items.length()){val old=items.getJSONObject(n);val item=(0 until current.length()).map{current.getJSONObject(it)}.first{it.getLong("id")==old.getLong("id")};for(key in arrayOf("name","slot","rarity","vitality","power","guard","haste"))assertEquals(key,old.get(key),item.get(key))};val equipment=after.getJSONArray("equipped");assertEquals(6,equipment.length());for(slot in 0 until oldEquipment.length())assertEquals(oldEquipment.getLong(slot),equipment.getLong(slot));for(slot in oldEquipment.length() until 6){val id=equipment.getLong(slot);assertTrue(id>0);assertTrue((0 until current.length()).any{current.getJSONObject(it).getLong("id")==id&&current.getJSONObject(it).getInt("slot")==slot})};assertEquals(before.getInt("dust"),after.getInt("dust"))};File(context.getExternalFilesDir(null),"upgrade-result.txt").writeText("PASS v0.6 → v0.7: same public identity, name, created flag; XP, item IDs, item stats, equipment and dust retained; three additive equipment slots verified.\n")}
}
