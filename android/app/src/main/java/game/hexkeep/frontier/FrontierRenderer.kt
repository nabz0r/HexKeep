package game.hexkeep.frontier

import android.graphics.*
import android.os.SystemClock
import kotlin.math.*
import org.json.JSONObject

/**
 * One cached procedural terrain bitmap (6 MiB), bounded particles, no per-frame bitmap allocation.
 */
class FrontierRenderer(private val u: UiKit, private val layers: EquipmentLayers) {
    private val entities = EntityRenderer(u)
    private val feel = CombatFeel()
    val director = CutsceneDirector(u)
    private var terrain: Bitmap? = null
    private var zoneId = -1
    private var camX = 0f
    private var camY = 0f
    private val tile = 32f

    fun close() {
        terrain?.recycle()
        terrain = null
        zoneId = -1
    }

    private fun terrain(zone: JSONObject): Bitmap {
        if (zoneId == zone.optInt("id") && terrain != null) return terrain!!
        terrain?.recycle()
        zoneId = zone.optInt("id")
        camX = zone.array("spawn").optInt(0) + .5f
        camY = zone.array("spawn").optInt(1) + .5f
        val bmp = Bitmap.createBitmap(48 * 32, 32 * 32, Bitmap.Config.ARGB_8888)
        val c = Canvas(bmp)
        val colors = zone.array("colors")
        val base = u.color(colors.optString(0))
        val surface = u.color(colors.optString(1))
        val accent = u.color(colors.optString(2))
        c.drawColor(base)
        for (y in 0..31) for (x in 0..47) {
            val seed = (x * 73856093) xor (y * 19349663) xor (zoneId * 83492791)
            val xx = x * tile
            val yy = y * tile
            u.rect(c, xx, yy, tile, tile, u.alpha(surface, 22 + (seed and 31)), 0f)
            if ((seed and 7) == 0) u.circle(c, xx + 9, yy + 13, 2f, u.alpha(accent, 50))
            if (zoneId == 0 && (seed and 15) < 4) {
                u.line(c, xx + 5, yy + 20, xx + 19, yy + 20, u.alpha(accent, 25))
                u.line(c, xx + 11, yy + 24, xx + 26, yy + 24, u.alpha(accent, 18))
            }
            if (zoneId == 1) {
                u.line(c, xx + 3, yy + 24, xx + 28, yy + 20, u.alpha(u.gold, 20))
            }
            if (zoneId == 2 && (seed and 3) == 0) {
                u.polygon(c, xx + 17, yy + 15, 3f, 4, u.alpha(u.white, 70))
            }
        }
        // Pale trails between the spawn and the three shrines add navigational structure.
        val points =
            listOf(zone.array("spawn")) +
                zone.array("beacons").let { a -> (0 until a.length()).map { a.getJSONArray(it) } } +
                listOf(zone.array("boss"))
        for (i in 1 until points.size) {
            val a = points[i - 1]
            val b = points[i]
            val path = Path()
            val ax = (a.optInt(0) + .5f) * tile
            val ay = (a.optInt(1) + .5f) * tile
            val bx = (b.optInt(0) + .5f) * tile
            val by = (b.optInt(1) + .5f) * tile
            path.moveTo(ax, ay)
            path.cubicTo((ax + bx) / 2, ay, (ax + bx) / 2, by, bx, by)
            u.p.color = u.alpha(accent, 15)
            u.p.strokeWidth = 34f
            u.p.style = Paint.Style.STROKE
            c.drawPath(path, u.p)
            u.p.color = u.alpha(u.gold, 20)
            u.p.strokeWidth = 2f
            c.drawPath(path, u.p)
            u.p.style = Paint.Style.FILL
        }
        val obs = zone.array("obstacles")
        for (i in 0 until obs.length()) {
            val r = obs.getJSONArray(i)
            val x = r.getInt(0) * tile
            val y = r.getInt(1) * tile
            val w = r.getInt(2) * tile
            val h = r.getInt(3) * tile
            u.rect(c, x + 8, y + 13, w, h, 0x60000000, 18f)
            u.rect(c, x, y, w, h, surface, 16f, u.alpha(accent, 80))
            for (yy in 0 until r.getInt(3)) for (xx in 0 until r.getInt(2)) {
                val px = x + xx * tile + 16
                val py = y + yy * tile + 16
                when (zoneId) {
                    0 -> {
                        u.polygon(c, px, py, 24f, 6, u.alpha(base, 190), 30f)
                        u.polygon(c, px - 3, py - 5, 19f, 5, u.alpha(accent, 55), -90f)
                        u.line(c, px, py + 8, px + 3, py + 18, u.gold, 2f)
                    }
                    1 -> {
                        u.polygon(c, px, py, 20f, 4, u.alpha(accent, 100), -65f)
                        u.polygon(c, px - 3, py - 4, 13f, 3, u.alpha(u.white, 65), -65f)
                    }
                    else -> {
                        u.polygon(c, px, py, 22f, 3, u.alpha(accent, 130))
                        u.polygon(c, px, py - 7, 13f, 3, u.alpha(u.white, 150))
                    }
                }
            }
        }
        for (y in 0..31) for (x in 0..47) if (x == 0 || x == 47 || y == 0 || y == 31)
            u.rect(c, x * tile, y * tile, tile, tile, 0xff070f1a.toInt(), 0f)
        terrain = bmp
        return bmp
    }

