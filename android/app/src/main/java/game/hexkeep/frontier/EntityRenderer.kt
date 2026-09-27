package game.hexkeep.frontier

import android.graphics.*
import game.hexkeep.art.PaintedArt
import kotlin.math.*
import org.json.JSONObject

/** Readable telegraphs around painted, state-driven creature animation. */
class EntityRenderer(private val u: UiKit, private val art: PaintedArt) {
    private data class Motion(
        var x: Int,
        var y: Int,
        var back: Boolean = false,
        var flip: Boolean = false,
    )

    private val motion = HashMap<Int, Motion>()

    fun clear() = motion.clear()

    fun monster(c: Canvas, m: JSONObject, x: Float, y: Float, t: Float, zone: Int) {
        val kind = m.optInt("kind").coerceIn(0, 4)
        val hp = m.optInt("hp")
        val name =
            if (kind == 4) "boss-${art.biomeNames[zone.coerceIn(0,2)]}" else art.enemyNames[kind]
        val pos = m.obj("pos")
        val xx = pos.optInt("x")
        val yy = pos.optInt("y")
        val a = motion.getOrPut(m.optInt("id")) { Motion(xx, yy) }
        if (xx != a.x || yy != a.y) {
            if (xx != a.x) a.flip = xx < a.x
            if (yy != a.y) a.back = yy < a.y
            a.x = xx
            a.y = yy
        }
        val state = m.optString("state")
        val frame =
            when (state) {
                "windup" -> 8
                "strike" -> if (a.back) 10 else 9
                "hurt",
                "dead" -> 11
                "move" ->
                    (if (a.back) 4 else 0) +
                        ((t * (if (kind == 3) 12 else 7) + m.optInt("id")).toInt() % 4)
                else -> if (a.back) 4 else 0
            }
        val height =
            when (kind) {
                0 -> 65f
                1 -> 81f
                2 -> 96f
                3 -> 45f
                else -> 148f
            }
        if (hp <= 0) {
            val alpha = (m.optInt("corpse") * 3).coerceIn(0, 130)
            if (alpha > 0) {
                c.save()
                c.rotate(68f, x, y)
                art.anchored(c, name, 11, x, y, height * .8f, a.flip, alpha)
                c.restore()
            }
            return
        }
        val r = if (kind == 4) 45f else if (kind == 2) 28f else 22f
        u.p.color = 0x78000000
        c.drawOval(x - r, y - 5, x + r, y + 8, u.p)
        if (state == "windup") {
            u.p.color = u.alpha(u.red, 65)
            c.drawOval(x - r - 15, y - r * .45f - 10, x + r + 15, y + r * .45f + 10, u.p)
            u.p.style = Paint.Style.STROKE
            u.p.strokeWidth = 2f
            u.p.color = u.red
            c.drawOval(x - r - 15, y - r * .45f - 10, x + r + 15, y + r * .45f + 10, u.p)
            u.p.style = Paint.Style.FILL
            u.text(c, "!", x, y - height - 10, 20f, u.red, true, u.bold)
        }
        art.anchored(
            c,
            name,
            frame,
            x,
            y + if (state == "move") sin(t * 12) * 1.2f else 0f,
            height,
            a.flip,
            if (m.optBoolean("active")) 255 else 155,
            if (state == "hurt") LightingColorFilter(Color.WHITE, 0x00602010) else null,
        )
        if (m.optString("stance") == "bulwark")
            u.circle(c, x, y - height * .48f, height * .44f, u.alpha(u.mint, 80), 1f)
        if (hp < m.optInt("max_hp") || kind == 4) {
            u.rect(c, x - r, y - height - 6, 2 * r, 4f, u.ink, 1f)
            u.rect(
                c,
                x - r,
                y - height - 6,
                2 * r * hp / m.optInt("max_hp").coerceAtLeast(1),
                4f,
                u.red,
                1f,
            )
        }
        if (!m.optBoolean("active")) u.text(c, "3 BALISES", x, y + 23, 10f, u.gold, true)
    }
}
