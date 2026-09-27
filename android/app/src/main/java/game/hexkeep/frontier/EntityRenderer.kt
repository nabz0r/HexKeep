package game.hexkeep.frontier

import android.graphics.*
import kotlin.math.*
import org.json.JSONObject

class EntityRenderer(private val u: UiKit) {
    fun monster(c: Canvas, m: JSONObject, x: Float, y: Float, t: Float, accent: Int) {
        val kind = m.optInt("kind")
        val hp = m.optInt("hp")
        if (hp <= 0) {
            u.polygon(c, x, y, 14f, 6, u.alpha(accent, m.optInt("corpse") * 2), 30f, 1f)
            return
        }
        val radius = if (kind == 4) 31f else if (kind == 2) 22f else if (kind == 3) 11f else 16f
        val state = m.optString("state")
        val stance = m.optString("stance")
        u.circle(c, x, y + 5, radius * .9f, 0x60000000)
        val bob =
            if (state == "move") sin(t * (if (kind == 3) 14 else 8) + m.optInt("id")) * 2
            else sin(t * 2 + m.optInt("id")) * .7f
        if (state == "windup") {
            u.circle(c, x, y, radius + 13, u.alpha(u.red, 45))
            u.circle(c, x, y, radius + 13, u.red, 1.5f)
            u.text(c, "!", x, y - radius - 19, 17f, u.red, true, u.bold)
        }
        c.save()
        c.translate(x, y - 12 + bob)
        when (kind) {
            0 -> {
                u.polygon(c, 0f, 0f, 19f, 3, accent, if (state == "strike") 30f else -90f)
                u.line(c, -12f, 0f, -24f, 9f, u.gold, 3f)
                u.line(c, 12f, 0f, 24f, 9f, u.gold, 3f)
            }
            1 -> {
                u.polygon(c, 0f, 0f, 18f, 4, accent, if (state == "windup") -65f else -90f)
                u.polygon(c, 0f, 0f, 10f, 4, u.ink)
                u.circle(c, 0f, -1f, 4f, u.gold)
                u.polygon(c, 0f, -25f, 6f, 3, u.alpha(accent, 170))
            }
            2 -> {
                u.polygon(c, 0f, 0f, 24f, 6, 0xff627e83.toInt(), 30f)
                u.polygon(c, 0f, 0f, 17f, 6, u.ink, 30f)
                u.rect(c, -5f, -10f, 10f, 20f, accent, 3f)
                if (stance == "bulwark") u.polygon(c, 0f, 0f, 28f, 6, u.mint, 30f, 2f)
            }
            3 -> {
                for (i in 0..2) u.polygon(
                    c,
                    (i - 1) * 9f,
                    if (i == 1) -6f else 3f,
                    8f,
                    3,
                    accent,
                    t * 40 + i * 120,
                )
            }
            else -> {
                u.polygon(c, 0f, 0f, 34f, 6, 0xff364b60.toInt(), 30f)
                u.polygon(c, 0f, 0f, 26f, 6, accent, 30f, 3f)
                for (i in 0..4) {
                    val a = (-155 + i * 32) * Math.PI / 180
                    val xx = cos(a).toFloat() * 30
                    val yy = sin(a).toFloat() * 30
                    u.polygon(c, xx, yy - 12, 12f, 3, u.gold, -90f + i * 10)
                }
                u.polygon(c, 0f, 0f, 12f, 4, u.gold)
                u.polygon(c, 0f, 0f, 6f, 4, u.ink)
                if (m.optInt("phase") > 0) u.circle(c, 0f, 0f, 40f, u.alpha(u.red, 100), 2f)
            }
        }
        c.restore()
        if (hp < m.optInt("max_hp") || kind == 4) {
            u.rect(c, x - radius, y - radius - 28, 2 * radius, 3f, u.ink, 2f)
            u.rect(
                c,
                x - radius,
                y - radius - 28,
                2 * radius * hp / m.optInt("max_hp").coerceAtLeast(1),
                3f,
                u.red,
                2f,
            )
        }
        if (!m.optBoolean("active")) u.text(c, "3 BALISES", x, y + 42, 10f, u.muted, true)
    }
}
