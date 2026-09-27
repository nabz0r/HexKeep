package game.hexkeep.frontier

import android.graphics.*
import kotlin.math.*
import org.json.JSONObject

class AtlasScreen(private val u: UiKit, private val art: game.hexkeep.art.PaintedArt) {
    var selected = 0

    fun draw(c: Canvas, w: Float, data: JSONObject, t: Float) {
        val f = data.obj("frontier")
        val zones = f.array("zones").objects()
        if (zones.size < 3) return
        u.header(c, w, "ATLAS DES CONFINS  /  CHAPITRE II", "Le monde se souvient.")
        val right = w - 342
        val map = RectF(32f, 104f, right - 20, 444f)
        u.rect(c, map, 0xff112830.toInt(), 16f, u.alpha(u.mint, 50))
        c.save()
        c.clipRect(map)
        art.frame(c, "world-atlas", 0, map)
        u.rect(c, map, 0x3206131c, 0f)
        fun point(z: JSONObject) =
            PointF(
                map.left + map.width() * z.optDouble("map_x").toFloat(),
                map.top + map.height() * z.optDouble("map_y").toFloat(),
            )
        for (i in 0..1) {
            val a = point(zones[i])
            val b = point(zones[i + 1])
            u.p.color = u.alpha(u.gold, 100)
            u.p.style = Paint.Style.STROKE
            u.p.strokeWidth = 2f
            u.p.pathEffect = DashPathEffect(floatArrayOf(5f, 7f), 0f)
            c.drawLine(a.x, a.y, b.x, b.y, u.p)
            u.p.pathEffect = null
            u.p.style = Paint.Style.FILL
        }
        zones.forEachIndexed { i, z ->
            val p = point(z)
            val color = u.color(z.array("colors").optString(2))
            val unlocked = z.optBoolean("unlocked")
            u.circle(c, p.x, p.y, 29f, 0xdf101a23.toInt())
            u.circle(
                c,
                p.x,
                p.y,
                29f,
                if (selected == i) u.gold else u.alpha(u.gold, 120),
                if (selected == i) 2.5f else 1f,
            )
            art.fit(
                c,
                "props-${art.biomeNames[i]}",
                if (z.optBoolean("complete")) 9 else 8,
                RectF(p.x - 22, p.y - 31, p.x + 22, p.y + 15),
                if (unlocked) 255 else 105,
            )
            u.text(
                c,
                if (z.optBoolean("complete")) "✓" else if (unlocked) "0${i+1}" else "×",
                p.x,
                p.y + 23,
                12f,
                u.white,
                true,
            )
            u.rect(
                c,
                p.x - map.width() * .19f,
                p.y + 35,
                map.width() * .38f,
                27f,
                0xe5101a23.toInt(),
                3f,
            )
            u.text(
                c,
                z.optString("name"),
                p.x,
                p.y + 55,
                15f,
                if (unlocked) u.white else u.muted,
                true,
                u.bold,
                map.width() * .41f,
            )
            u.hits.add(UiKit.Hit("zone:$i", RectF(p.x - 80, p.y - 42, p.x + 80, p.y + 68)))
        }
        c.restore()
        u.text(c, "LE REFUGE", map.left + 24, map.top + 32, 11f, u.gold, font = u.bold)
        u.rect(c, map.left + 16, map.bottom - 38, 310f, 26f, 0xdf101a23.toInt(), 3f)
        u.text(
            c,
            "3 régions • 12 missions • un même serment",
            map.left + 24,
            map.bottom - 20,
            13f,
            u.muted,
        )
        val z = zones[selected]
        val x = right
        u.rect(c, x - 12, 104f, 326f, 308f, 0xf0111923.toInt(), 8f, u.alpha(u.gold, 80))
        u.text(c, "RÉGION 0${selected+1}", x, 130f, 11f, u.gold, font = u.bold)
        u.text(c, z.optString("name"), x, 168f, 26f, u.white, font = u.serif, maxWidth = 310f)
        u.text(c, z.optString("biome"), x, 198f, 15f, u.mint)
        u.wrap(c, z.optString("overview"), x, 234f, 302f, 16f, u.muted, 24f, 5)
        u.text(c, "Exploration ${z.optInt("exploration")}%", x, 382f, 14f, u.white)
        u.rect(c, x, 397f, 302f, 4f, u.alpha(u.white, 25), 2f)
        u.rect(c, x, 397f, 302f * z.optInt("exploration") / 100, 4f, u.mint, 2f)
        val run = f.optJSONObject("run")
        val unlocked = z.optBoolean("unlocked")
        u.button(
            c,
            if (run != null) "f:resume" else "f:travel:$selected",
            if (run != null) "Reprendre l’expédition"
            else if (unlocked) "Voyager vers cette région  ›" else "Vaincre le gardien précédent",
            x,
            425f,
            302f,
            56f,
            true,
            unlocked || run != null,
        )
        u.button(c, "f:journal", "Journal des missions", 32f, 464f, 214f)
        u.button(c, "inventory", "Sac & équipement", 260f, 464f, 207f)
        if (run != null)
            u.text(
                c,
                "Une expédition est en cours. Termine-la ou rentre au refuge pour voyager.",
                490f,
                493f,
                12f,
                u.muted,
                maxWidth = w - 525,
            )
    }
}
