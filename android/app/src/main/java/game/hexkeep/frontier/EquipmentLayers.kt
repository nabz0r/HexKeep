package game.hexkeep.frontier

import android.graphics.Canvas
import kotlin.math.*
import org.json.JSONObject

/** Six independent, ordered sprite layers. Shapes are code-native procedural assets. */
class EquipmentLayers(private val u: UiKit) {
    private val rarity =
        intArrayOf(0xffacbfc2.toInt(), 0xff7dd9c1.toInt(), 0xffc9a4ed.toInt(), 0xffefc277.toInt())

    fun tint(item: JSONObject?) = rarity[(item?.optInt("rarity") ?: 0).coerceIn(0, 3)]

    fun icon(c: Canvas, slot: Int, x: Float, y: Float, size: Float, color: Int) {
        c.save()
        c.translate(x, y)
        c.scale(size / 24, size / 24)
        when (slot) {
            0 -> {
                c.rotate(35f)
                u.rect(c, -2f, -17f, 4f, 28f, color, 1f)
                u.rect(c, -8f, 5f, 16f, 3f, u.gold, 1f)
                u.rect(c, -2f, 9f, 4f, 9f, u.muted, 1f)
            }
            1 -> {
                u.polygon(c, 0f, 0f, 17f, 6, color, 30f)
                u.rect(c, -5f, -15f, 10f, 8f, u.ink, 3f)
                u.line(c, -7f, -3f, 7f, -3f, u.gold, 2f)
                u.line(c, 0f, -2f, 0f, 12f, u.gold, 1f)
            }
            2 -> {
                u.circle(c, 0f, -4f, 11f, color, 2f)
                u.polygon(c, 0f, 8f, 8f, 4, u.gold)
                u.circle(c, 0f, 8f, 3f, u.ink)
            }
            3 -> {
                u.polygon(c, 0f, 0f, 17f, 6, color, 30f)
                u.rect(c, -12f, 0f, 24f, 4f, u.ink, 1f)
                u.rect(c, -2f, -16f, 4f, 10f, u.gold, 1f)
            }
            4 -> {
                u.rect(c, -14f, -10f, 10f, 25f, color, 3f)
                u.rect(c, 4f, -10f, 10f, 25f, color, 3f)
                u.rect(c, -12f, 3f, 6f, 3f, u.gold, 1f)
                u.rect(c, 6f, 3f, 6f, 3f, u.gold, 1f)
            }
            else -> {
                u.rect(c, -13f, -12f, 9f, 23f, color, 2f)
                u.rect(c, 3f, -12f, 9f, 23f, color, 2f)
                u.rect(c, -13f, 7f, 13f, 7f, color, 2f)
                u.rect(c, 3f, 7f, 13f, 7f, color, 2f)
            }
        }
        c.restore()
    }

    fun hero(
        c: Canvas,
        x: Float,
        y: Float,
        scale: Float,
        player: JSONObject,
        journey: JSONObject,
        t: Float,
    ) {
        val items = journey.array("items").objects()
        val eq = journey.array("equipped")
        fun item(slot: Int) = items.firstOrNull { it.optLong("id") == eq.optLong(slot) }
        val state = player.optString("state")
        val stance = player.optString("stance", "balanced")
        val moving = state == "move" || state == "dodge"
        val bob = if (moving) sin(t * 11) * 1.8f else sin(t * 2) * .5f
        val leg = if (moving) sin(t * 11) * 4 else 0f
        c.save()
        c.translate(x, y)
        c.scale(scale, scale)
        val facing = player.optJSONObject("facing")
        if ((facing?.optInt("x") ?: 1) < 0) c.scale(-1f, 1f)
        u.circle(c, 0f, 3f, 14f, 0x60000000)
        c.translate(0f, bob)
        if (state == "dodge") {
            u.polygon(c, -10f, -15f, 18f, 3, u.alpha(u.mint, 70), 180f)
            c.rotate(-15f)
        }
        // Cape, boots, torso, gloves, necklace, helmet, weapon: depth order stays stable.
        u.polygon(
            c,
            0f,
            -13f,
            20f,
            3,
            if (stance == "assault") 0xff733f4e.toInt() else 0xff244e58.toInt(),
            -90f,
        )
        val boots = tint(item(5))
        u.rect(c, -10f, -7f + leg, 7f, 12f, boots, 2f)
        u.rect(c, 3f, -7f - leg, 7f, 12f, boots, 2f)
        u.polygon(c, 0f, -20f, 12f, 6, tint(item(1)), 30f)
        u.line(c, 0f, -28f, 0f, -11f, u.alpha(u.ink, 120), 2f)
        val glove = tint(item(4))
        u.circle(c, -14f, -20f, 4f, glove)
        u.circle(c, 14f, -20f, 4f, glove)
        u.polygon(c, 0f, -24f, 4f, 4, tint(item(2)))
        u.circle(c, 0f, -24f, 1.8f, u.gold)
        u.polygon(c, 0f, -36f, 9f, 6, tint(item(3)), 30f)
        u.rect(c, -6f, -36f, 12f, 3f, u.ink, 1f)
        val motif = item(3)?.optInt("motif") ?: 0
        u.polygon(c, 0f, -45f, 3f + motif, 3, u.gold)
        val angle =
            when (state) {
                "windup" -> -75f
                "strike" -> if (player.optInt("combo") == 3) 100f else 65f
                "recovery" -> 40f
                else -> if (stance == "bulwark") -28f else 10f
            }
        c.save()
        c.translate(13f, -20f)
        c.rotate(angle)
        u.rect(
            c,
            -2f,
            -24f - (item(0)?.optInt("catalog") ?: 0) % 3 * 2,
            4f,
            26f + (item(0)?.optInt("catalog") ?: 0) % 3 * 2,
            tint(item(0)),
            1f,
        )
        u.rect(c, -7f, -2f, 14f, 3f, u.gold, 1f)
        c.restore()
        if (stance == "bulwark") u.polygon(c, -16f, -20f, 10f, 6, u.mint, 30f, 2f)
        if (state == "hurt") u.circle(c, 0f, -23f, 21f, u.alpha(u.red, 110))
        if (state == "strike") {
            u.p.style = android.graphics.Paint.Style.STROKE
            u.p.strokeWidth = if (player.optInt("combo") == 3) 5f else 2f
            u.p.color = u.gold
            c.drawArc(-31f, -51f, 31f, 11f, -90f, 150f, false, u.p)
            u.p.style = android.graphics.Paint.Style.FILL
        }
        c.restore()
    }
}
