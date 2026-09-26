package game.hexkeep

import androidx.test.core.app.ActivityScenario
import androidx.test.ext.junit.runners.AndroidJUnit4
import androidx.test.platform.app.InstrumentationRegistry
import android.content.Intent
import android.graphics.Bitmap
import android.os.SystemClock
import android.view.MotionEvent
import org.json.JSONObject
import org.junit.Assert.*
import org.junit.Test
import org.junit.runner.RunWith
import java.io.File

@RunWith(AndroidJUnit4::class)
class GameplayTest {
    private val instrumentation=InstrumentationRegistry.getInstrumentation()
    private fun tap(x:Int,y:Int){
        val t=SystemClock.uptimeMillis()
        // Pixel 5 test AVD: 2340x1080, internal 584x240, 4x scale, margins 2/60.
        instrumentation.sendPointerSync(MotionEvent.obtain(t,t,MotionEvent.ACTION_DOWN,(2+x*4).toFloat(),(60+y*4).toFloat(),0))
        instrumentation.sendPointerSync(MotionEvent.obtain(t,t+40,MotionEvent.ACTION_UP,(2+x*4).toFloat(),(60+y*4).toFloat(),0))
        instrumentation.waitForIdleSync()
    }
    private fun capture(name:String){SystemClock.sleep(350);val image=instrumentation.uiAutomation.takeScreenshot();val f=File(instrumentation.targetContext.getExternalFilesDir(null),"$name.png");f.outputStream().use{image.compress(Bitmap.CompressFormat.PNG,100,it)}}
    @Test fun soloOfflineLifecycle(){
        var snapshot=""
        ActivityScenario.launch<MainActivity>(Intent(instrumentation.targetContext,MainActivity::class.java)).use{scenario->
            SystemClock.sleep(1200)
            capture("title")
            var created=false
            scenario.onActivity{created=JSONObject(it.engine.snapshot()).getBoolean("created")}
            tap(292,165)
            if(!created){
                tap(120,204);tap(450,210);tap(450,210)
                scenario.onActivity{assertTrue(it.engine.debugStatus().contains("screen=4"))}
                tap(450,218)
                for(i in 0..2){var correct=0;scenario.onActivity{correct=JSONObject(it.engine.snapshot()).getJSONArray("secret").getInt(10+i)%3};tap(292,111+correct*29)}
                scenario.onActivity{assertTrue(it.engine.debugStatus().contains("screen=6"))}
                capture("battle")
                val down=SystemClock.uptimeMillis()
                instrumentation.sendPointerSync(MotionEvent.obtain(down,down,MotionEvent.ACTION_DOWN,290f,820f,0))
                SystemClock.sleep(450)
                instrumentation.sendPointerSync(MotionEvent.obtain(down,down+450,MotionEvent.ACTION_UP,290f,820f,0))
                tap(560,9);tap(292,119)
            }
            scenario.onActivity{assertTrue(it.engine.debugStatus().contains("screen=7"))}
            tap(500,120)
            scenario.onActivity{
                snapshot=it.engine.snapshot()
                val obj=JSONObject(snapshot)
                assertTrue(obj.getBoolean("created"))
                assertTrue(obj.getJSONObject("ledger").getJSONArray("events").length()>=2)
                it.persist()
                val encrypted=File(it.filesDir,"state.hk").readBytes()
                assertNotEquals(snapshot,encrypted.toString(Charsets.UTF_8))
                assertEquals(snapshot,Vault(it).read())
            }
            capture("world")
        }
        ActivityScenario.launch<MainActivity>(Intent(instrumentation.targetContext,MainActivity::class.java)).use{scenario->
            SystemClock.sleep(800)
            scenario.onActivity{
                val before=JSONObject(snapshot);val after=JSONObject(it.engine.snapshot())
                assertEquals(before.getJSONArray("secret").toString(),after.getJSONArray("secret").toString())
                assertEquals(before.getJSONObject("world").getLong("current"),after.getJSONObject("world").getLong("current"))
                assertEquals(before.getJSONObject("ledger").toString(),after.getJSONObject("ledger").toString())
            }
            tap(292,165);capture("restored")
        }
    }
    @Test fun nativeDeterminism(){
        val e=game.hexkeep.core.Engine("",true)
        val digest=e.determinismCheck()
        assertEquals("54f7ff9666521f8b0d41b95742763a1abc94268013332efbe700bfb6289c15cc",digest)
        File(instrumentation.targetContext.getExternalFilesDir(null),"determinism-android.txt").writeText(digest)
    }

}
