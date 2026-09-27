package game.hexkeep
import android.content.Intent
import android.location.Location
import android.os.SystemClock
import androidx.test.core.app.ActivityScenario
import androidx.test.platform.app.InstrumentationRegistry
import org.json.JSONObject
import org.junit.Assert.*
import org.junit.Test
/** Injected platform callbacks exercise the same accuracy/speed gate and H3 path as GPS. */
class GeolocationTest {
 @Test fun discoveryAccuracyAndSimulationSwitch(){
  val i=InstrumentationRegistry.getInstrumentation()
  for(p in arrayOf(android.Manifest.permission.ACCESS_COARSE_LOCATION,android.Manifest.permission.ACCESS_FINE_LOCATION))i.uiAutomation.grantRuntimePermission("game.hexkeep.dev",p)
  ActivityScenario.launch<MainActivity>(Intent(i.targetContext,MainActivity::class.java)).use{scenario->
   SystemClock.sleep(1100);lateinit var a:MainActivity;scenario.onActivity{a=it;it.engine.uiAction("home")}
   fun world()=JSONObject(a.engine.snapshot()).getJSONObject("world")
   val base=SystemClock.elapsedRealtimeNanos()
   fun fix(lat:Double,accuracy:Float,sec:Long){i.runOnMainSync{a.onLocationChanged(Location("gps").apply{latitude=lat;longitude=6.1319;this.accuracy=accuracy;elapsedRealtimeNanos=base+sec*1_000_000_000;time=System.currentTimeMillis()})}}
   fix(49.6116,5f,0);val first=world().getLong("current");assertTrue(world().getBoolean("gps"))
   fix(49.62,150f,10);assertEquals(first,world().getLong("current"))
   fix(50.0,5f,11);assertEquals(first,world().getLong("current"))
   fix(49.6156,5f,120);assertNotEquals(first,world().getLong("current"));SystemClock.sleep(1100);assertTrue(JSONObject(a.engine.presentation(584)).getJSONObject("journey").getInt("discovered")>=2)
   val cells=JSONObject(a.engine.presentation(584)).getJSONArray("cells");val next=(0 until cells.length()).map{cells.getJSONObject(it)}.first{!it.getBoolean("current")};a.engine.uiAction("cell:"+next.getString("id"));a.engine.uiAction("walk");SystemClock.sleep(300);assertFalse(world().getBoolean("gps"));assertFalse(a.getPreferences(0).getBoolean("gps-enabled",true))
  }
 }
}
