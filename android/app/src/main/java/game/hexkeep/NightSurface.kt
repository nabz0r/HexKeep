package game.hexkeep

import android.app.Activity
import android.graphics.*
import android.os.SystemClock
import android.view.MotionEvent
import android.view.View
import game.hexkeep.core.Engine
import org.json.JSONArray
import org.json.JSONObject
import java.util.concurrent.Executors
import java.util.concurrent.TimeUnit
import java.util.concurrent.atomic.AtomicBoolean
import kotlin.math.*

/** High-resolution presentation. Simulation runs at 30 Hz independently of display refresh. */
class NightSurface(
    private val activity: Activity, private val engine: Engine,
    private val save: ()->Unit, private val action: (Int)->Unit,
    private val haptic: ()->Unit, private val secure: (Boolean)->Unit
): View(activity) {
    private val gold=Color.rgb(229,188,116)
    private val ivory=Color.rgb(238,234,220)
    private val muted=Color.rgb(151,174,179)
    private val teal=Color.rgb(105,205,195)
    private val red=Color.rgb(239,117,115)
    private val ink=Color.rgb(9,20,28)
    private val paint=Paint(Paint.ANTI_ALIAS_FLAG or Paint.FILTER_BITMAP_FLAG)
    private val serif=Typeface.create("serif",Typeface.NORMAL)
    private val sans=Typeface.create("sans-serif",Typeface.NORMAL)
    private val bold=Typeface.create("sans-serif-medium",Typeface.NORMAL)
    private val art=BitmapFactory.decodeStream(activity.assets.open("art/keep.png"))
    private val floor=BitmapFactory.decodeStream(activity.assets.open("art/courtyard.png"))
    private val atlas=BitmapFactory.decodeStream(activity.assets.open("art/characters.png"))
    private val ruins=BitmapFactory.decodeStream(activity.assets.open("art/ruins.png"))
    private val ruinRects=(0..2).map { column ->
        val width=ruins.width/3;val height=ruins.height;val pixels=IntArray(width*height);ruins.getPixels(pixels,0,width,column*width,0,width,height)
        var left=width;var top=height;var right=0;var bottom=0
        for(y in 0 until height)for(x in 0 until width)if((pixels[y*width+x] ushr 24)>40){left=min(left,x);right=max(right,x);top=min(top,y);bottom=max(bottom,y)}
        Rect(column*width+left.coerceAtMost(width-1),top.coerceAtMost(height-1),column*width+right+1,bottom+1)
    }
    private val impactUntil=HashMap<Int,Long>()
    private val worker=Executors.newSingleThreadScheduledExecutor { r -> Thread(r,"HEXKEEP simulation") }
    private val destroyed=AtomicBoolean(false)
    @Volatile private var active=true
    @Volatile private var logicalWidth=584
    @Volatile private var input=StickInput()
    private data class StickInput(val mx:Short=0,val my:Short=0,val ax:Short=0,val ay:Short=0)
    private val dashRequest=AtomicBoolean(false)
    private val skillRequest=AtomicBoolean(false)
    @Volatile private var autoAim=false
    @Volatile private var attackHeld=false
    private val attackRequest=AtomicBoolean(false)
    private var attackPointer=-1
    private var snapshot=JSONObject()
    private var previous=JSONObject()
    private var received=0L
    private var tickCount=0
    private var backdrop:Bitmap?=null
    private var backdropKey=""
    private var fog:Bitmap?=null
    private var fogKey=""
    private var viewportScale=2f
    private var viewportTop=0f
    private var vw=1170f
    private var intro=-1
    private var heroChoice=false
    private var worldMap=false
    private var inventoryPage=0
    private var chosenItem=0L
    private var firstChoice=false
    private var introStarted=0L
    private var screenStarted=0L
    private var seenScreen=-1
    private var movePointer=-1
    private var aimPointer=-1
    private var moveOrigin=PointF(100f,440f)
    private var aimOrigin=PointF()
    private var pointerButton=-1
    private val prefs=activity.getSharedPreferences("presentation-v3",0)
    private data class Button(val id:String,val rect:RectF,val run:()->Unit)
    private val buttons=ArrayList<Button>()
    private var focused:Button?=null
    private var frames=0L
    private var metricStart=0L
    private var slowFrames=0L
    private var lastFrame=0L
    private val recentHp=HashMap<Int,Int>()
    private data class Damage(val x:Float,val y:Float,val text:String,val born:Long)
    private val damage=ArrayList<Damage>()
    init {
        isFocusable=true;isFocusableInTouchMode=true
        autoAim=prefs.getBoolean("autoAim",false)
        contentDescription="HEXKEEP — Les Braises"
        worker.scheduleAtFixedRate({
            if(active&&!destroyed.get()) try {
                val v=input
                engine.controls(v.mx,v.my,v.ax,v.ay,autoAim||attackHeld||attackRequest.getAndSet(false),dashRequest.getAndSet(false),skillRequest.getAndSet(false))
                engine.tick((System.currentTimeMillis()/1000).toULong())
                val next=JSONObject(engine.presentation(logicalWidth))
                val sensitive=engine.sensitive()
                val a=engine.action().toInt();if(a!=0)action(a)
                if(engine.haptic().toInt()!=0)haptic()
                tickCount++;if(tickCount%60==0&&engine.dirty())save()
                post {
                    if(!destroyed.get()) { secure(sensitive);if(next.optInt("screen")!=snapshot.optInt("screen"))clearInput();previous=snapshot;snapshot=next;received=SystemClock.uptimeMillis() }
                }
            } catch(e:Exception) { android.util.Log.e("HEXKEEP","Presentation update failed",e) }
        },0,33_333_333,TimeUnit.NANOSECONDS)
    }
    fun onPause(){active=false;clearInput();if(snapshot.optInt("screen")==6&&!snapshot.optBoolean("online"))engine.uiAction("pause")}
    fun onResume(){active=true;invalidate()}
    fun close(){destroyed.set(true);active=false;worker.shutdownNow();clearInput()}
    fun back(){clearInput();when{snapshot.optInt("screen") in listOf(40,41)->command("home");intro>=0->{intro=-1;heroChoice=false};heroChoice->{heroChoice=false;if(firstChoice)intro=2 else engine.uiAction("home")};worldMap->{worldMap=false};snapshot.optInt("screen")==6->engine.uiAction("pause");snapshot.optInt("screen")==14->engine.uiAction("resume");snapshot.optInt("screen")==12->engine.uiAction("finish");snapshot.optInt("screen") in listOf(0,7)->activity.finish();else->engine.back()}}

    fun metrics():String=JSONObject().put("frames",frames).put("seconds",if(metricStart==0L)0.0 else (System.nanoTime()-metricStart)/1e9).put("over_33ms",slowFrames).put("renderer","Android hardware Canvas").toString()
    private fun clearInput(){input=StickInput();movePointer=-1;aimPointer=-1;attackPointer=-1;attackHeld=false;attackRequest.set(false);dashRequest.set(false);skillRequest.set(false);focused=null;engine.controls(0,0,0,0,false,false,false);engine.touch(0,3u,0,0)}
    private fun command(name:String){clearInput();engine.uiAction(name)}
    override fun onSizeChanged(w:Int,h:Int,oldw:Int,oldh:Int){viewportScale=min(h/540f,w/960f).coerceAtLeast(.1f);viewportTop=(h-540*viewportScale)/2;vw=w/viewportScale;logicalWidth=((vw/2).toInt()/8*8).coerceIn(400,640)}

    override fun onDraw(c:Canvas){
        super.onDraw(c)
        val now=System.nanoTime();if(metricStart==0L)metricStart=now;if(lastFrame!=0L&&now-lastFrame>33_333_333)slowFrames++;lastFrame=now;frames++
        if(viewportTop>0)c.drawColor(ink);c.save();c.translate(0f,viewportTop);c.scale(viewportScale,viewportScale);buttons.clear()
        val screen=snapshot.optInt("screen",0)
        if(screen!=seenScreen){seenScreen=screen;screenStarted=SystemClock.uptimeMillis();recentHp.clear();damage.clear()}
        when {
            snapshot.optBoolean("blocked")->legacy(c)
            intro>=0->introduction(c)
            heroChoice||screen==2||screen==3->heroes(c)
            screen==0->title(c)
            screen==7&&worldMap->map(c)
            screen==7->home(c)
            screen==6->battle(c)
            screen==14->{battle(c,false);pause(c)}
            screen==12->result(c)
            screen==10->settings(c)
            screen==20->fortress(c)
            screen==16->party(c)
            screen==40->inventory(c)
            screen==41->journal(c)
            else->legacy(c)
        }
        val message=snapshot.optString("message")
        if(message.isNotBlank()&&screen!=6){panel(c,vw/2-260,482f,520f,40f,0xe8152630.toInt());fitText(c,message,vw/2,507f,14f,ivory,490f,Paint.Align.CENTER)}
        c.restore()
        if(active){if(screen==6||screen==14)postInvalidateOnAnimation()else postInvalidateDelayed(50)}
    }
    private fun fill(c:Canvas,color:Int){paint.shader=null;paint.color=color;paint.style=Paint.Style.FILL;c.drawRect(0f,0f,vw,540f,paint)}
    private fun panel(c:Canvas,x:Float,y:Float,w:Float,h:Float,color:Int=0xde10212b.toInt(),stroke:Int=0x305da3ab){paint.shader=null;paint.style=Paint.Style.FILL;paint.color=color;c.drawRoundRect(x,y,x+w,y+h,if(h<30)4f else 12f,if(h<30)4f else 12f,paint);if(stroke!=0){paint.style=Paint.Style.STROKE;paint.strokeWidth=1f;paint.color=stroke;c.drawRoundRect(x,y,x+w,y+h,if(h<30)4f else 12f,if(h<30)4f else 12f,paint);paint.style=Paint.Style.FILL}}
    private fun line(c:Canvas,x:Float,y:Float,xx:Float,yy:Float,color:Int,width:Float=1f){paint.shader=null;paint.color=color;paint.strokeWidth=width;paint.style=Paint.Style.STROKE;c.drawLine(x,y,xx,yy,paint);paint.style=Paint.Style.FILL}
    private fun text(c:Canvas,s:String,x:Float,y:Float,size:Float=18f,color:Int=ivory,font:Typeface=sans,align:Paint.Align=Paint.Align.LEFT){paint.shader=null;paint.color=if(snapshot.optBoolean("accessible")&&color==muted)ivory else color;paint.typeface=font;paint.textSize=size;paint.textAlign=align;paint.style=Paint.Style.FILL;c.drawText(s,x,y,paint);paint.textAlign=Paint.Align.LEFT}
    private fun fitText(c:Canvas,s:String,x:Float,y:Float,size:Float,color:Int,w:Float,align:Paint.Align=Paint.Align.LEFT){paint.typeface=sans;paint.textSize=size;val measure=paint.measureText(s).coerceAtLeast(1f);text(c,s,x,y,min(size,size*w/measure),color,sans,align)}
    private fun paragraph(c:Canvas,s:String,x:Float,y:Float,w:Float,size:Float=19f,color:Int=muted,lineHeight:Float=29f){paint.typeface=sans;paint.textSize=size;var row="";var yy=y;s.split(' ').forEach{word->val next=if(row.isEmpty())word else "$row $word";if(paint.measureText(next)>w&&row.isNotEmpty()){text(c,row,x,yy,size,color);yy+=lineHeight;row=word}else row=next};if(row.isNotEmpty())text(c,row,x,yy,size,color)}
    private fun button(c:Canvas,id:String,label:String,x:Float,y:Float,w:Float,h:Float=54f,primary:Boolean=false,run:()->Unit){val down=focused?.id==id;panel(c,x,y,w,h,if(primary){if(down)0xfff2d193.toInt()else gold}else{if(down)0xff284551.toInt()else 0xe0142833.toInt()},if(primary)0 else 0x60789994);fitText(c,label,x+w/2,y+h/2+6,17f,if(primary)ink else ivory,w-24,Paint.Align.CENTER);buttons.add(Button(id,RectF(x,y,x+w,y+h),run))}
    private fun background(c:Canvas,dim:Float=0.2f){
        // Cache the static composition; only embers are redrawn above it each frame.
        val key="$vw:$dim"
        if(backdropKey!=key){
            val density=min(1.6f,1920f/vw);val bmp=Bitmap.createBitmap((vw*density).toInt(),(540*density).toInt(),Bitmap.Config.ARGB_8888);val target=Canvas(bmp);target.scale(density,density)
            val ratio=max(vw/art.width,540f/art.height);val w=art.width*ratio;val h=art.height*ratio
            paint.color=Color.WHITE;paint.alpha=255;paint.shader=null;target.drawBitmap(art,null,RectF((vw-w)/2,0f,(vw+w)/2,h),paint)
            paint.shader=LinearGradient(0f,0f,vw*.82f,0f,intArrayOf(0xf408131b.toInt(),0x9308131b.toInt(),0x0008131b),null,Shader.TileMode.CLAMP);target.drawRect(0f,0f,vw,540f,paint);paint.shader=null
            paint.color=(dim*255).toInt().coerceIn(0,255)shl 24;target.drawRect(0f,0f,vw,540f,paint);backdrop=bmp;backdropKey=key
        }
        paint.shader=null;paint.color=Color.WHITE;paint.alpha=255;backdrop?.let{c.drawBitmap(it,null,RectF(0f,0f,vw,540f),paint)};embers(c)
    }

    private fun embers(c:Canvas){val t=SystemClock.uptimeMillis()/1000f;for(i in 0..27){val x=((i*173.3f+sin(t*.2+i)*24)%vw).toFloat();val y=540f-((t*(7+i%7)+i*51)%540);paint.color=Color.argb(65+(i%4)*25,235,175,86);c.drawCircle(x,y,if(i%5==0)1.6f else .8f,paint)}}
    private fun ornament(c:Canvas,x:Float,y:Float){line(c,x,y,x+45,y,gold);val p=Path();p.moveTo(x+56,y-5);p.lineTo(x+61,y);p.lineTo(x+56,y+5);p.lineTo(x+51,y);p.close();paint.color=gold;c.drawPath(p,paint);line(c,x+67,y,x+112,y,gold)}
    private fun title(c:Canvas){background(c,0f);text(c,"LES BRAISES",64f,91f,14f,gold,bold);ornament(c,64f,112f);text(c,"HEXKEEP",58f,208f,76f,ivory,serif);paragraph(c,"Là où personne ne veille, le monde s’éteint.",66f,252f,450f,22f,ivory,30f);paragraph(c,"Porte la lumière. Réveille les forteresses. Laisse une trace dans la nuit.",66f,308f,370f,18f,muted,26f)
        button(c,"begin",if(snapshot.optBoolean("intro_seen"))"Reprendre la veille  ›"else"Entrer dans la nuit  ›",64f,382f,310f,58f,true){if(snapshot.optBoolean("intro_seen"))command("continue")else beginIntro()}
        button(c,"settings","Réglages",394f,382f,144f,58f){command("settings")}
        text(c,"CHAPITRE I  /  LA DERNIÈRE LANTERNE",66f,480f,12f,muted,bold);text(c,"0.4 • LES CHEMINS DE BRAISE",vw-44,506f,11f,muted,sans,Paint.Align.RIGHT)
    }
    private fun beginIntro(){intro=0;introStarted=SystemClock.uptimeMillis();heroChoice=false;worldMap=false;clearInput()}
    private fun introduction(c:Canvas){background(c,.16f);val elapsed=(SystemClock.uptimeMillis()-introStarted)/1000f;val fade=(elapsed/1.2f).coerceIn(0f,1f);val titles=arrayOf("Le monde oublie.","Une lumière demeure.","À toi de veiller.");val body=arrayOf("Les routes ont disparu sous la brume. Les noms se sont effacés des pierres. Une à une, les forteresses ont cessé de répondre.","Au cœur des ruines, une lanterne brûle encore. Elle n’attend ni roi, ni armée. Seulement quelqu’un pour la porter.","Rallume les trois balises. Apprends les mouvements des ombres. Puis affronte le gardien de la dernière porte.");text(c,"PROLOGUE   /   0${intro+1}",68f,123f,13f,gold,bold);text(c,titles[intro],64f,216f,46f,Color.argb((255*fade).toInt(),238,234,220),serif);paragraph(c,body[intro],68f,275f,min(530f,vw*.55f),22f,ivory,34f);for(i in 0..2)line(c,68f+i*51,420f,101f+i*51,420f,if(i<=intro)gold else 0xff3e5054.toInt(),3f)
        button(c,"intro-next",if(intro==2)"Choisir mon veilleur  ›"else"Continuer  ›",vw-316,410f,248f,58f,true){if(intro<2){intro++;introStarted=SystemClock.uptimeMillis()}else{intro=-1;heroChoice=true;firstChoice=true}}
        button(c,"intro-skip","Passer",vw-164,38f,112f,44f){intro=-1;heroChoice=true;firstChoice=true}
    }
    private fun portrait(c:Canvas,index:Int,x:Float,y:Float,w:Float,h:Float,alpha:Int=255,flip:Boolean=false){val cell=atlas.width/3;val row=atlas.height/2;val src=Rect(index%3*cell,index/3*row,(index%3+1)*cell,(index/3+1)*row);paint.shader=null;paint.alpha=alpha;paint.color=Color.WHITE;c.save();if(flip)c.scale(-1f,1f,x+w/2,y+h/2);c.drawBitmap(atlas,src,RectF(x,y,x+w,y+h),paint);c.restore();paint.alpha=255}
    private fun heroes(c:Canvas){background(c,.57f);text(c,"CHOISIS TA LUMIÈRE",48f,52f,13f,gold,bold);text(c,"Trois serments. Une même nuit.",46f,98f,32f,ivory,serif);val gap=18f;val cw=(vw-96-gap*2)/3;val names=arrayOf("Aurelon","Skarn","Vylde");val details=arrayOf("L’aube • équilibre et précision","Le givre • endurance et impact","La sève • mobilité et entraide");val chosen=snapshot.optInt("realm");for(i in 0..2){val x=48+i*(cw+gap);panel(c,x,126f,cw,274f,if(i==chosen)0xde253a40.toInt()else 0xb50e212c.toInt(),if(i==chosen)gold else 0x40567479);portrait(c,i,x+cw/2-104,125f,208f,208f);text(c,names[i],x+cw/2,352f,28f,if(i==chosen)gold else ivory,serif,Paint.Align.CENTER);fitText(c,details[i],x+cw/2,381f,14f,muted,cw-26,Paint.Align.CENTER);buttons.add(Button("hero$i",RectF(x,126f,x+cw,400f)){engine.hero(i.toUByte(),snapshot.optInt("role").toUByte())})}
        val roles=arrayOf("Foudre · impulsion","Rempart · rempart de pierre","Lien · soin de proximité");val role=snapshot.optInt("role");button(c,"role-cycle",roles[role],48f,421f,min(390f,vw*.43f),52f){engine.hero(chosen.toUByte(),((role+1)%3).toUByte())};button(c,"hero-ready",if(firstChoice)"Allumer ma lanterne  ›"else"Reprendre la veille  ›",vw-368,421f,320f,52f,true){heroChoice=false;if(firstChoice){firstChoice=false;command("prologue")}else command("home")};fitText(c,"Ton Nom reste sur cet appareil. Note tes 24 mots dans les réglages pour le retrouver ailleurs.",48f,507f,14f,muted,vw-96)
    }
    private fun home(c:Canvas){
        background(c,.27f);val j=snapshot.optJSONObject("journey")?:JSONObject()
        text(c,"HEXKEEP",47f,65f,30f,ivory,serif);text(c,"LES CHEMINS DE BRAISE  /  0.4",49f,90f,11f,gold,bold)
        button(c,"gear","Réglages",vw-191,36f,143f,45f){command("settings")}
        fitText(c,"${snapshot.optString("name")}  •  Niveau ${j.optInt("level",1)}",49f,160f,16f,teal,440f)
        text(c,"Retrouve les chemins.",45f,214f,39f,ivory,serif)
        paragraph(c,"Éline t’attend au refuge. Explore les marches, retrouve leurs mémoires et rapporte de quoi tenir une nuit de plus.",49f,254f,min(420f,vw*.44f),20f,ivory,29f)
        button(c,"expedition","Choisir une aventure  ›",48f,367f,355f,62f,true){worldMap=false;command("journal")}
        button(c,"map","Explorer la marche",48f,448f,216f,52f){worldMap=true}
        button(c,"inventory","Sac & équipement",278f,448f,218f,52f){command("inventory")}
        val x=vw-374;panel(c,x,132f,326f,300f,0xe00d222c.toInt());text(c,"TON VEILLEUR",x+24,167f,12f,gold,bold)
        text(c,"${j.optInt("discovered")} marches découvertes",x+24,206f,23f,ivory,serif)
        text(c,"${snapshot.optInt("xp")} éclats  •  ${j.optInt("dust")} poussières",x+24,235f,15f,muted)
        button(c,"hero","Identité & serment",x+20,262f,286f,45f){firstChoice=false;heroChoice=true}
        button(c,"network","Jouer avec des veilleurs  ›",x+20,318f,286f,45f){command("network")}
        button(c,"campaign","Forteresse & chroniques",x+20,374f,286f,43f){command("campaign")}
        text(c,"${snapshot.optString("realm_name")}  •  ${snapshot.optString("role_name")}",x+3,477f,14f,muted)
    }
    private fun rarity(n:Int)=intArrayOf(muted,teal,0xff8fb4ff.toInt(),0xffd29ffa.toInt())[n.coerceIn(0,3)]
    private fun itemIcon(c:Canvas,slot:Int,x:Float,y:Float,color:Int){
        glow(c,x,y,27f,color,35);val p=Path()
        when(slot){0->{line(c,x-13,y+14,x+13,y-14,color,4f);line(c,x-12,y+4,x-3,y+13,color,3f);circle(c,x+14,y-15,3f,color)}
            1->{p.moveTo(x,y-19);p.lineTo(x+16,y-11);p.lineTo(x+13,y+8);p.lineTo(x,y+20);p.lineTo(x-13,y+8);p.lineTo(x-16,y-11);p.close();paint.color=color;paint.style=Paint.Style.STROKE;paint.strokeWidth=2f;c.drawPath(p,paint);paint.style=Paint.Style.FILL;line(c,x,y-12,x,y+12,color,2f)}
            else->{circle(c,x,y,16f,color,2f);line(c,x-7,y+7,x+7,y-7,color,3f);circle(c,x,y,3f,color)}}
    }
    private fun inventory(c:Canvas){
        background(c,.73f);val j=snapshot.optJSONObject("journey")?:JSONObject();val unsorted=j.optJSONArray("items")?:JSONArray();val items=JSONArray((0 until unsorted.length()).map{unsorted.getJSONObject(it)}.sortedWith(compareByDescending<JSONObject>{it.optInt("rarity")}.thenByDescending{it.optLong("id")}));val equipped=j.optJSONArray("equipped")?:JSONArray()
        header(c,"Le sac du veilleur","Équipement • statistiques • forge"){command("home")}
        val lw=vw-397;val cw=(lw-15)/2;val pages=max(1,(items.length()+5)/6);inventoryPage=inventoryPage.coerceIn(0,pages-1)
        if(chosenItem==0L&&items.length()>0)chosenItem=items.getJSONObject(0).optLong("id")
        for(k in 0..5){val i=inventoryPage*6+k;if(i>=items.length())break;val it=items.getJSONObject(i);val id=it.optLong("id");val slot=it.optInt("slot");val x=48+(k%2)*(cw+15);val y=125+(k/2)*105f;val color=rarity(it.optInt("rarity"));val worn=equipped.optLong(slot)==id
            panel(c,x,y,cw,91f,if(chosenItem==id)0xe52b414e.toInt()else 0xe0132935.toInt(),if(chosenItem==id)gold else color);itemIcon(c,slot,x+35,y+40,color)
            fitText(c,it.optString("name"),x+68,y+29,17f,ivory,cw-80);text(c,arrayOf("Commun","Inhabituel","Rare","Épique")[it.optInt("rarity").coerceIn(0,3)]+if(worn)"  •  ÉQUIPÉ"else "",x+68,y+51,11f,color,bold)
            fitText(c,"+${it.optInt("vitality")} vie  +${it.optInt("power")} puissance",x+68,y+73,12f,muted,cw-80);buttons.add(Button("item$id",RectF(x,y,x+cw,y+91)){chosenItem=id})
        }
        button(c,"prev","‹",48f,456f,51f,46f){inventoryPage=(inventoryPage-1).coerceAtLeast(0)};text(c,"${inventoryPage+1} / $pages  •  ${items.length()} / 60 objets",114f,485f,13f,muted)
        button(c,"next","›",48+lw-51,456f,51f,46f){inventoryPage=(inventoryPage+1).coerceAtMost(pages-1)}
        val x=vw-322;panel(c,x,124f,274f,382f);text(c,"NIVEAU ${j.optInt("level",1)}",x+21,155f,12f,gold,bold)
        text(c,"${j.optInt("vitality")} vie   /   ${j.optInt("armor")} armure",x+21,185f,16f,ivory)
        text(c,"${j.optInt("power")}% puissance",x+21,212f,16f,ivory);text(c,"${j.optInt("haste")} cadence  •  Bonus en expédition",x+21,237f,12f,muted)
        val selected=(0 until items.length()).map{items.getJSONObject(it)}.find{it.optLong("id")==chosenItem}
        if(selected!=null){val slot=selected.optInt("slot");val worn=equipped.optLong(slot)==chosenItem
            val current=(0 until items.length()).map{items.getJSONObject(it)}.find{it.optLong("id")==equipped.optLong(slot)}
            fitText(c,selected.optString("name"),x+21,278f,18f,rarity(selected.optInt("rarity")),230f)
            val dh=selected.optInt("vitality")-(current?.optInt("vitality")?:0);val dp=selected.optInt("power")-(current?.optInt("power")?:0)
            text(c,"Écart : ${if(dh>=0)"+"else ""}$dh vie  /  ${if(dp>=0)"+"else ""}$dp puissance",x+21,303f,13f,muted)
            button(c,"equip",if(worn)"Équipé"else "Équiper",x+20,322f,234f,44f,!worn){command("equip:$chosenItem")}
            if(!worn)button(c,"salvage","Recycler · +${4*(selected.optInt("rarity")+1)} poussières",x+20,376f,234f,41f){command("salvage:$chosenItem");chosenItem=0}
        }
        button(c,"forge","Forge  30 ◆   /   ${j.optInt("dust")} ◆",x+20,449f,234f,41f){command("forge")}
        text(c,"Les combats entre joueurs utilisent les valeurs communes du Codex.",48f,527f,12f,muted)
    }
    private fun journal(c:Canvas){
        background(c,.68f);val j=snapshot.optJSONObject("journey")?:JSONObject();header(c,j.optString("region","Les marches"),"Le carnet d’Éline"){command("home")}
        paragraph(c,j.optString("story"),48f,132f,vw-96,17f,ivory,25f)
        val events=j.optJSONArray("events")?:JSONArray();val cw=(vw-128)/3
        val bodies=arrayOf("Rallume trois feux et défie le gardien. Chaque feu te rend de la vie.","Traque six ombres. Lis leurs attaques, esquive et révèle leur chef.","Explore les ruines. Retrouve trois mémoires avant d’ouvrir la dernière porte.")
        for(i in 0..2){val e=events.optJSONObject(i)?:JSONObject();val x=48+i*(cw+16);panel(c,x,191f,cw,247f,0xe0112936.toInt(),if(e.optBoolean("done"))teal else 0x60557d85);text(c,"0${i+1}  /  ${if(e.optBoolean("done"))"ACCOMPLI"else "DISPONIBLE"}",x+22,221f,11f,if(e.optBoolean("done"))teal else gold,bold);fitText(c,e.optString("name"),x+22,255f,24f,ivory,cw-44);paragraph(c,bodies[i],x+22,288f,cw-44,16f,muted,23f);text(c,if(e.optBoolean("done"))"Butin inhabituel en rejouant"else "Butin rare • +60 éclats",x+22,372f,13f,teal);button(c,"contract$i","Partir  ›",x+18,387f,cw-36,39f,true){command("contract:$i")}}
        text(c,"3 caches à découvrir dans chaque sortie. Ramasse les braises laissées par les ombres.",48f,474f,14f,muted)
        val remaining=j.optInt("remaining");text(c,"Événements locaux • Renouveau dans ${remaining/60} min ${remaining%60} s",48f,508f,13f,gold)
        button(c,"journal-map","Voir la carte",vw-241,469f,193f,43f){command("home");worldMap=true}
    }
    private fun header(c:Canvas,title:String,subtitle:String,back:()->Unit){text(c,subtitle.uppercase(),48f,48f,12f,gold,bold);text(c,title,46f,91f,34f,ivory,serif);button(c,"back","‹  Retour",vw-183,38f,135f,44f,run=back)}
    private fun map(c:Canvas){background(c,.68f);header(c,"Les marches oubliées","Territoire autour du refuge"){worldMap=false};val cells=snapshot.optJSONArray("cells")?:JSONArray();val radius=33f;val cx=vw*.40f;val cy=303f;for(i in 0 until cells.length()){val cell=cells.getJSONObject(i);val q=cell.optInt("q");val r=cell.optInt("r");val x=cx+sqrt(3f)*radius*.5f*(q+r);val y=cy+1.5f*radius*(q-r);val p=Path();for(j in 0..5){val a=(j*60-30)*PI/180;val xx=x+cos(a).toFloat()*(radius-2);val yy=y+sin(a).toFloat()*(radius-2);if(j==0)p.moveTo(xx,yy)else p.lineTo(xx,yy)};p.close();paint.color=if(cell.optBoolean("current"))0xff947846.toInt()else if(cell.optBoolean("clear"))0xff294d51.toInt()else 0xff152c37.toInt();c.drawPath(p,paint);paint.style=Paint.Style.STROKE;paint.strokeWidth=1f;paint.color=if(cell.optBoolean("selected"))gold else 0xff40616a.toInt();c.drawPath(p,paint);paint.style=Paint.Style.FILL;if(cell.optBoolean("current")){glow(c,x,y,35f,gold,80);text(c,"◆",x,y+8,22f,ivory,sans,Paint.Align.CENTER)}else if(!cell.isNull("bastion"))text(c,"◇",x,y+7,22f,teal,sans,Paint.Align.CENTER)else text(c,if(cell.optBoolean("discovered"))arrayOf("✦","⌖","◇")[cell.optInt("poi")]else "?",x,y+6,18f,if(cell.optBoolean("discovered"))teal else muted,sans,Paint.Align.CENTER);buttons.add(Button("cell$i",RectF(x-radius*.8f,y-radius*.8f,x+radius*.8f,y+radius*.8f)){command("cell:"+cell.optString("id"))})}
        val x=vw-350;panel(c,x,139f,302f,326f);fitText(c,(0 until cells.length()).map{cells.getJSONObject(it)}.find{it.optBoolean("selected")}?.optString("region")?:"Les marches",x+23,180f,23f,ivory,256f);text(c,if(snapshot.optBoolean("selected_current"))"Ici se tient ton veilleur."else"Une marche voisine t’appelle.",x+23,211f,15f,muted)
        button(c,"map-expedition","Événements de ma marche  ›",x+20,232f,262f,46f,true){worldMap=false;command("journal")}
        button(c,"map-found","Fonder",x+20,291f,124f,43f){if(snapshot.optBoolean("selected_current"))command("found")else engine.notice("Rejoins cette marche avant d’y fonder un bastion.")};button(c,"map-banner",if(snapshot.optBoolean("banner"))"Bannière ●"else"Bannière ○",x+154,291f,128f,43f){command("banner")}
        button(c,"map-walk",if(snapshot.optBoolean("selected_current"))"Utiliser ma position GPS"else"Voyager ici · simulation",x+20,347f,262f,43f){command(if(snapshot.optBoolean("selected_current"))"gps"else"walk")}
        button(c,"map-campaign","Forteresse & chroniques",x+20,403f,262f,43f){worldMap=false;command("campaign")};text(c,if(snapshot.optBoolean("gps"))"Position issue du GPS • Reprends ta marche pour découvrir les lieux voisins."else "Exploration simulée • Active le GPS pour découvrir les marches autour de toi.",48f,508f,13f,muted)
    }

    private fun party(c:Canvas){
        background(c,.72f);header(c,"Les lanternes voisines","Jouer ensemble • réseau de développement"){command("home")}
        val cw=(vw-112)/2;panel(c,48f,125f,cw,318f);text(c,"Rassemble les veilleurs",71f,162f,26f,ivory,serif)
        paragraph(c,"Même version, même marche et même Wi-Fi. Lance la recherche sur chaque appareil, puis ouvre un champ ou propose un duel.",71f,198f,cw-46,17f,muted,26f)
        button(c,"connect",if(snapshot.optBoolean("searching"))"Recherche active…"else "Rechercher des joueurs",68f,293f,cw-40,47f,true){command("connect")}
        button(c,"address","Connexion directe / relais",68f,355f,cw-40,47f){command("address")}
        val x=64+cw;panel(c,x,125f,cw,318f);val peers=snapshot.optJSONArray("party")?:JSONArray();text(c,"${peers.length()} veilleur(s) découvert(s)",x+23,164f,23f,ivory,serif)
        if(peers.length()==0)paragraph(c,"Aucun autre joueur pour le moment. Le refuge et les aventures restent jouables hors ligne.",x+23,211f,cw-46,18f,muted,28f)
        for(i in 0 until min(peers.length(),5)){val p=peers.getJSONObject(i);fitText(c,"◆  ${p.optString("name")}  •  ${p.optString("realm")}",x+23,204f+i*29,16f,teal,cw-46)}
        button(c,"duel","Duel",x+20,355f,(cw-50)/2,47f){command("duel")};button(c,"field","Champ · jusqu’à 10",x+30+(cw-50)/2,355f,(cw-50)/2,47f){command("field")}
        text(c,"${snapshot.optInt("latency")} ms  •  Équipement normalisé entre joueurs  •  Pas de serveur public permanent",48f,491f,13f,muted)
    }

    private fun fortress(c:Canvas){
        background(c,.61f);header(c,"La forteresse","Ce que nous tenons allumé"){command("home")}
        val titles=arrayOf("Sièges","Mémoires","La saison","L’atelier","Les Maisons","Le Phare","Le Trône · DEV","Mon Nom")
        val descriptions=arrayOf("Brise la Porte. Tiens la Cour.","Porte les traces du monde.","Éclats, serments et Nuit longue.","Apparences et lanternes.","Dessine l’emblème des tiens.","Veille avec les autres joueurs.","Règles, Sceaux et chroniques.","Identité et mots de récupération.")
        val cw=(vw-112)/2
        for(i in 0..7){val row=i/2;val col=i%2;val x=48+col*(cw+16);val y=119+row*88f
            panel(c,x,y,cw,75f,0xe0122b36.toInt(),0x50557d85);text(c,"0${i+1}",x+19,y+47,26f,if(i==6)gold else teal,serif);text(c,titles[i],x+72,y+31,20f,ivory,serif);fitText(c,descriptions[i],x+72,y+56,14f,muted,cw-120);text(c,"›",x+cw-24,y+47,26f,gold,serif,Paint.Align.CENTER)
            buttons.add(Button("fortress$i",RectF(x,y,x+cw,y+75)){
                clearInput();val xx=if(col==0)logicalWidth/4 else logicalWidth*3/4;engine.touch(70,0u,xx,62+row*45);engine.touch(70,2u,xx,62+row*45)
            })
        }
        text(c,"Les histoires se construisent avec les autres veilleurs.",48f,508f,14f,muted)
    }
    private fun settings(c:Canvas){background(c,.7f);header(c,"À ton rythme","Ambiance et commandes"){command(if(snapshot.optBoolean("created"))"home"else"continue")};val x=48f;val w=(vw-120)/2;val yy=132f;val toggles=listOf(Triple("music","Musique d’ambiance",snapshot.optBoolean("music")),Triple("effects","Sons de combat",snapshot.optBoolean("effects")),Triple("haptics","Vibrations",snapshot.optBoolean("haptics")));toggles.forEachIndexed{i,t->button(c,t.first,"${t.second}   ${if(t.third)"●"else"○"}",x,yy+i*76,w,60f){command(t.first)}};button(c,"aim","Attaque automatique   ${if(autoAim)"●"else"○"}",x,360f,w,60f){autoAim=!autoAim;prefs.edit().putBoolean("autoAim",autoAim).apply()};val xx=x+w+24;panel(c,xx,132f,w,288f);text(c,"Prends la nuit en main.",xx+24,176f,26f,ivory,serif);paragraph(c,"À gauche, déplace-toi. À droite, vise en glissant. Maintiens ATTAQUE pour viser une ombre visible, ou glisse depuis ce bouton pour viser toi-même. Esquive et pouvoir restent accessibles pendant le déplacement.",xx+24,216f,w-48,18f,muted,28f);button(c,"intro","Revoir le prologue",48f,446f,225f,49f){beginIntro()};button(c,"identity","Mon Nom & sauvegarde",289f,446f,259f,49f){command("identity")};button(c,"contrast","Contraste renforcé   ${if(snapshot.optBoolean("accessible"))"●"else"○"}",vw-348,446f,300f,49f){command("contrast")}}
    private fun glow(c:Canvas,x:Float,y:Float,r:Float,color:Int,alpha:Int=110){paint.shader=RadialGradient(x,y,r,Color.argb(alpha,Color.red(color),Color.green(color),Color.blue(color)),Color.TRANSPARENT,Shader.TileMode.CLAMP);c.drawCircle(x,y,r,paint);paint.shader=null}
    private fun circle(c:Canvas,x:Float,y:Float,r:Float,color:Int,stroke:Float=0f){paint.shader=null;paint.color=color;paint.style=if(stroke>0)Paint.Style.STROKE else Paint.Style.FILL;paint.strokeWidth=stroke;c.drawCircle(x,y,r,paint);paint.style=Paint.Style.FILL}
    private fun battle(c:Canvas,interactive:Boolean=true){
        val b=snapshot.optJSONObject("battle")?:return
        fill(c,ink) // The camera floor already fills the playfield; avoid a second full-screen texture.
        val tile=48f;val focus=b.optJSONArray("fighters")?.optJSONObject(snapshot.optInt("local"))?.optJSONObject("pos")?:JSONObject()
        val focusOld=previous.optJSONObject("battle")?.optJSONArray("fighters")?.optJSONObject(snapshot.optInt("local"))?.optJSONObject("pos")?:focus
        val blend=((SystemClock.uptimeMillis()-received)/33.333f).coerceIn(0f,1f)
        val fx=focusOld.optInt("x")+(focus.optInt("x")-focusOld.optInt("x"))*blend;val fy=focusOld.optInt("y")+(focus.optInt("y")-focusOld.optInt("y"))*blend
        val aw=tile*30;val ah=tile*16
        val left=if(aw<vw)(vw-aw)/2 else (vw*.5f-fx/256f*tile).coerceIn(vw-aw-18,18f)
        val top=(278f-fy/256f*tile).coerceIn(540-ah,80f)
        c.save();c.clipRect(0f,76f,vw,540f)
        paint.color=Color.WHITE;paint.shader=null;val dest=RectF(left-tile,top-tile,left+aw+tile,top+ah+tile)
        val region=(b.optLong("seed")%3).toInt();paint.colorFilter=LightingColorFilter(when(region){1->0xff93b2a2.toInt();2->0xffac96b7.toInt();else->0xffa7b0b9.toInt()},0);c.drawBitmap(floor,null,dest,paint);paint.colorFilter=null
        val obs=b.optJSONArray("obstacles")?:JSONArray()
        for(i in 0 until obs.length()){val o=obs.getJSONObject(i);val x=left+o.optInt("x")/256f*tile;val y=top+o.optInt("y")/256f*tile;val w=o.optInt("w")/256f*tile;val h=o.optInt("h")/256f*tile;val kind=o.optInt("kind");paint.color=0x70000000;c.drawOval(x-3,y+h*.55f,x+w+3,y+h+6,paint);paint.color=Color.WHITE;c.drawBitmap(ruins,ruinRects[kind.coerceIn(0,2)],RectF(x,y-8,x+w,y+h),paint)}

        val siege=b.optJSONObject("siege")
        if(siege!=null){val gx=left+20*tile;val gy=top+8*tile;val cell=ruins.width/3;paint.color=Color.WHITE;if(siege.optInt("gate_hp")>0){c.drawBitmap(ruins,Rect(0,0,cell,ruins.height),RectF(gx-40,gy-78,gx+40,gy+32),paint);glow(c,gx,gy-15,45f,gold,50)};circle(c,left+24*tile,gy,2*tile,gold,2f);text(c,if(siege.optInt("gate_hp")>0)"PORTE"else"TENIR LA COUR",left+24*tile,gy+6,11f,gold,bold,Paint.Align.CENTER)}
        val run=snapshot.optJSONObject("expedition");val charges=run?.optJSONArray("charges");val points=arrayOf(6 to 4,15 to 12,25 to 5)
        if(charges!=null)for(i in 0..2){val x=left+points[i].first*tile;val y=top+points[i].second*tile;val charge=charges.optInt(i);glow(c,x,y,42f,if(charge==90)gold else teal,if(charge==90)105 else 35);circle(c,x,y,1.5f*tile,if(charge==90)0x90d5b474.toInt()else 0x70549f9e,1.3f);if(charge>0){paint.color=gold;paint.style=Paint.Style.STROKE;paint.strokeWidth=3f;c.drawArc(x-1.5f*tile,y-1.5f*tile,x+1.5f*tile,y+1.5f*tile,-90f,charge*4f,false,paint);paint.style=Paint.Style.FILL};line(c,x,y,x,y-28,0xff6c7471.toInt(),5f);glow(c,x,y-28,22f,if(charge==90)gold else teal,150);text(c,if(charge==90)"◆"else"◇",x,y-22,21f,if(charge==90)gold else teal,sans,Paint.Align.CENTER);text(c,"${i+1}",x,y+18,11f,ivory,sans,Paint.Align.CENTER)}
        if(run!=null){val cache=run.optJSONArray("caches")?:JSONArray();val cachePoints=arrayOf(3 to 13,16 to 2,27 to 12)
            for(i in 0..2){if(cache.optBoolean(i))continue;val x=left+cachePoints[i].first*tile;val y=top+cachePoints[i].second*tile;glow(c,x,y,36f,gold,70);panel(c,x-15,y-13,30f,24f,0xff725139.toInt(),gold);line(c,x-15,y-4,x+15,y-4,gold,2f);text(c,"CACHE",x,y+28,10f,gold,bold,Paint.Align.CENTER)}
            val drops=run.optJSONArray("drops")?:JSONArray();for(i in 0 until drops.length()){val d=drops.getJSONObject(i);if(d.optBoolean("collected"))continue;val p=d.getJSONObject("pos");val x=left+p.optInt("x")/256f*tile;val y=top+p.optInt("y")/256f*tile;glow(c,x,y,20f,gold,90);text(c,"◆",x,y+5,18f,gold,sans,Paint.Align.CENTER)}
        }
        val pickups=b.optJSONArray("pickups")?:JSONArray();for(i in 0 until pickups.length()){val p=pickups.getJSONObject(i);if(p.optInt("next")>b.optInt("tick"))continue;val pos=p.getJSONObject("pos");val x=left+pos.optInt("x")/256f*tile;val y=top+pos.optInt("y")/256f*tile;val col=if(p.optInt("kind")==0)teal else gold;glow(c,x,y,23f,col,65);text(c,if(p.optInt("kind")==0)"+"else"◇",x,y+5,22f,col,bold,Paint.Align.CENTER)}
        val effects=b.optJSONArray("effects")?:JSONArray();for(i in 0 until effects.length()){val e=effects.getJSONObject(i);val pos=e.getJSONObject("pos");val x=left+pos.optInt("x")/256f*tile;val y=top+pos.optInt("y")/256f*tile;val r=e.optInt("radius")/256f*tile;glow(c,x,y,max(10f,r),teal,(e.optInt("life")*5).coerceIn(0,100));circle(c,x,y,max(8f,r),0x9052b9bc.toInt(),1.5f)}
        val fighters=b.optJSONArray("fighters")?:JSONArray();val local=snapshot.optInt("local");val alpha=((SystemClock.uptimeMillis()-received)/33.333f).coerceIn(0f,1f);val prev=previous.optJSONObject("battle")?.optJSONArray("fighters");val tick=b.optInt("tick")
        val ordered=(0 until fighters.length()).sortedBy{fighters.getJSONObject(it).getJSONObject("pos").optInt("y")}
        for(i in ordered){val f=fighters.getJSONObject(i);val id=f.optInt("id");val hp=f.optInt("hp");val pos=f.getJSONObject("pos");val old=prev?.optJSONObject(i)?.optJSONObject("pos");val px=pos.optInt("x").toFloat();val py=pos.optInt("y").toFloat();val ox=old?.optInt("x")?.toFloat()?:px;val oy=old?.optInt("y")?.toFloat()?:py;val interp=abs(px-ox)+abs(py-oy)<600;val x=left+(if(interp)ox+(px-ox)*alpha else px)/256f*tile;val y=top+(if(interp)oy+(py-oy)*alpha else py)/256f*tile
            val before=recentHp.put(id,hp);if(before!=null&&hp<before&&hp>0){damage.add(Damage(x,y-50,"−${before-hp}",SystemClock.uptimeMillis()));impactUntil[id]=SystemClock.uptimeMillis()+110}
            if(hp<=0){if(id==local){glow(c,x,y,35f,teal,70);text(c,"Reviens à la lumière…",x,y,14f,ivory,sans,Paint.Align.CENTER)};continue}
            val mine=id==local;val role=f.optString("role");val enemy=f.optBoolean("bot");val boss=run?.optInt("wave")==1&&id!=0;val color=if(mine)teal else if(enemy)red else gold
            val phase=(tick+id*17)%120
            if(enemy&&role=="Rempart"&&phase in 60..99){val aim=f.optJSONObject("aim")?:JSONObject();val angle=atan2(aim.optInt("y").toFloat(),aim.optInt("x").toFloat());val cone=Path();cone.moveTo(x,y);for(a in listOf(angle-.4f,angle+.4f)){cone.lineTo(x+cos(a)*tile*4.3f,y+sin(a)*tile*4.3f)};cone.close();paint.color=0x359f4f43;c.drawPath(cone,paint)}
            if(enemy&&((role=="Rempart"&&phase in 60..99)||(role=="Foudre"&&phase in 72..89)||(role=="Lien"&&phase in 58..74))){circle(c,x,y,24f+phase%20*.5f,0x909b4e44.toInt(),2f);text(c,"!",x,y-68,21f,red,bold,Paint.Align.CENTER)}
            paint.color=0x80000000.toInt();c.drawOval(x-19,y-5,x+19,y+9,paint);circle(c,x,y,if(mine)18f else 14f,if(mine)0x9073d1c8.toInt()else 0x707c4950,1.5f)
            if(f.optInt("invulnerable")>0||f.optInt("shield")>0)glow(c,x,y-22,42f,teal,70)
            val which=if(!enemy){when(f.optString("realm")){"Aurelon"->0;"Skarn"->1;else->2}}else if(boss)5 else when(role){"Foudre"->3;"Lien"->4;else->5}
            val sz=if(boss)130f else if(which==3)91f else 89f;val moving=abs(px-ox)+abs(py-oy)>4;val bob=if(moving)sin(tick*.65+i).toFloat()*2 else sin(tick*.08+i).toFloat()*.6f
            val flip=(f.optJSONObject("aim")?.optInt("x",1)?:1)<0
            if(f.optInt("airborne")>0){portrait(c,which,x-sz/2-(px-ox)/256f*tile*2,y-sz*.83f,sz,sz,65,flip);glow(c,x,y,36f,teal,100)}
            if(mine){when(snapshot.optString("equipped")){"skin_pierre"->paint.colorFilter=LightingColorFilter(0xffb8c1c4.toInt(),0x00202020);"skin_givre"->paint.colorFilter=LightingColorFilter(0xff9bd3f5.toInt(),0x00051820);"skin_seve"->paint.colorFilter=LightingColorFilter(0xff9beaaf.toInt(),0x00051808);"lantern_ambre"->glow(c,x,y-22,46f,gold,100);"signature_serment"->circle(c,x,y,24f,gold,1.5f)}}
            if((impactUntil[id]?:0)>SystemClock.uptimeMillis())paint.colorFilter=LightingColorFilter(Color.WHITE,0x00808080)
            val lean=if(moving)(px-ox).coerceIn(-60f,60f)*.07f else 0f
            c.save();c.rotate(lean,x,y);portrait(c,which,x-sz/2,y-sz*.83f+bob,sz,sz,255,flip);c.restore();paint.colorFilter=null
            if(!mine){val maxHp=f.optInt("max_hp",100).toFloat();panel(c,x-18,y-sz*.79f,36f,4f,0xb009151b.toInt(),0);paint.color=color;c.drawRect(x-18,y-sz*.79f,x-18+36*(hp/maxHp).coerceIn(0f,1f),y-sz*.79f+4,paint)}
        }
        val projectiles=b.optJSONArray("projectiles")?:JSONArray();for(i in 0 until projectiles.length()){val p=projectiles.getJSONObject(i);val pos=p.getJSONObject("pos");val vel=p.getJSONObject("vel");val x=left+pos.optInt("x")/256f*tile;val y=top+pos.optInt("y")/256f*tile;val col=if(p.optInt("owner")==local)gold else red;line(c,x-vel.optInt("x")/256f*tile*.6f,y-vel.optInt("y")/256f*tile*.6f,x,y,col,3.5f);glow(c,x,y,10f,col,160);circle(c,x,y,2.5f,ivory)}
        val now=SystemClock.uptimeMillis();damage.removeAll{now-it.born>850};damage.forEach{val age=(now-it.born)/850f;text(c,it.text,it.x,it.y-age*25,16f,Color.argb((255*(1-age)).toInt(),255,212,158),bold,Paint.Align.CENTER)}
        if(run!=null){
            val explored=run.optJSONArray("explored")?:JSONArray();val key=explored.toString()
            if(key!=fogKey){
                val mask=fog?:Bitmap.createBitmap(480,256,Bitmap.Config.ARGB_8888).also{fog=it}
                val maskCanvas=Canvas(mask);maskCanvas.drawColor(Color.TRANSPARENT,PorterDuff.Mode.CLEAR);maskCanvas.drawColor(0xe610212c.toInt())
                val reveal=Path();for(i in 0 until explored.length())if(explored.optBoolean(i))reveal.addRect((i%30)*16f,(i/30)*16f,(i%30+1)*16f,(i/30+1)*16f,Path.Direction.CW)
                val brush=Paint(Paint.ANTI_ALIAS_FLAG);brush.color=Color.WHITE;brush.xfermode=PorterDuffXfermode(PorterDuff.Mode.DST_OUT);brush.maskFilter=BlurMaskFilter(12f,BlurMaskFilter.Blur.NORMAL);maskCanvas.drawPath(reveal,brush);fogKey=key
            }
            paint.shader=null;paint.color=Color.WHITE;fog?.let{c.drawBitmap(it,null,RectF(left,top,left+aw,top+ah),paint)}
        }
        c.restore()
        // HUD stays at native resolution; gameplay is never uploaded as a low-resolution texture.
        panel(c,24f,18f,248f,53f,0xe80a1b24.toInt());val player=fighters.optJSONObject(local);val hp=player?.optInt("hp")?:0;val maxHp=player?.optInt("max_hp",160)?.toFloat()?:160f;text(c,"${snapshot.optString("realm_name")}  /  ${snapshot.optString("role_name")}",39f,38f,11f,gold,bold);panel(c,39f,49f,161f,6f,0xff293a41.toInt(),0);paint.color=teal;c.drawRoundRect(39f,49f,39+161*(hp/maxHp).coerceIn(0f,1f),55f,3f,3f,paint);text(c,"${hp.coerceAtLeast(0)}",225f,57f,15f,ivory,bold,Paint.Align.CENTER)
        val objective=if(snapshot.optBoolean("tutorial"))arrayOf("Avance avec le pouce gauche","Maintiens ATTAQUE près de l’ombre","Utilise l’esquive pour traverser le danger","Déclenche ton pouvoir","Dissipe l’ombre pour ouvrir la route")[snapshot.optInt("lesson").coerceIn(0,4)]else if(run!=null){if(run.optInt("wave")==0)when(run.optInt("kind")){1->"Ombres dissipées  ${min(6,player?.optInt("kills")?:0)} / 6";2->"Mémoires retrouvées  ${(0..2).count{charges?.optInt(it)==90}} / 3";else->"Feux rallumés  ${(0..2).count{charges?.optInt(it)==90}} / 3"}else"Dissipe le gardien de la porte"}else if(siege!=null)"Porte : ${siege.optInt("gate_hp")}  •  Cour : ${siege.optInt("capture")/30} / 60 s"else if(snapshot.optBoolean("online"))"Veilleurs réunis  •  ${snapshot.optInt("latency")} ms"else"Tiens la veille"
        panel(c,vw/2-200,17f,400f,47f,0xe60b1c25.toInt());fitText(c,objective,vw/2,46f,16f,ivory,376f,Paint.Align.CENTER)
        if(interactive)button(c,"pause","Ⅱ",vw-96,18f,67f,47f){command("pause")}
        if(run!=null){text(c,"${(3-(player?.optInt("deaths")?:0)).coerceAtLeast(0)} vies  •  ${(0..2).count{run.optJSONArray("caches")?.optBoolean(it)==true}} / 3 caches  •  ${run.optInt("dust")} ◆",vw/2,511f,13f,gold,sans,Paint.Align.CENTER)}else text(c,if(autoAim)"ATTAQUE AUTOMATIQUE"else"MAINTIENS ATTAQUE OU GLISSE POUR VISER",vw/2,511f,11f,muted,bold,Paint.Align.CENTER)
        if(run!=null){val mx=vw-208;val my=91f;panel(c,mx,my,174f,98f,0xeb0c1d26.toInt());val explored=run.optJSONArray("explored")?:JSONArray();for(i in 0 until explored.length()){if(explored.optBoolean(i)){paint.color=0xff36525a.toInt();c.drawRect(mx+7+(i%30)*5.3f,my+7+(i/30)*5.2f,mx+12+(i%30)*5.3f,my+12+(i/30)*5.2f,paint)}};for(i in 0..2)circle(c,mx+7+points[i].first*5.3f,my+7+points[i].second*5.2f,2.5f,if(charges?.optInt(i)==90)gold else teal);circle(c,mx+7+focus.optInt("x")/256f*5.3f,my+7+focus.optInt("y")/256f*5.2f,3f,ivory)
            if(interactive)button(c,"heal","Soin  ${run.optInt("flasks")} / 2",24f,95f,130f,42f){engine.uiAction("heal")}}
        if(interactive){controls(c,player);if(SystemClock.uptimeMillis()-screenStarted<4500){panel(c,vw/2-207,420f,414f,52f,0xda0a1c24.toInt());fitText(c,if(run!=null)if(run.optInt("kind")==1)"Approche, attaque, esquive. Six ombres puis leur chef."else "Explore la brume. Les caches contiennent de l’équipement."else"Gauche : avancer  •  ATTAQUE : tirer  •  ◇ : esquiver",vw/2,451f,14f,ivory,386f,Paint.Align.CENTER)}}
    }
    private fun controls(c:Canvas,player:JSONObject?){val origin=if(movePointer<0)PointF(99f,435f)else moveOrigin;circle(c,origin.x,origin.y,49f,0x752d4855);circle(c,origin.x,origin.y,49f,0x806e919d.toInt(),1.2f);circle(c,origin.x+input.mx/1024f*32,origin.y+input.my/1024f*32,23f,0xc072929a.toInt());text(c,"DÉPLACER",99f,510f,10f,muted,bold,Paint.Align.CENTER)
        if(aimPointer>=0){circle(c,aimOrigin.x,aimOrigin.y,46f,0x40755749);circle(c,aimOrigin.x+input.ax/1024f*30,aimOrigin.y+input.ay/1024f*30,16f,0xaaddb879.toInt())}
        ability(c,"attack","⌖","ATTAQUE",vw-91,326f,player?.optInt("cooldown")?:0,25f){attackRequest.set(true)}
        ability(c,"dash","◇","ESQUIVE",vw-91,445f,player?.optInt("dash_cd")?:0,105f){dashRequest.set(true)};ability(c,"skill","✦","POUVOIR",vw-199,445f,player?.optInt("skill_cd")?:0,180f){skillRequest.set(true)}
    }
    private fun ability(c:Canvas,id:String,symbol:String,label:String,x:Float,y:Float,cool:Int,max:Float,run:()->Unit){circle(c,x,y,43f,0xe01b3440.toInt());circle(c,x,y,43f,if(cool==0)gold else 0xff50646d.toInt(),1.8f);if(cool==0)glow(c,x,y,37f,gold,25);text(c,if(cool==0)symbol else String.format(java.util.Locale.US,"%.1f",cool/30f),x,y+11,if(cool==0)31f else 23f,if(cool==0)gold else muted,serif,Paint.Align.CENTER);if(cool>0){paint.color=gold;paint.style=Paint.Style.STROKE;paint.strokeWidth=3f;c.drawArc(x-43,y-43,x+43,y+43,-90f,360*(1-cool/max),false,paint);paint.style=Paint.Style.FILL};text(c,label,x,y+65f,10f,muted,bold,Paint.Align.CENTER);buttons.add(Button(id,RectF(x-48,y-48,x+48,y+48),run))}
    private fun pause(c:Canvas){buttons.clear();fill(c,0xbd061018.toInt());panel(c,vw/2-228,112f,456f,315f,0xf7112631.toInt());text(c,"Une respiration.",vw/2,173f,36f,ivory,serif,Paint.Align.CENTER);text(c,if(snapshot.optBoolean("online"))"Le combat continue en ligne."else"La nuit peut attendre un instant.",vw/2,210f,17f,muted,sans,Paint.Align.CENTER);button(c,"resume","Reprendre la veille",vw/2-176,246f,352f,58f,true){command("resume")};button(c,"leave","Retourner au refuge",vw/2-176,324f,352f,52f){command("home")}}
    private fun result(c:Canvas){background(c,.54f);val b=snapshot.optJSONObject("battle");val run=snapshot.optJSONObject("expedition");val win=if(run!=null)run.optBoolean("victory")else if(b?.optJSONObject("siege")!=null)b.getJSONObject("siege").optBoolean("captured")else snapshot.optBoolean("tutorial")||(b?.optJSONArray("fighters")?.optJSONObject(0)?.optInt("kills")?:0)>0;val x=vw/2;text(c,if(win)"LA NUIT RECULE"else"LA FLAMME DEMEURE",x,122f,13f,gold,bold,Paint.Align.CENTER);text(c,if(win)"Tu as porté la lumière."else"Chaque veille t’apprend.",x,202f,45f,ivory,serif,Paint.Align.CENTER);val kills=b?.optJSONArray("fighters")?.optJSONObject(0)?.optInt("kills")?:0;val seconds=(b?.optInt("tick")?:0)/30;val lit=run?.optJSONArray("charges");val subtitle=if(run!=null)"${(0..2).count{lit?.optInt(it)==90}} balises réveillées  •  Ombres dissipées : $kills"else"Ombres dissipées : $kills  •  ${seconds/60} min ${seconds%60} s";text(c,subtitle,x,261f,19f,muted,sans,Paint.Align.CENTER);if(run!=null)text(c,"Butin : ${(0..2).count{run.optJSONArray("caches")?.optBoolean(it)==true}+(if(win)1 else 0)} objets  •  ${run.optInt("dust")} poussières",x,300f,17f,teal,sans,Paint.Align.CENTER)else ornament(c,x-56,300f);button(c,"again",if(run!=null)"Examiner mon butin  ›"else "Découvrir le refuge  ›",x-180,342f,360f,61f,true){command("finish");if(run!=null){chosenItem=0;inventoryPage=0;command("inventory")}};button(c,"finish","Retrouver le refuge",x-180,423f,360f,51f){command("finish")}}
    private fun legacy(c:Canvas){background(c,.79f);val commands=snapshot.optJSONArray("ui")?:JSONArray();val logical=snapshot.optInt("width",logicalWidth);val sc=min(2f,(vw-32)/logical);val left=(vw-logical*sc)/2;val top=30f
        c.save();c.translate(left,top);c.scale(sc,sc)
        for(i in 0 until commands.length()){val d=commands.getJSONObject(i);val x=d.optInt("x").toFloat();val y=d.optInt("y").toFloat();when(d.optString("kind")){
            "Panel"->panel(c,x,y,d.optInt("w").toFloat(),d.optInt("h").toFloat(),0xed102531.toInt())
            "Text"->{val raw=d.optString("text").replace("V0.2","LES BRAISES").replace("v0.2","v0.4");val size=if(d.optInt("scale")==2)17f else 9.7f;val color=d.optLong("color").toInt();text(c,raw,x,y+size*.83f,size,if(color==0)ivory else color or 0xff000000.toInt(),if(d.optInt("scale")==2)serif else sans)}
            "Button"->{val w=d.optInt("w").toFloat();val lit=d.optBoolean("active");panel(c,x,y,w,24f,if(lit)0xff294550.toInt()else 0xff1a3541.toInt(),if(lit)gold else 0x8065868e.toInt());fitText(c,d.optString("text"),x+w/2,y+16f,10f,ivory,w-12,Paint.Align.CENTER)}
            "Sprite"->{portrait(c,d.optInt("realm").coerceIn(0,2),x-6,y-8,30f,30f)}
        }}
        if(snapshot.optInt("screen")==25){val a=snapshot.optJSONArray("emblem")?:JSONArray();for(i in 0..255){paint.color=intArrayOf(0xff152733.toInt(),gold,teal,ivory)[a.optInt(i).coerceIn(0,3)];c.drawRect(16f+(i%16)*8,50f+(i/16)*8,23f+(i%16)*8,57f+(i/16)*8,paint)}}
        c.restore()
    }
    override fun onTouchEvent(event:MotionEvent):Boolean{
        if(height<=0)return true
        val scale=viewportScale;val phase=event.actionMasked
        if(phase==MotionEvent.ACTION_CANCEL){clearInput();return true}
        val battle=snapshot.optInt("screen")==6&&intro<0&&!heroChoice
        for(i in 0 until event.pointerCount){if(phase!=MotionEvent.ACTION_MOVE&&i!=event.actionIndex)continue
            val id=event.getPointerId(i);val x=event.getX(i)/scale;val y=(event.getY(i)-viewportTop)/scale
            val down=phase==MotionEvent.ACTION_DOWN||phase==MotionEvent.ACTION_POINTER_DOWN
            val up=phase==MotionEvent.ACTION_UP||phase==MotionEvent.ACTION_POINTER_UP
            if(down){
                val hit=buttons.lastOrNull{it.rect.contains(x,y)}
                if(hit!=null){if(battle){if(hit.id=="attack"){attackPointer=id;attackHeld=true;attackRequest.set(true)};hit.run()}else{focused=hit;pointerButton=id};continue}
                if(battle){if(x<vw*.43f&&y>90&&movePointer<0){movePointer=id;moveOrigin=PointF(x,y)}else if(x>vw*.52f&&y>90&&aimPointer<0){aimPointer=id;aimOrigin=PointF(x,y)}}
                else if(intro<0&&!heroChoice&&snapshot.optInt("screen") !in listOf(0,2,3,7,10,12,14,16,20,40,41)){val sc=min(2f,(vw-32)/logicalWidth);val left=(vw-logicalWidth*sc)/2;engine.touch(id,0u,((x-left)/sc).toInt(),((y-30)/sc).toInt())}
            }
            if(battle){if(id==attackPointer){if(up){attackHeld=false;attackPointer=-1}else if(hypot(x-(vw-91),y-326)>25){attackHeld=false;aimPointer=id;aimOrigin=PointF(vw-91,326f)}};if(id==movePointer){if(up){movePointer=-1;input=input.copy(mx=0,my=0)}else{val dx=x-moveOrigin.x;val dy=y-moveOrigin.y;val len=hypot(dx,dy);if(len>65){moveOrigin.x=x-dx/len*65;moveOrigin.y=y-dy/len*65};val power=((len-3)/42).coerceIn(0f,1f);input=input.copy(mx=(dx/len.coerceAtLeast(1f)*power*1024).toInt().toShort(),my=(dy/len.coerceAtLeast(1f)*power*1024).toInt().toShort())}};if(id==aimPointer){if(up){aimPointer=-1;input=input.copy(ax=0,ay=0)}else{val dx=x-aimOrigin.x;val dy=y-aimOrigin.y;val len=hypot(dx,dy);val power=((len-4)/35).coerceIn(0f,1f);input=input.copy(ax=(dx/len.coerceAtLeast(1f)*power*1024).toInt().toShort(),ay=(dy/len.coerceAtLeast(1f)*power*1024).toInt().toShort())}}}
            if(up){if(id==pointerButton){val hit=focused;focused=null;pointerButton=-1;if(hit!=null&&hit.rect.contains(x,y)){performClick();hit.run()}};engine.touch(id,2u,0,0)}
        }
        invalidate();return true
    }
    override fun performClick():Boolean {super.performClick();return true}
}