    fun draw(c: Canvas, w: Float, data: JSONObject, reduced: Boolean, cinematic: Boolean = false) {
        val f = data.obj("frontier")
        val run = f.optJSONObject("run") ?: return
        val zone = f.array("zones").optJSONObject(run.optInt("zone")) ?: return
        val bmp = terrain(zone)
        val player = run.obj("player")
        val pos = player.obj("pos")
        val colors = zone.array("colors")
        val accent = u.color(colors.optString(2))
        val t = run.optInt("tick") / 30f
        val scene = f.optJSONObject("scene")
        val camera = if (cinematic && scene != null) director.camera(scene, reduced) else null
        val tx = pos.optInt("x") / 256f
        val ty = pos.optInt("y") / 256f
        if (camera == null) {
            camX += (tx - camX) * .25f
            camY += (ty - camY) * .25f
        }
        val zoom = camera?.zoom ?: 1.05f
        val halfW = w / 2 / tile / zoom
        val halfH = 540 / 2f / tile / zoom
        val centerX =
            (camera?.x ?: camX).coerceIn(halfW.coerceAtMost(24f), 48 - halfW.coerceAtMost(24f))
        val centerY = (camera?.y ?: camY).coerceIn(halfH, 32 - halfH)
        val originX = w / 2 - centerX * tile * zoom
        val originY = 270 - centerY * tile * zoom
        val shake = feel.offset(run, SystemClock.uptimeMillis(), reduced || cinematic)
        c.save()
        c.clipRect(0f, 0f, w, 540f)
        c.translate(originX + shake.first, originY + shake.second)
        c.scale(zoom, zoom)
        u.p.shader = null
        u.p.color = Color.WHITE
        u.p.alpha = 255
        c.drawBitmap(bmp, 0f, 0f, u.p)
        // Objective silhouettes remain legible in every palette; shape and labels supplement
        // colour.
        val beacons = zone.array("beacons")
        for (i in 0 until beacons.length()) {
            val point = beacons.getJSONArray(i)
            val x = (point.getInt(0) + .5f) * tile
            val y = (point.getInt(1) + .5f) * tile
            val lit = run.array("beacons").optBoolean(i)
            u.circle(c, x, y, 28f, u.alpha(if (lit) u.gold else accent, 45))
            u.polygon(c, x, y, 24f, 6, accent, 30f, 1f)
            u.rect(c, x - 7, y - 37, 14f, 38f, 0xff718587.toInt(), 2f)
            u.polygon(c, x, y - 42, 10f, 4, if (lit) u.gold else u.muted)
            if (lit) {
                u.circle(c, x, y - 42, 18f, u.alpha(u.gold, 40))
                u.line(c, x, y - 54, x, y - 84, u.alpha(u.gold, 110), 2f)
            }
            u.text(
                c,
                if (lit) "BALISE ÉVEILLÉE" else "BALISE ${i+1}",
                x,
                y + 38,
                9f,
                if (lit) u.gold else u.white,
                true,
            )
        }
        val relics = zone.array("relics")
        for (i in 0 until relics.length()) if (!run.array("relics").optBoolean(i)) {
            val point = relics.getJSONArray(i)
            val x = (point.getInt(0) + .5f) * tile
            val y = (point.getInt(1) + .5f) * tile
            u.circle(c, x, y, 18f, u.alpha(u.gold, 30))
            u.polygon(c, x, y - 8 + if (reduced) 0f else sin(t * 2 + i) * 3, 9f, 4, u.gold)
            u.polygon(c, x, y - 8, 4f, 4, u.ink)
        }
        val rescue = zone.array("rescue")
        if (!run.optBoolean("rescued")) {
            val x = (rescue.optInt(0) + .5f) * tile
            val y = (rescue.optInt(1) + .5f) * tile
            u.polygon(c, x, y - 9, 15f, 3, u.mint)
            u.circle(c, x, y - 29, 6f, u.white)
            u.text(c, "VOYAGEUR", x, y + 22, 9f, u.mint, true)
        }
        val explored = run.array("explored")
        val minX = ((centerX - halfW) - 1).toInt().coerceAtLeast(0)
        val maxX = ((centerX + halfW) + 1).toInt().coerceAtMost(47)
        val minY = ((centerY - halfH) - 1).toInt().coerceAtLeast(0)
        val maxY = ((centerY + halfH) + 1).toInt().coerceAtMost(31)
        if (!cinematic)
            for (y in minY..maxY) for (x in minX..maxX) if (!explored.optBoolean(y * 48 + x))
                u.rect(c, x * tile, y * tile, tile + .5f, tile + .5f, 0xa5081724.toInt(), 0f)
        fun visible(p: JSONObject) =
            cinematic || explored.optBoolean((p.optInt("y") / 256) * 48 + p.optInt("x") / 256)
        val monsters =
            run.array("monsters")
                .objects()
                .filter { visible(it.obj("pos")) }
                .sortedBy { it.obj("pos").optInt("y") }
        var heroDrawn = false
        for (m in monsters) {
            val p = m.obj("pos")
            val x = p.optInt("x") / 256f * tile
            val y = p.optInt("y") / 256f * tile
            if (!heroDrawn && p.optInt("y") > pos.optInt("y")) {
                layers.hero(c, tx * tile, ty * tile, 1f, player, data.obj("journey"), t)
                heroDrawn = true
            }
            if (
                x / tile in (minX - 2).toFloat()..(maxX + 2).toFloat() &&
                    y / tile in (minY - 2).toFloat()..(maxY + 2).toFloat()
            )
                entities.monster(c, m, x, y, t, if (m.optInt("kind") == 3) u.red else accent)
        }
        if (!heroDrawn) layers.hero(c, tx * tile, ty * tile, 1f, player, data.obj("journey"), t)
        for (b in run.array("projectiles").objects()) {
            val p = b.obj("pos")
            val x = p.optInt("x") / 256f * tile
            val y = p.optInt("y") / 256f * tile
            u.circle(c, x, y, 8f, u.alpha(u.red, 65))
            u.polygon(c, x, y, 4f, 4, u.red)
        }
        for (e in run.array("effects").objects()) {
            val p = e.obj("pos")
            val x = p.optInt("x") / 256f * tile
            val y = p.optInt("y") / 256f * tile
            val life = e.optInt("life")
            val color = if (e.optInt("amount") < 0) u.red else u.gold
            val alpha = life * 10
            if (e.optInt("amount") != 0)
                u.text(
                    c,
                    abs(e.optInt("amount")).toString(),
                    x,
                    y - 45 - (24 - life) * .8f,
                    if (e.optInt("kind") == 2) 21f else 16f,
                    u.alpha(color, alpha),
                    true,
                    u.bold,
                )
            else
                u.circle(
                    c,
                    x,
                    y - 12,
                    (24 - life) * if (e.optInt("kind") == 3) 5f else 2f,
                    u.alpha(color, alpha / 2),
                    if (e.optInt("kind") == 2) 3f else 1f,
                )
        }
        c.restore()
        if (cinematic && scene != null) director.overlay(c, w, scene, reduced)
    }

