package game.hexkeep

import android.graphics.*
import androidx.test.platform.app.InstrumentationRegistry
import game.hexkeep.art.PaintedArt
import game.hexkeep.core.Engine
import game.hexkeep.frontier.EquipmentLayers
import game.hexkeep.frontier.FrontierUi
import game.hexkeep.frontier.UiKit
import java.io.File
import java.util.zip.CRC32
import org.json.JSONArray
import org.json.JSONObject
import org.junit.Assert.*
import org.junit.Test

/** Renderer fixtures complement the live touch journeys; they never alter a player's vault. */
class ArtDirectionTest {
    private val context = InstrumentationRegistry.getInstrumentation().targetContext

    private fun save(bitmap: Bitmap, name: String) {
        File(context.getExternalFilesDir(null), "v08-$name.png").outputStream().use {
            bitmap.compress(Bitmap.CompressFormat.PNG, 100, it)
        }
    }

    @Test
    fun atlasTransparencyFramesAndMemoryBudget() {
        val art = PaintedArt(context.assets)
        val files = context.assets.list("art/v08")!!.filter { it.endsWith(".png") }
        assertEquals(26, files.size)
        files.forEach { file ->
            val name = file.removeSuffix(".png")
            val s = art.sheet(name)
            val expected =
                when {
                    name.startsWith("floor-") || name == "refuge" || name == "world-atlas" -> 1
                    name == "npcs" -> 6
                    name == "abilities" -> 8
                    else -> 12
                }
            assertEquals(name, expected, s.frames.size)
            assertTrue(name, s.frames.all { it.width() > 24 && it.height() > 24 })
            if (expected > 1) {
                var transparent = 0
                var opaque = 0
                for (y in 0 until s.bitmap.height step 13) for (x in
                    0 until s.bitmap.width step 13) {
                    val a = Color.alpha(s.bitmap.getPixel(x, y))
                    if (a < 10) transparent++
                    if (a > 230) opaque++
                }
                assertTrue("$name has transparent gutters", transparent > 200)
                assertTrue("$name has opaque painted subjects", opaque > 200)
            }
            val m = art.metrics()
            assertTrue(m.toString(), m.getLong("resident_bytes") <= m.getLong("budget_bytes"))
        }
        File(context.getExternalFilesDir(null), "v08-art-memory.json")
            .writeText(art.metrics().toString(2))
        art.close()
    }

    @Test
    fun allRegionsHeroesAndMenusRenderWithinSafeCanvas() {
        val engine = Engine.offline("")
        engine.uiAction("home")
        engine.uiAction("f:travel:0")
        engine.uiAction("f:skip")
        val source = JSONObject(engine.presentation(584))
        assertNotNull(source.getJSONObject("frontier").getJSONObject("run"))
        val art = PaintedArt(context.assets)
        val ui = FrontierUi(art, {}, { _, _ -> }, {}, {}, {})
        for (width in listOf(960, 1200, 1440)) {
            val bitmap = Bitmap.createBitmap(width, 540, Bitmap.Config.ARGB_8888)
            val c = Canvas(bitmap)
            for (zone in 0..2) {
                val data = JSONObject(source.toString()).put("screen", 52).put("realm", zone)
                val frontier = data.getJSONObject("frontier")
                val run = frontier.getJSONObject("run").put("zone", zone)
                val z = frontier.getJSONArray("zones").getJSONObject(zone)
                val boss = z.getJSONArray("boss")
                val px = (boss.getInt(0) - 3) * 256
                val py = boss.getInt(1) * 256
                run.getJSONObject("player").getJSONObject("pos").put("x", px).put("y", py)
                run.put("explored", JSONArray((0 until 1536).map { true }))
                run.put("beacons", JSONArray(listOf(true, true, true)))
                val enemies = run.getJSONArray("monsters")
                for (i in 0 until enemies.length()) {
                    val m = enemies.getJSONObject(i)
                    val kind = m.getInt("kind")
                    if (i < 4 || kind == 4) {
                        m.getJSONObject("pos")
                            .put("x", px + (if (kind == 4) 4 else i - 1) * 256)
                            .put("y", py + (if (kind == 4) 0 else -2) * 256)
                        m.put("active", true).put("state", "idle")
                    } else m.getJSONObject("pos").put("x", 256).put("y", 256)
                }
                repeat(20) { ui.draw(c, width.toFloat(), data, true) }
                checkHits(ui, width)
                if (width == 1200) save(bitmap, "renderer-${art.biomeNames[zone]}")
            }
            for (screen in listOf(7, 40, 50, 51, 56)) {
                val data = JSONObject(source.toString()).put("screen", screen)
                ui.draw(c, width.toFloat(), data, true)
                checkHits(ui, width)
                if (width == 1200) save(bitmap, "renderer-screen-$screen")
            }
            bitmap.recycle()
        }
        assertTrue(art.metrics().getLong("resident_bytes") <= art.metrics().getLong("budget_bytes"))
        ui.close()
        art.close()
    }

    private fun checkHits(ui: FrontierUi, width: Int) {
        val buttons = ui.metrics().getJSONArray("buttons")
        for (i in 0 until buttons.length()) {
            val b = buttons.getJSONObject(i)
            assertTrue(
                b.toString(),
                b.getDouble("left") >= 0 &&
                    b.getDouble("top") >= 0 &&
                    b.getDouble("right") <= width + 1 &&
                    b.getDouble("bottom") <= 541,
            )
        }
    }

    @Test
    fun equipmentAndAnimationChangePaintedPixels() {
        val art = PaintedArt(context.assets)
        val layers = EquipmentLayers(UiKit(art), art)
        val bitmap = Bitmap.createBitmap(240, 260, Bitmap.Config.ARGB_8888)
        val c = Canvas(bitmap)
        val journey =
            JSONObject()
                .put("items", JSONArray())
                .put("equipped", JSONArray(listOf(0, 0, 0, 0, 0, 0)))
        val player =
            JSONObject().put("state", "idle").put("facing", JSONObject().put("x", 1).put("y", 1))
        fun fingerprint(): Long {
            bitmap.eraseColor(Color.TRANSPARENT)
            layers.hero(c, 120f, 250f, 2.7f, player, journey, 0f)
            val crc = CRC32()
            for (y in 0 until 260) for (x in 0 until 240) {
                val p = bitmap.getPixel(x, y)
                crc.update(p)
                crc.update(p ushr 8)
                crc.update(p ushr 16)
                crc.update(p ushr 24)
            }
            return crc.value
        }
        val base = fingerprint()
        for (slot in 0..5) {
            journey.put(
                "items",
                JSONArray()
                    .put(
                        JSONObject()
                            .put("id", 1)
                            .put("slot", slot)
                            .put("catalog", slot * 12 + 2)
                            .put("motif", 1)
                    ),
            )
            val eq = JSONArray(listOf(0, 0, 0, 0, 0, 0))
            eq.put(slot, 1)
            journey.put("equipped", eq)
            assertNotEquals("visible slot $slot", base, fingerprint())
        }
        val front = fingerprint()
        player.getJSONObject("facing").put("y", -1)
        assertNotEquals(front, fingerprint())
        player.put("state", "strike")
        assertNotEquals(front, fingerprint())
        bitmap.recycle()
        art.close()
    }
}
