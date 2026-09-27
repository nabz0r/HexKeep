package game.hexkeep

import android.content.Intent
import android.graphics.Bitmap
import android.os.SystemClock
import android.view.InputDevice
import android.view.MotionEvent
import androidx.test.core.app.ActivityScenario
import androidx.test.platform.app.InstrumentationRegistry
import game.hexkeep.core.Engine
import java.io.File
import kotlin.math.*
import org.json.JSONObject
import org.junit.*
import org.junit.Assert.*

/** v0.7 acceptance uses real injected fingers. The player's vault is restored after each case. */
class FrontierTest {
    private val test = InstrumentationRegistry.getInstrumentation()
    private lateinit var activity: MainActivity
    private lateinit var engine: Engine
    private val file
        get() = File(test.targetContext.filesDir, "state.hk")

    private var original: ByteArray? = null
    private var down = 0L
    private var left = 0f
    private var top = 0f
    private var scale = 1f
    private var width = 960f

    @Before
    fun freshJourney() {
        original = if (file.exists()) file.readBytes() else null
        Vault(test.targetContext).write(Engine.offline("").snapshot())
    }

    @After
    fun restoreJourney() {
        File(file.path + ".bak").delete()
        if (original == null) file.delete() else file.writeBytes(original!!)
    }

    private fun launch(): ActivityScenario<MainActivity> {
        val s =
            ActivityScenario.launch<MainActivity>(
                Intent(test.targetContext, MainActivity::class.java)
            )
        SystemClock.sleep(1200)
        s.onActivity {
            activity = it
            engine = it.engine
        }
        val v = metrics().getJSONObject("viewport")
        left = v.getDouble("left").toFloat()
        top = v.getDouble("top").toFloat()
        scale = v.getDouble("scale").toFloat()
        width = v.getDouble("width").toFloat()
        return s
    }

    private fun state() = JSONObject(engine.presentation(584))

    private fun metrics(): JSONObject {
        var result = JSONObject()
        test.runOnMainSync { result = JSONObject(activity.renderMetrics()) }
        return result
    }

    private fun hits() = metrics().getJSONObject("frontier").getJSONArray("buttons")

    private fun hit(id: String): JSONObject {
        val deadline = SystemClock.uptimeMillis() + 4000
        while (SystemClock.uptimeMillis() < deadline) {
            val list = hits()
            for (i in 0 until list.length()) if (list.getJSONObject(i).getString("id") == id)
                return list.getJSONObject(i)
            SystemClock.sleep(60)
        }
        error("Missing UI target $id: ${metrics()}")
    }

    private fun point(id: String): Pair<Float, Float> {
        val h = hit(id)
        return (h.getDouble("left").toFloat() + h.getDouble("right").toFloat()) / 2 to
            (h.getDouble("top").toFloat() + h.getDouble("bottom").toFloat()) / 2
    }

    private fun event(action: Int, points: List<Pair<Float, Float>>) {
        val props =
            points.indices
                .map {
                    MotionEvent.PointerProperties().apply {
                        id = it
                        toolType = MotionEvent.TOOL_TYPE_FINGER
                    }
                }
                .toTypedArray()
        val coords =
            points
                .map {
                    MotionEvent.PointerCoords().apply {
                        x = left + it.first * scale
                        y = top + it.second * scale
                        pressure = 1f
                        size = 1f
                    }
                }
                .toTypedArray()
        val e =
            MotionEvent.obtain(
                down,
                SystemClock.uptimeMillis(),
                action,
                points.size,
                props,
                coords,
                0,
                0,
                1f,
                1f,
                0,
                0,
                InputDevice.SOURCE_TOUCHSCREEN,
                0,
            )
        assertTrue(test.uiAutomation.injectInputEvent(e, true))
        e.recycle()
    }

    private fun tapAt(p: Pair<Float, Float>) {
        down = SystemClock.uptimeMillis()
        event(MotionEvent.ACTION_DOWN, listOf(p))
        SystemClock.sleep(60)
        event(MotionEvent.ACTION_UP, listOf(p))
        SystemClock.sleep(220)
    }

    private fun tap(id: String) = tapAt(point(id))

    private fun drag(from: Pair<Float, Float>, to: Pair<Float, Float>) {
        down = SystemClock.uptimeMillis()
        event(MotionEvent.ACTION_DOWN, listOf(from))
        for (i in 1..10) {
            SystemClock.sleep(30)
            event(
                MotionEvent.ACTION_MOVE,
                listOf(
                    (from.first + (to.first - from.first) * i / 10) to
                        (from.second + (to.second - from.second) * i / 10)
                ),
            )
        }
        event(MotionEvent.ACTION_UP, listOf(to))
        SystemClock.sleep(220)
    }

