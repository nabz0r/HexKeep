package game.hexkeep.frontier

import android.graphics.*
import game.hexkeep.art.PaintedArt
import kotlin.math.*
import org.json.JSONObject

/** Painted character, with six material masks retaining the source brushwork and silhouette. */
class EquipmentLayers(private val u: UiKit, private val art: PaintedArt) {
    var realm = 0
    private val rarity =
        intArrayOf(0xffacbfc2.toInt(), 0xff7dd9c1.toInt(), 0xffc9a4ed.toInt(), 0xffefc277.toInt())
    private val materials =
        intArrayOf(0xffd0b18b.toInt(), 0xff86acaa.toInt(), 0xffa591bf.toInt(), 0xffb7c5d2.toInt())
    private val filters = materials.map { LightingColorFilter(it, 0x00161008) }

    fun tint(item: JSONObject?) = rarity[(item?.optInt("rarity") ?: 0).coerceIn(0, 3)]

    fun icon(c: Canvas, slot: Int, x: Float, y: Float, size: Float, color: Int) {
        art.fit(
            c,
            "items-${art.itemNames[slot.coerceIn(0,5)]}",
            0,
            RectF(x - size, y - size, x + size, y + size),
            105,
        )
    }

    fun itemIcon(c: Canvas, item: JSONObject, x: Float, y: Float, size: Float) =
        art.item(c, item, RectF(x - size, y - size, x + size, y + size))

    fun hero(
        c: Canvas,
        x: Float,
        y: Float,
        scale: Float,
        player: JSONObject,
        journey: JSONObject,
        t: Float,
    ) {
        val state = player.optString("state")
        val facing = player.optJSONObject("facing")
        val back = (facing?.optInt("y") ?: 1) < 0
        val flip = (facing?.optInt("x") ?: 1) < 0
        val index =
            when (state) {
                "windup" -> if (back) 9 else 8
                "strike" -> if (back) 9 else 8
                "hurt" -> if (back) 4 else 0
                "dead" -> 11
                "dodge" -> if (back) 11 else 10
                "move" -> (if (back) 4 else 0) + ((t * 9).toInt() % 4)
                else -> if (back) 4 else 0
            }
        val height = 82f * scale
        val bob = if (state == "move") sin(t * 16) * scale else sin(t * 2) * .5f * scale
        val name = art.heroNames[realm.coerceIn(0, 2)]
        u.p.color = 0x68000000
        c.drawOval(x - 23 * scale, y - 5 * scale, x + 23 * scale, y + 6 * scale, u.p)
        val alpha = if (state == "dead") 105 else 255
        if (state == "dodge")
            art.anchored(c, name, index, x + (if (flip) 15 else -15) * scale, y, height, flip, 75)
        val dest = art.anchored(c, name, index, x, y + bob, height, flip, alpha)
        val items = journey.array("items").objects()
        val eq = journey.array("equipped")
        // Material regions: head, cuirass, weapon, gloves, boots, amulet. They are clipped copies
        // of the painted pose, not solid replacement shapes, and follow mirroring/animation.
        val masks =
            arrayOf(
                floatArrayOf(.62f, .28f, 1f, .91f),
                floatArrayOf(.24f, .29f, .71f, .64f),
                floatArrayOf(.40f, .31f, .61f, .43f),
                floatArrayOf(.26f, 0f, .74f, .27f),
                floatArrayOf(0f, .40f, .35f, .68f),
                floatArrayOf(.15f, .73f, .81f, 1f),
            )
        for (slot in 0..5) {
            val item = items.firstOrNull { it.optLong("id") == eq.optLong(slot) } ?: continue
            val m = masks[slot]
            val l = if (flip) 1 - m[2] else m[0]
            val r = if (flip) 1 - m[0] else m[2]
            c.save()
            c.clipRect(
                dest.left + dest.width() * l,
                dest.top + dest.height() * m[1],
                dest.left + dest.width() * r,
                dest.top + dest.height() * m[3],
            )
            art.anchored(
                c,
                name,
                index,
                x,
                y + bob,
                height,
                flip,
                120,
                filters[(item.optInt("catalog") + item.optInt("motif")) % 4],
            )
            c.restore()
        }
        if (state == "hurt")
            art.anchored(
                c,
                name,
                index,
                x,
                y + bob,
                height,
                flip,
                100,
                LightingColorFilter(Color.WHITE, 0x00702010),
            )
        if (state == "strike") {
            u.p.color = u.gold
            u.p.style = Paint.Style.STROKE
            u.p.strokeWidth = (if (player.optInt("combo") == 3) 5f else 2f) * scale
            c.drawArc(
                x - 42 * scale,
                y - 64 * scale,
                x + 42 * scale,
                y + 9 * scale,
                if (flip) 105f else -85f,
                150f,
                false,
                u.p,
            )
            u.p.style = Paint.Style.FILL
        }
        if (player.optString("stance") == "bulwark") {
            u.p.color = u.alpha(u.mint, 105)
            u.p.style = Paint.Style.STROKE
            u.p.strokeWidth = 1.5f * scale
            c.drawOval(x - 29 * scale, y - 9 * scale, x + 29 * scale, y + 9 * scale, u.p)
            u.p.style = Paint.Style.FILL
        }
    }
}