    fun hud(c: Canvas, w: Float, data: JSONObject, mx: Float, my: Float, origin: PointF) {
        val f = data.obj("frontier")
        val run = f.obj("run")
        val player = run.obj("player")
        val zone = f.array("zones").optJSONObject(run.optInt("zone")) ?: return
        u.rect(c, 22f, 16f, 304f, 66f, 0xe70b1c29.toInt(), 12f, u.alpha(u.gold, 60))
        u.text(c, zone.optString("name"), 36f, 38f, 15f, u.white, font = u.bold, maxWidth = 270f)
        u.rect(c, 36f, 52f, 205f, 7f, u.alpha(u.white, 25), 4f)
        u.rect(
            c,
            36f,
            52f,
            205f * player.optInt("hp") / player.optInt("max_hp", 1).coerceAtLeast(1),
            7f,
            u.mint,
            4f,
        )
        u.text(c, "${player.optInt("hp")}", 258f, 63f, 16f, u.white)
        u.button(c, "inventory", "Sac", w - 375, 18f, 92f)
        u.button(c, "f:atlas", "Atlas", w - 271, 18f, 118f)
        u.button(c, "f:pause", "Ⅱ", w - 141, 18f, 109f)
        val quest =
            f.array("quests").objects().firstOrNull {
                it.optBoolean("tracked") && !it.optBoolean("claimed")
            }
        u.rect(c, 22f, 93f, 304f, 66f, 0xda0b1c29.toInt(), 10f)
        u.text(
            c,
            quest?.optString("title") ?: "Les trois balises",
            36f,
            116f,
            14f,
            u.gold,
            maxWidth = 270f,
        )
        u.text(
            c,
            if (quest != null)
                "${quest.optInt("progress")} / ${quest.optInt("target")} · ${if(quest.optBoolean("ready"))"récompense disponible"else"mission suivie"}"
            else "Rallume les balises, puis affronte le gardien.",
            36f,
            140f,
            12f,
            u.muted,
            maxWidth = 270f,
        )
        minimap(c, w - 216, 85f, 184f, 122f, zone, run)
        val boss =
            run.array("monsters").objects().firstOrNull {
                it.optInt("kind") == 4 && it.optBoolean("active") && it.optInt("hp") > 0
            }
        if (boss != null) {
            val bw = min(340f, w - 690)
            val x = (w - bw) / 2
            u.rect(c, x - 10, 77f, bw + 20, 54f, 0xde0b1c29.toInt(), 10f)
            u.text(
                c,
                "${zone.optString("boss_name")} · ${boss.optInt("phase")+1}/3",
                x + bw / 2,
                98f,
                12f,
                u.white,
                true,
                maxWidth = bw,
            )
            u.rect(c, x, 113f, bw, 5f, u.alpha(u.red, 45), 3f)
            u.rect(c, x, 113f, bw * boss.optInt("hp") / boss.optInt("max_hp", 1), 5f, u.red, 3f)
        }
        u.circle(c, origin.x, origin.y, 48f, 0x883a5865.toInt())
        u.circle(c, origin.x, origin.y, 48f, u.alpha(u.mint, 90), 1f)
        u.circle(c, origin.x + mx * 29, origin.y + my * 29, 22f, 0xbb99b8ba.toInt())
        u.text(c, "DÉPLACER", origin.x, origin.y + 70f, 10f, u.muted, true)
        ability(c, "attack", "ATTAQUER", w - 98, 338f, 44f, 0, "${player.optInt("combo")+1}")
        ability(c, "dash", "ESQUIVE", w - 90f, 446f, 34f, player.optInt("dash_cd"), "◇")
        ability(c, "skill", "ÉCLAT", w - 188, 438f, 36f, player.optInt("skill_cd"), "✦")
        u.button(
            c,
            "f:heal",
            "+ ${player.optInt("flasks")}",
            w - 310,
            432f,
            64f,
            56f,
            enabled = player.optInt("flasks") > 0,
        )
        val stance =
            when (player.optString("stance")) {
                "assault" -> "Assaut · +30% dégâts"
                "bulwark" -> "Rempart · −35% reçus"
                else -> "Équilibre · polyvalent"
            }
        u.button(c, "f:stance", stance, 232f, 468f, 205f, 48f)
        u.button(c, "f:journal", "Missions", 451f, 468f, 126f)
        val interaction = f.optJSONObject("interaction")
        if (interaction != null)
            u.button(
                c,
                "f:interact",
                interaction.optString("label"),
                w / 2 - 150,
                378f,
                300f,
                54f,
                true,
            )
        u.text(c, "3 coups rapides → 1 coup lourd", w / 2, 530f, 10f, u.muted, true)
    }

