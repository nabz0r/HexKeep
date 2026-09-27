package game.hexkeep

import game.hexkeep.art.PaintedArt
import android.graphics.*
import android.os.SystemClock
import org.json.JSONObject
import kotlin.math.*

/** Visual events follow authoritative snapshots; display refresh never changes combat timing. */
class ActorAnimator(private val art: PaintedArt) {
    data class Motion(
        var px:Int=0,var py:Int=0,var hp:Int=0,var maximum:Int=0,
        var facingX:Float=1f,var facingY:Float=1f,var distance:Float=0f,
        var moving:Boolean=false,var attack:Long=0,var dash:Long=0,var hurt:Long=0,
        var death:Long=0,var cooldown:Int=0,var dashCd:Int=0,var skillCd:Int=0,
        var cast:Long=0,var dx:Float=0f,var dy:Float=0f
    )
    private val actors=HashMap<Int,Motion>()
    private var seed=Long.MIN_VALUE
    private var tick=-1
    private var walkedFrames=0L
    private var turns=0L
    private var attacks=0L
    private var dodges=0L
    private var enemyAttacks=0L
    private var playerAttacks=0L
    private var playerDodges=0L
    private var playerTurns=0L
    private val playerFrames=HashSet<Int>()
    private var localId=0
    private val observedFrames=HashSet<String>()
    fun state(id:Int)=actors[id]
    fun sync(snapshot:JSONObject) {
        val b=snapshot.optJSONObject("battle")?:return
        localId=snapshot.optInt("local")
        val next=b.optInt("tick")
        if(seed!=b.optLong("seed")||next<tick){actors.clear();seed=b.optLong("seed");tick=-1}
        if(next==tick)return
        tick=next
        val now=SystemClock.uptimeMillis()
        val fs=b.optJSONArray("fighters")?:return
        for(i in 0 until fs.length()) {
            val f=fs.getJSONObject(i);val id=f.optInt("id");val p=f.getJSONObject("pos")
            val x=p.optInt("x");val y=p.optInt("y");val hp=f.optInt("hp")
            val fresh=!actors.containsKey(id)||actors[id]?.maximum!=f.optInt("max_hp")
            val m=if(fresh)Motion(x,y,hp,f.optInt("max_hp")).also{actors[id]=it}else actors.getValue(id)
            val dx=(x-m.px).toFloat();val dy=(y-m.py).toFloat();val length=hypot(dx,dy)
            m.moving=length in 2f..599f && hp>0
            if(m.moving){m.distance+=length/256f;m.dx=dx;m.dy=dy}
            val aim=f.optJSONObject("aim")
            val attack=f.optInt("cooldown")>m.cooldown && !fresh && hp>0
            val dash=f.optInt("dash_cd")>m.dashCd && !fresh && hp>0
            if(attack){m.attack=now+230;attacks++;if(f.optBoolean("bot"))enemyAttacks++;if(id==localId)playerAttacks++}
            if(dash){m.dash=now+300;dodges++;if(id==localId)playerDodges++}
            if(f.optInt("skill_cd")>m.skillCd && !fresh)m.cast=now+500
            val oldBack=m.facingY<-.12f;val oldFlip=m.facingX<0
            if(attack || (!m.moving && now<m.attack)) {
                val ax=aim?.optInt("x")?.toFloat()?:1f;val ay=aim?.optInt("y")?.toFloat()?:0f
                val norm=hypot(ax,ay).coerceAtLeast(1f);m.facingX=ax/norm;m.facingY=ay/norm
            } else if(m.moving && now>=m.attack) {m.facingX=dx/length;m.facingY=dy/length}
            if(oldBack!=(m.facingY<-.12f)||oldFlip!=(m.facingX<0)){turns++;if(id==localId)playerTurns++}
            if(hp<m.hp && !fresh)m.hurt=now+150
            if(hp<=0 && m.hp>0)m.death=now
            if(hp>0)m.death=0
            m.px=x;m.py=y;m.hp=hp;m.maximum=f.optInt("max_hp")
            m.cooldown=f.optInt("cooldown");m.dashCd=f.optInt("dash_cd");m.skillCd=f.optInt("skill_cd")
        }
    }
    private fun sheet(which:Int)=if(which<3)art.heroNames[which.coerceIn(0,2)]else art.enemyNames[(which-3).coerceIn(0,3)]
    private fun frame(c:Canvas,s:String,index:Int,x:Float,y:Float,size:Float,flip:Boolean,alpha:Int,paint:Paint) {
        art.anchored(c,s,index,x,y,size,flip,alpha,paint.colorFilter)
    }
    fun draw(c:Canvas,paint:Paint,which:Int,id:Int,x:Float,y:Float,size:Float,color:Int,reduced:Boolean=false) {
        val m=actors[id]?:return
        val now=SystemClock.uptimeMillis();val s=sheet(which)
        val back=m.facingY<-.12f;val flip=m.facingX<0
        val walk=if(m.moving)floor(m.distance*2.6f).toInt()%4 else 0
        val index=when { now<m.dash -> if(back)11 else 10;now<m.attack || now<m.cast -> if(which<3){if(back)9 else 8}else{if(back)10 else 9};else->(if(back)4 else 0)+walk }
        if(m.moving&&observedFrames.add("$id:$index"))walkedFrames++
        if(id==localId && m.moving && index<8)playerFrames.add(index)
        if(m.hp<=0) {
            if(m.death==0L || now-m.death>650)return
            val p=(now-m.death)/650f;c.save();c.rotate(if(flip)-p*65 else p*65,x,y)
            frame(c,s,index,x,y,size*(1-p*.22f),flip,(255*(1-p)).toInt(),paint);c.restore();return
        }
        val originalFilter=paint.colorFilter
        if(now<m.hurt)paint.colorFilter=LightingColorFilter(Color.WHITE,0x00805040)
        if(now<m.dash&&!reduced) {
            for(i in 3 downTo 1)frame(c,s,index,x-m.facingX*i*11,y-m.facingY*i*8,size,flip,45-i*8,paint)
        }
        val breath=if(m.moving)sin(m.distance*PI*5).toFloat()*.8f else sin(now*.002+id).toFloat()*.6f
        val recoil=if(now<m.attack)(1-(m.attack-now)/230f)*3.5f else 0f
        c.save()
        if(now<m.dash)c.rotate(sin((1-(m.dash-now)/300f)*PI).toFloat()*(if(flip)18 else -18),x,y)
        frame(c,s,index,x+m.facingX*recoil,y+breath,size,flip,255,paint)
        c.restore();paint.colorFilter=originalFilter
        if(now<m.attack) {
            val progress=(1-(m.attack-now)/230f).coerceIn(0f,1f)
            val angle=atan2(m.facingY,m.facingX)*180f/PI.toFloat()
            paint.color=color;paint.alpha=(170*(1-progress)).toInt();paint.style=Paint.Style.STROKE;paint.strokeWidth=5f*(1-progress)+1f
            c.drawArc(x-36,y-48,x+36,y+16,angle-65,130*progress,false,paint)
            if(!reduced)for(k in 0..3){val a=(angle-40+k*25)*PI/180;val radius=28+progress*24;c.drawCircle(x+cos(a).toFloat()*radius,y-16+sin(a).toFloat()*radius,1.5f,paint)}
            paint.style=Paint.Style.FILL;paint.alpha=255
        }
        if(now<m.cast) {
            val p=(1-(m.cast-now)/500f).coerceIn(0f,1f);paint.color=color;paint.alpha=(130*(1-p)).toInt();paint.style=Paint.Style.STROKE;paint.strokeWidth=2f
            c.drawOval(x-18-p*20,y-5-p*7,x+18+p*20,y+5+p*7,paint);paint.style=Paint.Style.FILL;paint.alpha=255
        }
    }
    fun metrics():JSONObject=JSONObject().put("walk_frames",walkedFrames).put("direction_changes",turns).put("attack_animations",attacks).put("dodge_animations",dodges).put("enemy_attack_animations",enemyAttacks).put("player_walk_frames",playerFrames.size).put("player_turns",playerTurns).put("player_attacks",playerAttacks).put("player_dodges",playerDodges).put("player_facing_x",actors[localId]?.facingX?:0f).put("player_facing_y",actors[localId]?.facingY?:0f)
}
