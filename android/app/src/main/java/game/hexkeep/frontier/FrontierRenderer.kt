package game.hexkeep.frontier

import android.graphics.*
import android.os.SystemClock
import game.hexkeep.art.PaintedArt
import kotlin.math.*
import org.json.JSONObject

/** Painted world, depth-sorted scenery and state-driven actors. */
class FrontierRenderer(
    private val u: UiKit,
    private val layers: EquipmentLayers,
    private val art: PaintedArt,
) {
    private val entities = EntityRenderer(u, art)
    private val scenery = SceneryRenderer(art, u)
    private val feel = CombatFeel()
    val director = CutsceneDirector(u)
    private var zoneId = -1
    private var camX = 0f
    private var camY = 0f
    private val tile = 48f

    fun close() {
        scenery.close()
        entities.clear()
    }

    fun draw(c: Canvas, w: Float, data: JSONObject, reduced: Boolean, cinematic: Boolean = false) {
        val f = data.obj("frontier")
        val run = f.optJSONObject("run") ?: return
        val zone = f.array("zones").optJSONObject(run.optInt("zone")) ?: return
        if (zoneId != zone.optInt("id")) {
            zoneId = zone.optInt("id")
            camX = zone.array("spawn").optInt(0) + .5f
            camY = zone.array("spawn").optInt(1) + .5f
            entities.clear()
        }
        val props = scenery.prepare(zone, tile)
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
        scenery.floor(c, zoneId, tile)
        // Objective silhouettes remain legible in every palette; shape and labels supplement
        // colour.
        val beacons = zone.array("beacons")
        for (i in 0 until beacons.length()) {
            val point = beacons.getJSONArray(i)
            val x = (point.getInt(0) + .5f) * tile
            val y = (point.getInt(1) + .5f) * tile
            val lit = run.array("beacons").optBoolean(i)
            u.circle(c, x, y, 28f, u.alpha(if (lit) u.gold else accent, 45))
            art.anchored(c, "props-${art.biomeNames[zoneId]}", if (lit) 9 else 8, x, y, 92f)
            if (lit) {
                u.p.shader =
                    RadialGradient(
                        x,
                        y - 58,
                        40f,
                        intArrayOf(0x65ffd38b, 0x00ffd38b),
                        null,
                        Shader.TileMode.CLAMP,
                    )
                c.drawCircle(x, y - 58, 40f, u.p)
                u.p.shader = null
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
            art.anchored(
                c,
                "props-${art.biomeNames[zoneId]}",
                11,
                x,
                y + if (reduced) 0f else sin(t * 2 + i) * 3,
                40f,
            )
        }
        val rescue = zone.array("rescue")
        if (!run.optBoolean("rescued")) {
            val x = (rescue.optInt(0) + .5f) * tile
            val y = (rescue.optInt(1) + .5f) * tile
            art.anchored(c, "npcs", zoneId + 3, x, y, 82f)
            u.text(c, "VOYAGEUR", x, y + 22, 9f, u.mint, true)
        }
        val explored = run.array("explored")
        val minX = ((centerX - halfW) - 1).toInt().coerceAtLeast(0)
        val maxX = ((centerX + halfW) + 1).toInt().coerceAtMost(47)
        val minY = ((centerY - halfH) - 1).toInt().coerceAtLeast(0)
        val maxY = ((centerY + halfH) + 1).toInt().coerceAtMost(31)
        fun visible(p: JSONObject) =
            cinematic || explored.optBoolean((p.optInt("y") / 256) * 48 + p.optInt("x") / 256)
        val monsters =
            run.array("monsters")
                .objects()
                .filter { visible(it.obj("pos")) }
                .sortedBy { it.obj("pos").optInt("y") }
        val visibleProps =
            props.filter {
                it.x / tile in (minX - 3).toFloat()..(maxX + 3).toFloat() &&
                    it.y / tile in (minY - 1).toFloat()..(maxY + 4).toFloat()
            }
        var pi = 0
        fun sceneryUntil(y: Float) {
            while (pi < visibleProps.size && visibleProps[pi].y <= y) {
                scenery.prop(c, visibleProps[pi++], zoneId, tx * tile, ty * tile)
            }
        }
        var heroDrawn = false
        for (m in monsters) {
            val p = m.obj("pos")
            val x = p.optInt("x") / 256f * tile
            val y = p.optInt("y") / 256f * tile
            if (!heroDrawn && y > ty * tile) {
                sceneryUntil(ty * tile)
                layers.hero(c, tx * tile, ty * tile, 1f, player, data.obj("journey"), t)
                heroDrawn = true
            }
            sceneryUntil(y)
            if (
                x / tile in (minX - 3).toFloat()..(maxX + 3).toFloat() &&
                    y / tile in (minY - 1).toFloat()..(maxY + 4).toFloat()
            )
                entities.monster(c, m, x, y, t, zoneId)
        }
        if (!heroDrawn) {
            sceneryUntil(ty * tile)
            layers.hero(c, tx * tile, ty * tile, 1f, player, data.obj("journey"), t)
        }
        sceneryUntil(Float.MAX_VALUE)
        if (!cinematic) scenery.fog(c, run, tile)
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
        scenery.atmosphere(c, w, zoneId, t, reduced)
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
        val boss =
            run.array("monsters").objects().firstOrNull {
                it.optInt("kind") == 4 && it.optBoolean("active") && it.optInt("hp") > 0
            }
        val bossNear =
            boss?.obj("pos")?.let { p ->
                val pp = player.obj("pos")
                hypot(
                    (p.optInt("x") - pp.optInt("x")).toFloat(),
                    (p.optInt("y") - pp.optInt("y")).toFloat(),
                ) < 8 * 256
            } ?: false
        if (bossNear) {
            art.fit(c, "abilities", 7, RectF(w - 79, 86f, w - 35, 130f))
            u.hits.add(UiKit.Hit("f:atlas", RectF(w - 85, 80f, w - 29, 136f)))
        } else minimap(c, w - 216, 85f, 184f, 122f, zone, run)
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
        art.fit(
            c,
            "abilities",
            when (id) {
                "attack" -> 0
                "dash" -> 1
                else -> 2
            },
            RectF(x - r + 6, y - r + 6, x + r - 6, y + r - 6),
            if (cd == 0) 255 else 80,
        )
        if (cd > 0)
            u.text(
                c,
                String.format(java.util.Locale.US, "%.1f", cd / 30f),
                x,
                y + 8,
                24f,
                u.white,
                true,
                u.bold,
            )
        else if (id == "attack") u.text(c, symbol, x + r - 8, y + r - 3, 15f, u.gold, true, u.bold)
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
        art.frame(c, "floor-${art.biomeNames[z.optInt("id")]}", 0, RectF(x, y, x + w, y + h), 170)
        val explored = run.array("explored")
        val sx = w / 48
        val sy = h / 32
        for (yy in 0..31) for (xx in 0..47) if (!explored.optBoolean(yy * 48 + xx))
            u.rect(c, x + xx * sx, y + yy * sy, sx + .2f, sy + .2f, 0xdc0d1829.toInt(), 0f)
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
