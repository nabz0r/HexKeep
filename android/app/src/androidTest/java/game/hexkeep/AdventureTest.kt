package game.hexkeep

import android.content.Intent
import android.graphics.Bitmap
import android.os.SystemClock
import android.view.MotionEvent
import androidx.test.core.app.ActivityScenario
import androidx.test.platform.app.InstrumentationRegistry
import org.json.JSONObject
import org.junit.Assert.*
import org.junit.Test
import java.io.File
import kotlin.math.*

/** A full expedition using injected fingers. No warps, forced damage or generated rewards. */
class AdventureTest {
 private val test=InstrumentationRegistry.getInstrumentation()
 private var scale=2f;private var top=0f;private var vw=1170f
 private var down=0L
 private lateinit var activity:MainActivity
 private lateinit var engine:game.hexkeep.core.Engine
 private fun event(action:Int,points:List<Pair<Float,Float>>){
  val props=points.indices.map{MotionEvent.PointerProperties().apply{id=it;toolType=MotionEvent.TOOL_TYPE_FINGER}}.toTypedArray()
  val coords=points.map{MotionEvent.PointerCoords().apply{x=it.first*scale;y=it.second*scale+top;pressure=1f;size=1f}}.toTypedArray()
  test.uiAutomation.injectInputEvent(MotionEvent.obtain(down,SystemClock.uptimeMillis(),action,points.size,props,coords,0,0,1f,1f,0,0,android.view.InputDevice.SOURCE_TOUCHSCREEN,0),false)
 }
 private fun tap(x:Float,y:Float){down=SystemClock.uptimeMillis();event(MotionEvent.ACTION_DOWN,listOf(x to y));SystemClock.sleep(70);event(MotionEvent.ACTION_UP,listOf(x to y));SystemClock.sleep(300)}
 private fun shot(name:String){SystemClock.sleep(150);val b=test.uiAutomation.takeScreenshot();File(test.targetContext.getExternalFilesDir(null),"v05-$name.png").outputStream().use{b.compress(Bitmap.CompressFormat.PNG,100,it)}}
 private fun state()=JSONObject(engine.presentation(584))
 private fun route(b:JSONObject,goal:Pair<Float,Float>):Pair<Float,Float>{
  val p=b.getJSONArray("fighters").getJSONObject(0).getJSONObject("pos");val px=p.getInt("x").toFloat();val py=p.getInt("y").toFloat()
  if(hypot(goal.first-px,goal.second-py)<230)return 0f to 0f
  val obs=b.getJSONArray("obstacles")
  fun free(x:Float,y:Float):Boolean=x>=64&&y>=64&&x<=7616&&y<=4032&&(0 until obs.length()).none{val o=obs.getJSONObject(it);val nx=x.coerceIn(o.getInt("x").toFloat(),(o.getInt("x")+o.getInt("w")).toFloat());val ny=y.coerceIn(o.getInt("y").toFloat(),(o.getInt("y")+o.getInt("h")).toFloat());(x-nx)*(x-nx)+(y-ny)*(y-ny)<64*64}
  fun segment(ax:Float,ay:Float,bx:Float,by:Float):Boolean{val n=(max(abs(bx-ax),abs(by-ay))/16).toInt()+1;return(0..n).all{free(ax+(bx-ax)*it/n,ay+(by-ay)*it/n)}}
  fun clear(x:Float,y:Float)=segment(px,py,x,y)
  var dest=goal
  if(!clear(goal.first,goal.second)){
   val start=(py.toInt()/256).coerceIn(0,15)*30+(px.toInt()/256).coerceIn(0,29);val end=(goal.second.toInt()/256).coerceIn(0,15)*30+(goal.first.toInt()/256).coerceIn(0,29)
   val prev=IntArray(480){-1};val queue=java.util.ArrayDeque<Int>();queue.add(start);prev[start]=start
   while(queue.isNotEmpty()){val at=queue.removeFirst();if(at==end)break;for((dx,dy) in listOf(1 to 0,0 to 1,-1 to 0,0 to -1)){val x=at%30+dx;val y=at/30+dy;if(x !in 0..29||y !in 0..15)continue;val next=y*30+x;if(prev[next]>=0||!free(x*256f+128,y*256f+128))continue;if(!segment(if(at==start)px else at%30*256f+128,if(at==start)py else at/30*256f+128,x*256f+128,y*256f+128))continue;prev[next]=at;queue.add(next)}}
   if(prev[end]>=0){var at=end;while(at!=start){val x=at%30*256f+128;val y=at/30*256f+128;if(clear(x,y)){dest=x to y;break};at=prev[at]}}
  }
  val dx=dest.first-px;val dy=dest.second-py;val len=hypot(dx,dy).coerceAtLeast(1f);return dx/len to dy/len
 }
 @Test fun completeLootEquipAndRestore(){
  var identity="";var count=0
  ActivityScenario.launch<MainActivity>(Intent(test.targetContext,MainActivity::class.java)).use{scenario->
   SystemClock.sleep(1000);scenario.onActivity{activity=it;engine=it.engine;scale=min(it.window.decorView.height/540f,it.window.decorView.width/960f);top=(it.window.decorView.height-540*scale)/2;vw=it.window.decorView.width/scale;it.engine.uiAction("home");it.engine.hero(0u,0u);identity=it.engine.identityPublic().contentToString()}
   SystemClock.sleep(250);tap(200f,398f);assertEquals(41,state().getInt("screen"));shot("journal")
   val cw=(vw-128)/3;tap(48+2*(cw+16)+cw/2,407f);SystemClock.sleep(450);assertEquals(6,state().getInt("screen"))
   val metricBefore=JSONObject(activity.renderMetrics());val combatStarted=SystemClock.uptimeMillis()
   down=SystemClock.uptimeMillis();event(MotionEvent.ACTION_DOWN,listOf(100f to 435f));SystemClock.sleep(50);event(MotionEvent.ACTION_POINTER_DOWN or (1 shl 8),listOf(100f to 435f,(vw-91) to 326f))
   val deadline=SystemClock.uptimeMillis()+100000;var captured=false
   while(state().getInt("screen")==6&&SystemClock.uptimeMillis()<deadline){
    val s=state();val b=s.getJSONObject("battle");val run=s.getJSONObject("expedition");val caches=run.getJSONArray("caches");val charges=run.getJSONArray("charges");val p=b.getJSONArray("fighters").getJSONObject(0).getJSONObject("pos")
    val missing=(0..2).firstOrNull{!caches.getBoolean(it)};val beacon=(0..2).firstOrNull{charges.getInt(it)<90}
    val goal=if(missing!=null){val (x,y)=arrayOf(3 to 13,16 to 2,27 to 12)[missing];x*256f to y*256f}else if(beacon!=null){val(x,y)=arrayOf(6 to 4,15 to 12,25 to 5)[beacon];x*256f to y*256f}else{val boss=b.getJSONArray("fighters").getJSONObject(1).getJSONObject("pos");val dx=boss.getInt("x")-p.getInt("x");val dy=boss.getInt("y")-p.getInt("y");if(hypot(dx.toFloat(),dy.toFloat())>800)boss.getInt("x").toFloat() to boss.getInt("y").toFloat() else p.getInt("x").toFloat() to p.getInt("y").toFloat()}
    val (dx,dy)=route(b,goal);event(MotionEvent.ACTION_MOVE,listOf((100+dx*49) to (435+dy*49),(vw-91) to 326f));SystemClock.sleep(90)
    if(!captured&&b.getInt("tick")>210){shot("adventure-touch");captured=true}
   }
   val duration=(SystemClock.uptimeMillis()-combatStarted)/1000.0;val metricAfter=JSONObject(activity.renderMetrics());val fps=(metricAfter.getLong("frames")-metricBefore.getLong("frames"))/duration;File(test.targetContext.getExternalFilesDir(null),"v05-moving-performance.json").writeText(JSONObject().put("fps",fps).put("seconds",duration).put("renderer","SwiftShader emulator, full touchscreen expedition").toString());assertTrue("Moving gameplay below 20 FPS: $fps",fps>=20)
   event(MotionEvent.ACTION_CANCEL,listOf(100f to 435f,(vw-91) to 326f));assertEquals("Expedition timed out: ${state()}",12,state().getInt("screen"));assertTrue(state().getJSONObject("expedition").getBoolean("victory"));assertTrue((0..2).all{state().getJSONObject("expedition").getJSONArray("caches").getBoolean(it)});shot("loot-result")
   tap(vw/2,447f);assertEquals(7,state().getInt("screen"));tap(385f,474f);assertEquals(40,state().getInt("screen"));shot("inventory")
   val j=state().getJSONObject("journey");count=j.getJSONArray("items").length();assertTrue(count>=7)
   // Select a recovered item through its card, then equip it through the actual UI.
   val all=j.getJSONArray("items");val item=(0 until all.length()).map{all.getJSONObject(it)}.maxWith(compareBy<JSONObject>{it.getInt("rarity")}.thenBy{it.getLong("id")});val lw=vw-397;val card=(lw-15)/2;tap(48+card/2,166f);tap(vw-185,344f);assertEquals(item.getLong("id"),state().getJSONObject("journey").getJSONArray("equipped").getLong(item.getInt("slot")));shot("inventory-equipped")
   engine.uiAction("home");SystemClock.sleep(2300)
  }
  ActivityScenario.launch<MainActivity>(Intent(test.targetContext,MainActivity::class.java)).use{scenario->SystemClock.sleep(1000);scenario.onActivity{engine=it.engine;assertEquals(identity,engine.identityPublic().contentToString());assertEquals(count,state().getJSONObject("journey").getJSONArray("items").length())}}
 }
}
