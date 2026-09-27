package game.hexkeep
import androidx.test.core.app.ActivityScenario
import androidx.test.ext.junit.runners.AndroidJUnit4
import androidx.test.platform.app.InstrumentationRegistry
import android.content.Intent
import android.graphics.Bitmap
import android.os.SystemClock
import org.json.JSONObject
import org.junit.Assert.*
import org.junit.Test
import org.junit.runner.RunWith
import java.io.File
@RunWith(AndroidJUnit4::class)
class ExpansionTest {
 private val instrumentation=InstrumentationRegistry.getInstrumentation()
 @Test fun campaignThroneShopAndKeystore(){
 ActivityScenario.launch<MainActivity>(Intent(instrumentation.targetContext,MainActivity::class.java)).use{scenario->
  SystemClock.sleep(1000)
  fun tap(x:Int,y:Int){scenario.onActivity{it.engine.touch(71,0u,x,y);it.engine.touch(71,2u,x,y)};SystemClock.sleep(100)}
  fun shot(name:String){SystemClock.sleep(200);val b=instrumentation.uiAutomation.takeScreenshot();File(instrumentation.targetContext.getExternalFilesDir(null),"v05-$name.png").outputStream().use{b.compress(Bitmap.CompressFormat.PNG,100,it)}}
  scenario.onActivity{assertTrue(DeviceIdentity.certify(it,it.engine))}
  var created=false;scenario.onActivity{created=JSONObject(it.engine.snapshot()).getBoolean("created")}
  tap(290,166)
  if(!created){tap(120,204);tap(450,210);tap(450,210);tap(450,218);for(i in 0..2){var c=0;scenario.onActivity{c=JSONObject(it.engine.snapshot()).getJSONArray("secret").getInt(10+i)%3};tap(290,111+c*29)};tap(560,9);tap(292,119)}
  tap(40,10);shot("campaign")
  // Store, obtain a test cosmetic, equip it, and return.
  tap(420,108);scenario.onActivity{assertTrue(it.engine.debugStatus().contains("screen=24"))};tap(100,62);tap(100,107);tap(100,152);shot("shop")
  scenario.onActivity{val c=JSONObject(it.engine.snapshot()).getJSONObject("expansion").getJSONObject("campaign");assertTrue(c.getJSONArray("owned").toString().contains("skin_pierre"));assertEquals("skin_pierre",c.getString("equipped"))}
  tap(560,10);tap(100,152);shot("heraldry");tap(40,60);tap(350,105);scenario.onActivity{assertTrue(JSONObject(it.engine.snapshot()).getJSONObject("expansion").getJSONObject("campaign").getJSONArray("houses").length()>0)}
  tap(560,10);tap(100,198);SystemClock.sleep(500);scenario.onActivity{assertTrue(it.engine.debugStatus().contains("screen=27"))};tap(100,62);shot("throne")
  scenario.onActivity{val e=JSONObject(it.engine.snapshot()).getJSONObject("expansion");assertTrue(e.getJSONObject("authority").getJSONArray("seals").length()>0);it.persist();assertEquals(it.engine.snapshot(),Vault(it).read())}
  tap(420,153);tap(100,62);shot("codex");tap(420,10);tap(560,10);tap(100,198);tap(100,62)
 }
 }
}