    private fun shot(name: String) {
        SystemClock.sleep(180)
        File(test.targetContext.getExternalFilesDir(null), "v08-$name.png").outputStream().use {
            test.uiAutomation.takeScreenshot().compress(Bitmap.CompressFormat.PNG, 100, it)
        }
    }

    private fun run() = state().getJSONObject("frontier").getJSONObject("run")

    private fun start() {
        engine.uiAction("home")
        SystemClock.sleep(250)
        tap("f:atlas")
        tap("f:travel:0")
        if (state().getInt("screen") == 53) tap("f:skip")
        assertEquals(52, state().getInt("screen"))
    }

    @Test
    fun atlasQuestsInventoryAndCutsceneSurviveRecreation() {
        var saved = ""
        launch().use {
            engine.uiAction("home")
            SystemClock.sleep(300)
            shot("refuge")
            tap("f:atlas")
            shot("atlas")
            tap("zone:2")
            assertFalse(hit("f:travel:2").getBoolean("enabled"))
            tap("zone:0")
            tap("f:journal")
            tap("quest:hunt-0")
            tap("f:accept:hunt-0")
            assertTrue(
                state()
                    .getJSONObject("frontier")
                    .getJSONArray("quests")
                    .getJSONObject(3)
                    .getBoolean("accepted")
            )
            shot("missions")
            tap("journal-lore")
            tap("f:chronicles")
            tap("chronicle-mode")
            tap("chronicle-next")
            shot("bestiary")
            tap("f:journal")
            tap("f:atlas")
            tap("f:travel:0")
            assertEquals(53, state().getInt("screen"))
            SystemClock.sleep(500)
            shot("cinematic")
            tap("f:pause")
            assertEquals(54, state().getInt("screen"))
            saved = run().toString()
            SystemClock.sleep(300)
            assertEquals(saved, run().toString())
            test.runOnMainSync { activity.persist() }
        }
        launch().use {
            assertEquals(0, state().getInt("screen"))
            tapAt(200f to 410f)
            assertEquals(54, state().getInt("screen"))
            assertEquals(saved, run().toString())
            tap("f:resume")
            assertEquals(53, state().getInt("screen"))
            tap("f:skip")
            tap("inventory")
            assertEquals(40, state().getInt("screen"))
            val before = state().getJSONObject("journey").getJSONArray("items")
            val a = before.getJSONObject(0).getLong("id")
            val b = before.getJSONObject(1).getLong("id")
            val tick = run().getInt("tick")
            drag(point("item:$a"), point("item:$b"))
            assertEquals(
                b,
                state()
                    .getJSONObject("journey")
                    .getJSONArray("items")
                    .getJSONObject(0)
                    .getLong("id"),
            )
            tap("bag_sort")
            assertEquals(
                a,
                state()
                    .getJSONObject("journey")
                    .getJSONArray("items")
                    .getJSONObject(0)
                    .getLong("id"),
            )
            assertEquals(tick, run().getInt("tick"))
            val hover = point("item:$b")
            val now = SystemClock.uptimeMillis()
            val hoverEvent =
                MotionEvent.obtain(
                        now,
                        now,
                        MotionEvent.ACTION_HOVER_MOVE,
                        left + hover.first * scale,
                        top + hover.second * scale,
                        0,
                    )
                    .apply { source = InputDevice.SOURCE_MOUSE }
            assertTrue(test.uiAutomation.injectInputEvent(hoverEvent, true))
            hoverEvent.recycle()
            assertEquals("equip:$b", hit("equip:$b").getString("id"))
            shot("inventory")
            val eq = state().getJSONObject("journey").getJSONArray("equipped").toString()
            drag(point("item:$a"), point("slot:5"))
            assertEquals(eq, state().getJSONObject("journey").getJSONArray("equipped").toString())
            tap("inventory_back")
            tap("f:resume")
            tap("f:stance")
            assertEquals("assault", run().getJSONObject("player").getString("stance"))
            tap("f:stance")
            assertEquals("bulwark", run().getJSONObject("player").getString("stance"))
            shot("combat")
            val p = point("attack")
            down = SystemClock.uptimeMillis()
            event(MotionEvent.ACTION_DOWN, listOf(p))
            SystemClock.sleep(3400)
            event(MotionEvent.ACTION_UP, listOf(p))
            assertTrue(run().getJSONObject("player").getInt("heavy_count") > 0)
            tap("f:pause")
            saved = run().toString()
            test.uiAutomation.performGlobalAction(
                android.accessibilityservice.AccessibilityService.GLOBAL_ACTION_HOME
            )
            SystemClock.sleep(500)
            test.targetContext.startActivity(
                Intent(test.targetContext, MainActivity::class.java)
                    .addFlags(
                        Intent.FLAG_ACTIVITY_NEW_TASK or Intent.FLAG_ACTIVITY_REORDER_TO_FRONT
                    )
            )
            SystemClock.sleep(800)
            assertEquals(54, state().getInt("screen"))
            assertEquals(saved, run().toString())
        }
    }

