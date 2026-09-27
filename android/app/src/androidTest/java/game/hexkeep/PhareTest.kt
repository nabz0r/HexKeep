package game.hexkeep
import androidx.test.core.app.ActivityScenario
import androidx.lifecycle.Lifecycle
import androidx.test.ext.junit.runners.AndroidJUnit4
import androidx.test.platform.app.InstrumentationRegistry
import android.content.Intent
import android.os.SystemClock
import org.json.JSONObject
import org.junit.Assert.*
import org.junit.Test
import org.junit.runner.RunWith
import java.io.File
@RunWith(AndroidJUnit4::class)class PhareTest {
 @Test fun foregroundWatchAndRender(){val instrumentation=InstrumentationRegistry.getInstrumentation();ActivityScenario.launch<MainActivity>(Intent(instrumentation.targetContext,MainActivity::class.java)).use{scenario->SystemClock.sleep(800);lateinit var engine:game.hexkeep.core.Engine;scenario.onActivity{engine=it.engine;assertTrue(JSONObject(engine.snapshot()).getBoolean("created"));engine.touch(11,0u,290,166);engine.touch(11,2u,290,166);engine.touch(11,0u,40,10);engine.touch(11,2u,40,10);engine.touch(11,0u,450,152);engine.touch(11,2u,450,152);engine.touch(11,0u,100,107);engine.touch(11,2u,100,107)};SystemClock.sleep(2500);assertTrue("Phare could not start",engine.phare());val before=Regex("ticks=(\\d+)").find(engine.debugStatus())!!.groupValues[1].toLong();scenario.moveToState(Lifecycle.State.CREATED);SystemClock.sleep(3000);val after=Regex("ticks=(\\d+)").find(engine.debugStatus())!!.groupValues[1].toLong();assertTrue("Background simulation stopped",after-before>=60);engine.stopPhare("Test de veille terminé.");scenario.moveToState(Lifecycle.State.RESUMED);SystemClock.sleep(5000);scenario.onActivity{File(it.getExternalFilesDir(null),"v02-render-metrics.json").writeText(it.renderMetrics());it.persist()};assertFalse(engine.phare())}}
}
