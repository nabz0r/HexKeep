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

/** v0.5 acceptance: actual touches, no relocation, damage injection or generated rewards. */
class EveilTest {
 private val test=InstrumentationRegistry.getInstrumentation()
 private var left=0f;private var scale=2f;private var top=0f;private var vw=1170f
 private var down=0L
 private var gestureDown=false
 private var lastPoints=listOf(100f to 435f)
 private lateinit var activity:MainActivity
 private lateinit var engine:game.hexkeep.core.Engine
 private fun event(action:Int,points:List<Pair<Float,Float>>){
  if(action==MotionEvent.ACTION_DOWN)gestureDown=true
  if(action==MotionEvent.ACTION_UP||action==MotionEvent.ACTION_CANCEL)gestureDown=false
  val props=points.indices.map{MotionEvent.PointerProperties().apply{id=it;toolType=MotionEvent.TOOL_TYPE_FINGER}}.toTypedArray()
  val coords=points.map{MotionEvent.PointerCoords().apply{x=left+it.first*scale;y=it.second*scale+top;pressure=1f;size=1f}}.toTypedArray()
  assertTrue("Touch event rejected",test.uiAutomation.injectInputEvent(MotionEvent.obtain(down,SystemClock.uptimeMillis(),action,points.size,props,coords,0,0,1f,1f,0,0,android.view.InputDevice.SOURCE_TOUCHSCREEN,0),false));lastPoints=points
 }
 private fun tap(x:Float,y:Float,settle:Long=300){down=SystemClock.uptimeMillis();event(MotionEvent.ACTION_DOWN,listOf(x to y));SystemClock.sleep(70);event(MotionEvent.ACTION_UP,listOf(x to y));SystemClock.sleep(settle)}
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
 private fun beginMove(dx:Float,dy:Float,attack:Boolean=false){
  down=SystemClock.uptimeMillis();event(MotionEvent.ACTION_DOWN,listOf(100f to 435f));SystemClock.sleep(60)
  if(attack)event(MotionEvent.ACTION_POINTER_DOWN or (1 shl 8),listOf(100f to 435f,(vw-91) to 326f))
  event(MotionEvent.ACTION_MOVE,if(attack)listOf((100+dx*49) to (435+dy*49),(vw-91) to 326f)else listOf((100+dx*49) to (435+dy*49)))
 }
 private fun stop(){if(!gestureDown)return;val first=lastPoints.take(1);if(lastPoints.size>1)event(MotionEvent.ACTION_POINTER_UP or (1 shl 8),lastPoints);event(MotionEvent.ACTION_UP,first);SystemClock.sleep(120)}
 private fun metrics()=JSONObject(activity.renderMetrics()).getJSONObject("animation")
 @Test fun animateExploreInspectAndResume(){
  ActivityScenario.launch<MainActivity>(Intent(test.targetContext,MainActivity::class.java)).use{scenario->
   SystemClock.sleep(1500);scenario.onActivity{activity=it;engine=it.engine;val viewport=JSONObject(it.renderMetrics()).getJSONObject("viewport");scale=viewport.getDouble("scale").toFloat();left=viewport.getDouble("left").toFloat();top=viewport.getDouble("top").toFloat();vw=viewport.getDouble("width").toFloat();engine.uiAction("continue");engine.uiAction("home");engine.hero(0u,0u)}
   SystemClock.sleep(300);tap(200f,398f);tap(160f,407f);assertEquals(6,state().getInt("screen"))
   for((index,direction) in listOf(1f to 0f,0f to -1f,-1f to 0f,0f to 1f).withIndex()){
    beginMove(direction.first,direction.second);SystemClock.sleep(650);shot("walk-$index");stop()
   }
   tap(vw-91,445f,0);shot("dodge");tap(vw-199,445f)
   beginMove(1f,0f,true);SystemClock.sleep(1600);shot("combat");stop()
   val m=metrics();assertTrue("Player gait: $m",m.getInt("player_walk_frames")>=3);assertTrue("Turning: $m",m.getInt("player_turns")>=2);assertTrue("Attack pose: $m",m.getInt("player_attacks")>0);assertTrue("Dodge pose: $m",m.getInt("player_dodges")>0)
   tap(vw-190,43f);assertEquals(40,state().getInt("screen"));shot("field-inventory")
   val paused=state().getJSONObject("battle");val tick=paused.getInt("tick");val seed=paused.getLong("seed");val hp=paused.getJSONArray("fighters").getJSONObject(0).getInt("hp")
   SystemClock.sleep(850);assertEquals(tick,state().getJSONObject("battle").getInt("tick"));tap(vw-190,344f);assertTrue(state().getJSONObject("battle").getJSONArray("fighters").getJSONObject(0).getInt("hp")<=hp)
   tap(vw-275,56f);assertEquals(42,state().getInt("screen"));shot("bestiary");tap(vw-267,57f);shot("memories");assertEquals(tick,state().getJSONObject("battle").getInt("tick"));tap(vw-114,57f);assertEquals(6,state().getInt("screen"));assertEquals(seed,state().getJSONObject("battle").getLong("seed"))
   // Walk to the chest, the bell and Minuit using obstacle-aware joystick input.
   val before=state().getJSONObject("journey").getJSONArray("items").length()
   for(siteId in listOf(0,3,5)){
    val deadline=SystemClock.uptimeMillis()+35000;var opened=false;val trace=org.json.JSONArray()
    beginMove(0f,0f,true)
    while(SystemClock.uptimeMillis()<deadline&&state().getInt("screen")==6){
     val s=state();val b=s.getJSONObject("battle");val site=s.getJSONObject("expedition").getJSONArray("sites").getJSONObject(siteId);val target=site.getJSONObject("pos");val player=b.getJSONArray("fighters").getJSONObject(0);val p=player.getJSONObject("pos")
     if(hypot((target.getInt("x")-p.getInt("x")).toFloat(),(target.getInt("y")-p.getInt("y")).toFloat())<450){stop();shot("curiosity-$siteId");tap(vw/2,375f);opened=state().getJSONObject("expedition").getJSONArray("sites").getJSONObject(siteId).getBoolean("opened");if(opened)break;beginMove(0f,0f,true)}
     val(dx,dy)=route(b,target.getInt("x").toFloat() to target.getInt("y").toFloat());trace.put(JSONObject().put("pos",p).put("goal",target).put("dx",dx).put("dy",dy).put("tick",b.getInt("tick")));event(MotionEvent.ACTION_MOVE,listOf((100+dx*49) to (435+dy*49),(vw-91) to 326f));SystemClock.sleep(90)
     if(player.getInt("hp")<player.getInt("max_hp")/2){stop();tap(90f,164f);beginMove(dx,dy,true)}
    }
    stop();if(!opened){shot("route-failure");File(test.targetContext.getExternalFilesDir(null),"v05-route-failure.json").writeText(JSONObject().put("trace",trace).put("battle",state().optJSONObject("battle")).put("sites",state().optJSONObject("expedition")?.optJSONArray("sites")).toString())};assertTrue("Could not interact with site $siteId using movement and UI",opened)
   }
   assertTrue(state().getJSONObject("journey").getJSONArray("items").length()>=min(60,before+3))
   assertTrue(state().getJSONObject("journey").getJSONArray("secrets").length()>=2)
   tap(vw-190,43f);shot("loot-in-bag");tap(vw-275,56f);tap(vw-267,57f);tap(vw-382,498f);tap(vw-382,498f);shot("unlocked-memories")
   val metrics=metrics();assertTrue("Enemy action poses absent: $metrics",metrics.getInt("enemy_attack_animations")>0)
   File(test.targetContext.getExternalFilesDir(null),"v05-animation-acceptance.json").writeText(metrics.put("inventory_paused",true).put("resumed_same_seed",true).put("curiosities_opened",3).put("real_touch_input",true).toString(2))
   engine.uiAction("home");SystemClock.sleep(2200)
  }
 }
}