    @Test
    fun touchCombatEarnsLootAndDragEquipsIt() {
        launch().use {
            start()
            val before = metrics()
            val started = SystemClock.uptimeMillis()
            var capturedBattle = false
            val deadline = started + 110000
            while (run().getInt("kills") < 4 && SystemClock.uptimeMillis() < deadline) {
                val r = run()
                val p = r.getJSONObject("player")
                assertTrue("Player died", p.getInt("hp") > 0)
                val pos = p.getJSONObject("pos")
                val x = pos.getInt("x").toFloat()
                val y = pos.getInt("y").toFloat()
                val enemies = r.getJSONArray("monsters")
                val target =
                    (0 until enemies.length())
                        .map { enemies.getJSONObject(it) }
                        .filter { it.getBoolean("active") && it.getInt("hp") > 0 }
                        .minBy {
                            val q = it.getJSONObject("pos")
                            hypot(q.getInt("x") - x, q.getInt("y") - y)
                        }
                        .getJSONObject("pos")
                val dist = hypot(target.getInt("x") - x, target.getInt("y") - y)
                val fighting = dist < 500
                val direction =
                    route(
                        state().getJSONObject("frontier").getJSONArray("zones").getJSONObject(0),
                        x,
                        y,
                        target.getInt("x").toFloat(),
                        target.getInt("y").toFloat(),
                    )
                val move = if (dist > 250) direction else 0f to 0f
                val base = 108f to 421f
                val finger = (108 + move.first * 47) to (421 + move.second * 47)
                down = SystemClock.uptimeMillis()
                event(MotionEvent.ACTION_DOWN, listOf(base))
                event(MotionEvent.ACTION_MOVE, listOf(finger))
                if (fighting)
                    event(
                        MotionEvent.ACTION_POINTER_DOWN or (1 shl 8),
                        listOf(finger, point("attack")),
                    )
                SystemClock.sleep(if (fighting) 750 else 280)
                event(
                    MotionEvent.ACTION_CANCEL,
                    if (fighting) listOf(finger, point("attack")) else listOf(finger),
                )
                if (fighting && p.getInt("skill_cd") == 0) tap("skill")
                if (p.getInt("hp") < p.getInt("max_hp") / 2 && p.getInt("flasks") > 0) tap("f:heal")
                if (fighting && !capturedBattle) {
                    shot("touch-battle")
                    capturedBattle = true
                }
            }
            assertTrue("No camp loot earned within the touch journey", run().getInt("kills") >= 4)
            tap("inventory")
            val j = state().getJSONObject("journey")
            assertTrue(j.getJSONArray("items").length() > 6)
            val all = j.getJSONArray("items")
            val item = all.getJSONObject(all.length() - 1)
            val id = item.getLong("id")
            val slot = item.getInt("slot")
            val hp = run().getJSONObject("player").getInt("hp")
            drag(point("item:$id"), point("slot:$slot"))
            assertEquals(
                id,
                state().getJSONObject("journey").getJSONArray("equipped").getLong(slot),
            )
            assertTrue(run().getJSONObject("player").getInt("hp") <= hp)
            shot("loot-equipped")
            val after = metrics()
            val seconds = (SystemClock.uptimeMillis() - started) / 1000.0
            val fps = (after.getLong("frames") - before.getLong("frames")) / seconds
            assertTrue("Canvas failed to render during combat", fps > 0)
            File(test.targetContext.getExternalFilesDir(null), "v08-touch-performance.json")
                .writeText(
                    JSONObject()
                        .put("fps", fps)
                        .put("seconds", seconds)
                        .put("kills", run().getInt("kills"))
                        .put(
                            "six_layers",
                            after.getJSONObject("frontier").getInt("equipment_layers"),
                        )
                        .put("native_heap_bytes", android.os.Debug.getNativeHeapAllocatedSize())
                        .put("real_touch", true)
                        .toString(2)
                )
        }
    }