    private fun ability(
        c: Canvas,
        id: String,
        label: String,
        x: Float,
        y: Float,
        r: Float,
        cd: Int,
        symbol: String,
    ) {
        u.circle(c, x, y, r, 0xe5193341.toInt())
        u.circle(c, x, y, r, if (cd == 0) u.gold else u.muted, 1.5f)
        u.text(
            c,
            if (cd == 0) symbol else String.format(java.util.Locale.US, "%.1f", cd / 30f),
            x,
            y + 9,
            26f,
            if (cd == 0) u.gold else u.muted,
            true,
            u.serif,
        )
        u.text(c, label, x, y + r + 17, 10f, u.white, true)
        u.hits.add(UiKit.Hit(id, RectF(x - r - 4, y - r - 4, x + r + 4, y + r + 4)))
    }

    private fun minimap(
        c: Canvas,
        x: Float,
        y: Float,
        w: Float,
        h: Float,
        z: JSONObject,
        run: JSONObject,
    ) {
        u.rect(c, x - 6, y - 6, w + 12, h + 12, 0xe4091926.toInt(), 10f, u.alpha(u.gold, 70))
        val explored = run.array("explored")
        val sx = w / 48
        val sy = h / 32
        for (yy in 0..31) for (xx in 0..47) if (explored.optBoolean(yy * 48 + xx))
            u.rect(c, x + xx * sx, y + yy * sy, sx + .2f, sy + .2f, 0xff38535d.toInt(), 0f)
        val obs = z.array("obstacles")
        for (i in 0 until obs.length()) {
            val o = obs.getJSONArray(i)
            u.rect(
                c,
                x + o.getInt(0) * sx,
                y + o.getInt(1) * sy,
                o.getInt(2) * sx,
                o.getInt(3) * sy,
                0x70000000,
                0f,
            )
        }
        val beacons = z.array("beacons")
        for (i in 0 until beacons.length()) {
            val p = beacons.getJSONArray(i)
            u.polygon(
                c,
                x + (p.getInt(0) + .5f) * sx,
                y + (p.getInt(1) + .5f) * sy,
                4f,
                4,
                if (run.array("beacons").optBoolean(i)) u.gold else u.white,
            )
        }
        val p = run.obj("player").obj("pos")
        u.circle(c, x + p.optInt("x") / 256f * sx, y + p.optInt("y") / 256f * sy, 3.5f, u.mint)
        val rescue = z.array("rescue")
        if (!run.optBoolean("rescued"))
            u.circle(
                c,
                x + (rescue.optInt(0) + .5f) * sx,
                y + (rescue.optInt(1) + .5f) * sy,
                2f,
                u.mint,
            )
        u.hits.add(UiKit.Hit("f:atlas", RectF(x - 6, y - 6, x + w + 6, y + h + 6)))
    }
}
