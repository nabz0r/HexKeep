package game.hexkeep
import android.content.Intent
import android.os.SystemClock
import androidx.test.core.app.ActivityScenario
import androidx.test.platform.app.InstrumentationRegistry
import org.json.JSONObject
import org.junit.Test
import org.junit.Assert.*
import java.io.File
class PresentationPerformanceTest {
 @Test fun steadyBattle(){
  val i=InstrumentationRegistry.getInstrumentation()
  ActivityScenario.launch<MainActivity>(Intent(i.targetContext,MainActivity::class.java)).use{s->
   SystemClock.sleep(900);lateinit var activity:MainActivity;s.onActivity{activity=it;it.engine.uiAction("expedition")}
   SystemClock.sleep(1800);var before=JSONObject();var after=JSONObject()
   i.runOnMainSync{before=JSONObject(activity.renderMetrics())}
   SystemClock.sleep(12000)
   i.runOnMainSync{after=JSONObject(activity.renderMetrics())}
   val seconds=after.getDouble("seconds")-before.getDouble("seconds");val frames=after.getLong("frames")-before.getLong("frames");val fps=frames/seconds
   val report=JSONObject().put("seconds",seconds).put("frames",frames).put("fps",fps).put("over_33ms",after.getLong("over_33ms")-before.getLong("over_33ms")).put("screen_width",activity.window.decorView.width).put("screen_height",activity.window.decorView.height)
   File(i.targetContext.getExternalFilesDir(null),"v04-performance.json").writeText(report.toString())
   assertTrue("Renderer did not progress: $report",fps>=20)
   i.runOnMainSync{activity.engine.uiAction("home");activity.finish()}
  }
 }
}
