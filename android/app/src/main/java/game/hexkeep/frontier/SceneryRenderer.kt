package game.hexkeep.frontier

import android.graphics.*
import game.hexkeep.art.PaintedArt
import kotlin.math.*
import org.json.JSONObject

/** Painted scenery is placed on collision footprints and shares the actors' depth order. */
class SceneryRenderer(private val art: PaintedArt, private val u: UiKit) {
    data class Prop(val frame: Int, val x: Float, val y: Float, val height: Float)

    private var zoneId = -1
    private var props = emptyList<Prop>()
    private val fog = Bitmap.createBitmap(48, 32, Bitmap.Config.ARGB_8888)
    private val fogPixels = IntArray(48 * 32)
    private val paint = Paint(Paint.ANTI_ALIAS_FLAG or Paint.FILTER_BITMAP_FLAG)

    fun prepare(zone: JSONObject, tile: Float): List<Prop> {
        if (zoneId == zone.optInt("id")) return props
        zoneId = zone.optInt("id")
        val out = ArrayList<Prop>()
        for (i in 0 until zone.array("obstacles").length()) {
            val o = zone.array("obstacles").getJSONArray(i)
            for (yy in 0 until o.getInt(3) step 2) for (xx in 0 until o.getInt(2) step 2) {
                val frame =
                    when ((i + xx + yy) % 6) {
                        0 -> 0
                        1 -> 1
                        2 -> 2
                        3 -> 4
                        4 -> 4
                        else -> 6
                    }
                out.add(
                    Prop(
                        frame,
                        (o.getInt(0) + xx + 1f) * tile,
                        (o.getInt(1) + yy + 1.6f).coerceAtMost(
                            (o.getInt(1) + o.getInt(3)).toFloat()
                        ) * tile,
                        if (frame < 2) 155f else 112f,
                    )
                )
            }
        }
        // A physical rim makes the limits of the world visible without enclosing the camera in UI.
        for (x in 0..47 step 2) {
            out.add(Prop(2, (x + .5f) * tile, tile, 78f))
            out.add(Prop(0, (x + .5f) * tile, 32 * tile, 145f))
        }
        for (y in 2..29 step 2) {
            out.add(Prop(0, .5f * tile, (y + .5f) * tile, 155f))
            out.add(Prop(1, 47.5f * tile, (y + .5f) * tile, 155f))
        }
        val spawn = zone.array("spawn")
        out.add(Prop(7, (spawn.optInt(0) - 1f) * tile, (spawn.optInt(1) - 1f) * tile, 94f))
        props = out.sortedBy { it.y }
        return props
    }

    fun floor(c: Canvas, zone: Int, tile: Float) {
        art.frame(c, "floor-${art.biomeNames[zone]}", 0, RectF(0f, 0f, 48 * tile, 32 * tile))
    }

    fun prop(c: Canvas, p: Prop, zone: Int, heroX: Float, heroY: Float) {
        val obscure =
            abs(p.x - heroX) < p.height * .40f && heroY < p.y && heroY > p.y - p.height * .75f
        if (p.frame == 6)
            art.anchored(
                c,
                "props-${art.biomeNames[zone]}",
                2,
                p.x,
                p.y,
                95f,
                false,
                if (obscure) 100 else 255,
            )
        art.anchored(
            c,
            "props-${art.biomeNames[zone]}",
            p.frame,
            p.x,
            p.y,
            p.height,
            false,
            if (obscure) 100 else 255,
        )
    }

    fun fog(c: Canvas, run: JSONObject, tile: Float) {
        val explored = run.array("explored")
        var changed = false
        for (i in fogPixels.indices) {
            val next = if (explored.optBoolean(i)) 0x000d1829 else 0xb00d1829.toInt()
            if (fogPixels[i] != next) {
                fogPixels[i] = next
                changed = true
            }
        }
        if (changed) fog.setPixels(fogPixels, 0, 48, 0, 0, 48, 32)
        c.drawBitmap(fog, null, RectF(0f, 0f, 48 * tile, 32 * tile), paint)
    }

    fun atmosphere(c: Canvas, w: Float, zone: Int, t: Float, reduced: Boolean) {
        if (reduced) return
        for (i in 0..24) {
            val x = ((i * 173.1f + t * (if (zone == 2) 9 else 3)) % w)
            val y = (i * 83.3f + t * (if (zone == 2) 16 else -4)).mod(540f)
            u.circle(
                c,
                x,
                y,
                if (zone == 2) 1.2f else 1.6f,
                u.alpha(if (zone == 2) u.white else u.gold, if (zone == 1) 45 else 90),
            )
        }
    }

    fun close() {
        fog.recycle()
        props = emptyList()
    }
}