    @Test
    fun everyMenuTargetFitsTheSafeViewport() {
        launch().use {
            val textPaint =
                android.graphics.Paint().apply {
                    textSize = 18f
                    typeface =
                        android.graphics.Typeface.create(
                            "sans-serif",
                            android.graphics.Typeface.NORMAL,
                        )
                }
            val zones = state().getJSONObject("frontier").getJSONArray("zones")
            for (i in 0 until zones.length()) {
                var row = ""
                var lines = 1
                for (word in zones.getJSONObject(i).getString("lore").split(' ')) {
                    val next = if (row.isEmpty()) word else "$row $word"
                    if (textPaint.measureText(next) > 630f && row.isNotEmpty()) {
                        lines++
                        row = word
                    } else row = next
                }
                assertTrue(
                    "Cinematic text exceeds four lines at minimum width: region $i / $lines",
                    lines <= 4,
                )
            }
            engine.uiAction("home")
            SystemClock.sleep(250)
            for (action in listOf("home", "f:atlas", "f:journal", "f:chronicles", "inventory")) {
                engine.uiAction(action)
                SystemClock.sleep(250)
                val list = hits()
                for (i in 0 until list.length()) {
                    val h = list.getJSONObject(i)
                    assertTrue("${h.getString("id")} beyond left", h.getDouble("left") >= 0)
                    assertTrue(
                        "${h.getString("id")} beyond right",
                        h.getDouble("right") <= width + .1,
                    )
                    assertTrue(
                        "${h.getString("id")} below safe viewport",
                        h.getDouble("bottom") <= 540,
                    )
                    assertTrue(h.getDouble("top") >= 0)
                }
                shot("layout-${state().getInt("screen")}-${width.toInt()}")
            }
        }
    }

    private fun route(
        z: JSONObject,
        px: Float,
        py: Float,
        gx: Float,
        gy: Float,
    ): Pair<Float, Float> {
        val obs = z.getJSONArray("obstacles")
        fun free(x: Float, y: Float): Boolean {
            for (dx in listOf(-70, 70)) for (dy in listOf(-70, 70)) {
                val xx = ((x + dx) / 256).toInt()
                val yy = ((y + dy) / 256).toInt()
                if (xx !in 1..46 || yy !in 1..30) return false
                for (i in 0 until obs.length()) {
                    val o = obs.getJSONArray(i)
                    if (
                        xx >= o.getInt(0) &&
                            xx < o.getInt(0) + o.getInt(2) &&
                            yy >= o.getInt(1) &&
                            yy < o.getInt(1) + o.getInt(3)
                    )
                        return false
                }
            }
            return true
        }
        fun clear(x: Float, y: Float): Boolean {
            val n = (max(abs(x - px), abs(y - py)) / 24).toInt() + 1
            return (0..n).all { free(px + (x - px) * it / n, py + (y - py) * it / n) }
        }
        var dest = gx to gy
        if (!clear(gx, gy)) {
            val start = (py.toInt() / 256) * 48 + px.toInt() / 256
            val goal = (gy.toInt() / 256) * 48 + gx.toInt() / 256
            val previous = IntArray(1536) { -1 }
            val queue = java.util.ArrayDeque<Int>()
            queue.add(start)
            previous[start] = start
            while (queue.isNotEmpty()) {
                val at = queue.removeFirst()
                if (at == goal) break
                for ((dx, dy) in listOf(1 to 0, -1 to 0, 0 to 1, 0 to -1)) {
                    val x = at % 48 + dx
                    val y = at / 48 + dy
                    if (x !in 1..46 || y !in 1..30) continue
                    val next = y * 48 + x
                    if (previous[next] >= 0 || !free(x * 256f + 128, y * 256f + 128)) continue
                    previous[next] = at
                    queue.add(next)
                }
            }
            if (previous[goal] >= 0) {
                var at = goal
                while (at != start) {
                    val x = at % 48 * 256f + 128
                    val y = at / 48 * 256f + 128
                    if (clear(x, y)) {
                        dest = x to y
                        break
                    }
                    at = previous[at]
                }
            }
        }
        val dx = dest.first - px
        val dy = dest.second - py
        val len = hypot(dx, dy).coerceAtLeast(1f)
        return dx / len to dy / len
    }
}
