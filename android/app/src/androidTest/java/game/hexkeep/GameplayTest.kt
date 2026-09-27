package game.hexkeep

import androidx.test.core.app.ActivityScenario
import androidx.test.ext.junit.runners.AndroidJUnit4
import androidx.test.platform.app.InstrumentationRegistry
import androidx.lifecycle.Lifecycle
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
    private var scale=2f
    private var vw=1170f
    private var top=0f
    private lateinit var engine:game.hexkeep.core.Engine
    private lateinit var activity:MainActivity
    private fun onActivity(block:(MainActivity)->Unit){instrumentation.runOnMainSync{block(activity)}}
    private fun tap(x:Float,y:Float){val t=SystemClock.uptimeMillis();instrumentation.uiAutomation.injectInputEvent(MotionEvent.obtain(t,t,MotionEvent.ACTION_DOWN,x*scale,y*scale+top,0),false);SystemClock.sleep(60);instrumentation.uiAutomation.injectInputEvent(MotionEvent.obtain(t,SystemClock.uptimeMillis(),MotionEvent.ACTION_UP,x*scale,y*scale+top,0),false);SystemClock.sleep(200)}
    private fun capture(name:String){SystemClock.sleep(350);val image=instrumentation.uiAutomation.takeScreenshot();val f=File(instrumentation.targetContext.getExternalFilesDir(null),"v05-$name.png");f.outputStream().use{image.compress(Bitmap.CompressFormat.PNG,100,it)}}
    private fun point(down:Long,action:Int,points:List<Triple<Int,Float,Float>>){val props=points.map{MotionEvent.PointerProperties().apply{id=it.first;toolType=MotionEvent.TOOL_TYPE_FINGER}}.toTypedArray();val coords=points.map{MotionEvent.PointerCoords().apply{x=it.second*scale;y=it.third*scale+top;pressure=1f;size=1f}}.toTypedArray();instrumentation.uiAutomation.injectInputEvent(MotionEvent.obtain(down,SystemClock.uptimeMillis(),action,points.size,props,coords,0,0,1f,1f,0,0,android.view.InputDevice.SOURCE_TOUCHSCREEN,0),false);SystemClock.sleep(60)}
    @Test fun soloOfflineLifecycle(){
        var identity="";var storedXp=0
        ActivityScenario.launch<MainActivity>(Intent(instrumentation.targetContext,MainActivity::class.java)).use{scenario->
            SystemClock.sleep(1400)
            scenario.onActivity{activity=it;engine=it.engine;scale=minOf(it.window.decorView.height/540f,it.window.decorView.width/960f);top=(it.window.decorView.height-540*scale)/2;vw=it.window.decorView.width/scale;identity=JSONObject(it.engine.snapshot()).getJSONArray("secret").toString()}
            capture("title")
            // Replaying the introduction also exercises upgrades from a v0.2 identity.
            onActivity{it.engine.uiAction("settings")};SystemClock.sleep(200);capture("settings");tap(150f,470f);SystemClock.sleep(1200);capture("intro-1")
            tap(vw-190,440f);SystemClock.sleep(700);capture("intro-2");tap(vw-190,440f);SystemClock.sleep(700);capture("intro-3");tap(vw-190,440f);capture("heroes")
            tap(vw/2,265f);tap(vw-200,447f)
            capture("first-combat-frame")
            var beforeX=0
            onActivity{val p=JSONObject(it.engine.presentation(584));assertEquals(6,p.getInt("screen"));assertEquals(1,p.getInt("realm"));beforeX=p.getJSONObject("battle").getJSONArray("fighters").getJSONObject(0).getJSONObject("pos").getInt("x")}
            val down=SystemClock.uptimeMillis()
            point(down,MotionEvent.ACTION_DOWN,listOf(Triple(0,95f,435f)))
            point(down,MotionEvent.ACTION_MOVE,listOf(Triple(0,138f,435f)))
            SystemClock.sleep(1400)
            point(down,MotionEvent.ACTION_POINTER_DOWN or (1 shl MotionEvent.ACTION_POINTER_INDEX_SHIFT),listOf(Triple(0,138f,435f),Triple(1,vw-91,326f)))
            point(down,MotionEvent.ACTION_POINTER_UP or (1 shl MotionEvent.ACTION_POINTER_INDEX_SHIFT),listOf(Triple(0,138f,435f),Triple(1,vw-91,326f)))
            // Two pointers: movement remains held while triggering the dash.
            point(down,MotionEvent.ACTION_POINTER_DOWN or (1 shl MotionEvent.ACTION_POINTER_INDEX_SHIFT),listOf(Triple(0,138f,435f),Triple(1,vw-91,445f)))
            point(down,MotionEvent.ACTION_POINTER_UP or (1 shl MotionEvent.ACTION_POINTER_INDEX_SHIFT),listOf(Triple(0,138f,435f),Triple(1,vw-91,445f)))
            SystemClock.sleep(100)
            run{val f=JSONObject(engine.presentation(584)).getJSONObject("battle").getJSONArray("fighters").getJSONObject(0);assertTrue("Movement failed: $f before=$beforeX",f.getJSONObject("pos").getInt("x")>beforeX);assertTrue("Dash failed: $f",f.getInt("dash_cd")>0)}
            point(down,MotionEvent.ACTION_CANCEL,listOf(Triple(0,138f,435f)));tap(vw-199,445f)
            run{val f=JSONObject(engine.presentation(584)).getJSONObject("battle").getJSONArray("fighters").getJSONObject(0);assertTrue(f.getInt("skill_cd")>0)}
            capture("prologue-combat");tap(vw-60,40f);capture("pause");tap(vw/2,if(JSONObject(engine.presentation(584)).getInt("screen")==12)446f else 350f)
            onActivity{assertEquals(7,JSONObject(it.engine.presentation(584)).getInt("screen"))}
            capture("refuge");tap(140f,470f);capture("map");tap(vw*.40f+57f,303f);assertFalse(JSONObject(engine.presentation(584)).getBoolean("selected_current"));tap(vw-268,312f);assertTrue(JSONObject(engine.presentation(584)).getString("message").contains("Rejoins"));tap(vw-110,60f)
            tap(200f,400f);capture("journal");tap(180f,408f);SystemClock.sleep(900);capture("expedition")
            onActivity{assertEquals(8,JSONObject(it.engine.presentation(584)).getInt("battle_mode"))}
            scenario.moveToState(Lifecycle.State.CREATED);SystemClock.sleep(300)
            var pausedTick=0
            onActivity{val p=JSONObject(it.engine.presentation(584));assertEquals(14,p.getInt("screen"));pausedTick=p.getJSONObject("battle").getInt("tick")}
            SystemClock.sleep(600)
            onActivity{assertEquals(pausedTick,JSONObject(it.engine.presentation(584)).getJSONObject("battle").getInt("tick"))}
            scenario.moveToState(Lifecycle.State.RESUMED);SystemClock.sleep(300);tap(vw/2,275f);SystemClock.sleep(200)
            onActivity{assertEquals(6,JSONObject(it.engine.presentation(584)).getInt("screen"));it.engine.uiAction("home")}
            SystemClock.sleep(250)
            onActivity{
                val snapshot=it.engine.snapshot();val obj=JSONObject(snapshot);assertTrue(obj.getBoolean("created"));assertTrue(obj.getBoolean("introduction_seen"));assertEquals(identity,obj.getJSONArray("secret").toString());storedXp=obj.getJSONObject("expansion").getJSONObject("campaign").getInt("xp");it.persist();assertEquals(snapshot,Vault(it).read());assertNotEquals(snapshot,File(it.filesDir,"state.hk").readText());File(it.getExternalFilesDir(null),"v05-render.json").writeText(it.renderMetrics())
            }
        }
        ActivityScenario.launch<MainActivity>(Intent(instrumentation.targetContext,MainActivity::class.java)).use{scenario->SystemClock.sleep(1100);scenario.onActivity{activity=it};onActivity{val saved=JSONObject(it.engine.snapshot());assertEquals(identity,saved.getJSONArray("secret").toString());assertTrue("Saved XP must survive; passive watch can add an éclat during relaunch",saved.getJSONObject("expansion").getJSONObject("campaign").getInt("xp")>=storedXp);assertTrue(saved.getBoolean("introduction_seen"))};tap(180f,410f);capture("restored")}
    }
    @Test fun nativeDeterminism(){val e=game.hexkeep.core.Engine("",true);val digest=e.determinismCheck();assertEquals("015c7661915282832b72f2943daa4b53888f18f01fc1b0034a2a7a79ac0ef997",digest);File(instrumentation.targetContext.getExternalFilesDir(null),"determinism-android.txt").writeText(digest)}
}
