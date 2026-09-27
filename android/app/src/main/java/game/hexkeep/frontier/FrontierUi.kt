package game.hexkeep.frontier

import android.graphics.*
import android.os.SystemClock
import android.view.MotionEvent
import kotlin.math.*
import org.json.JSONObject

/** Screen routing and pointer ownership. Simulation, content and equipment rules live in Rust. */
class FrontierUi(
    private val art: game.hexkeep.art.PaintedArt,
    private val command: (String) -> Unit,
    private val movement: (Short, Short) -> Unit,
    private val attack: (Boolean) -> Unit,
    private val dash: () -> Unit,
    private val skill: () -> Unit,
) {
    private val u = UiKit(art)
    private val layers = EquipmentLayers(u, art)
    private val atlas = AtlasScreen(u, art)
    private val journal = QuestJournalScreen(u, art)
    private val inventory = InventoryScreen(u, layers)
    private val world = FrontierRenderer(u, layers, art)
    private val chronicles = ChronicleScreen(u, art)
    private var screen = -1
    private var started = 0L
    private var data = JSONObject()
    private var width = 960f
    private var movePointer = -1
    private var attackPointer = -1
    private var buttonPointer = -1
    private var itemPointer = -1
    private var origin = PointF(108f, 421f)
    private var mx = 0f
    private var my = 0f
    private var pressed: UiKit.Hit? = null

    fun handles(screen: Int) = screen == 7 || screen == 40 || screen in 50..56

    fun clear() {
        movePointer = -1
        attackPointer = -1
        buttonPointer = -1
        itemPointer = -1
        mx = 0f
        my = 0f
        origin = PointF(108f, 421f)
        pressed = null
        u.focused = ""
        inventory.clear()
    }

    fun close() {
        world.close()
    }

    fun metrics() =
        JSONObject()
            .put("screen", screen)
            .put("buttons", u.metrics())
            .put("art", art.metrics())
            .put("equipment_layers", 6)

    fun draw(c: Canvas, w: Float, snapshot: JSONObject, reduced: Boolean) {
        data = snapshot
        width = w
        val next = data.optInt("screen")
        val now = SystemClock.uptimeMillis()
        if (next != screen) {
            screen = next
            started = now
            clear()
        }
        layers.realm = data.optInt("realm")
        u.highContrast = data.optBoolean("accessible")
        u.reset()
        val t = now / 1000f
        if (screen !in 52..55) u.backdrop(c, w, t, !reduced)
        when (screen) {
            7 -> home(c, w, t)
            40 -> inventory.draw(c, w, data, t)
            50 -> atlas.draw(c, w, data, t)
            51 -> journal.draw(c, w, data)
            56 -> chronicles.draw(c, w, data)
            52 -> {
                world.draw(c, w, data, reduced)
                world.hud(c, w, data, mx, my, origin)
            }
            53 -> world.draw(c, w, data, reduced, true)
            54 -> {
                world.draw(c, w, data, reduced)
                pause(c, w)
            }
            55 -> {
                world.draw(c, w, data, reduced)
                result(c, w)
            }
        }
        // A brief fade resolves menu changes without changing hit target geometry.
        if (!reduced && screen !in 52..53) {
            val alpha = (1 - (now - started) / 170f).coerceIn(0f, 1f)
            if (alpha > 0) u.rect(c, 0f, 0f, w, 540f, u.alpha(u.ink, (alpha * 120).toInt()), 0f)
        }
    }

    private fun home(c: Canvas, w: Float, t: Float) {
        val f = data.obj("frontier")
        val j = data.obj("journey")
        val run = f.optJSONObject("run")
        u.text(c, "HEXKEEP", 31f, 64f, 34f, u.white, font = u.serif)
        u.text(c, "0.8  /  LE SERMENT DES LANTERNES", 33f, 89f, 11f, u.gold, font = u.bold)
        u.button(c, "settings", "Réglages", w - 172, 28f, 140f)
        val x = w * .70f
        u.p.shader =
            RadialGradient(
                x,
                330f,
                190f,
                intArrayOf(0x40d5a458, 0x00d5a458),
                null,
                Shader.TileMode.CLAMP,
            )
        c.drawCircle(x, 330f, 190f, u.p)
        u.p.shader = null
        layers.hero(c, x, 426f, 3.55f, JSONObject(), j, t)
        u.text(c, "GARDIEN DE LA DERNIÈRE LUMIÈRE", x, 457f, 11f, u.gold, true, u.bold)
        u.text(
            c,
            "${data.optString("name")} · Niveau ${j.optInt("level",1)}",
            33f,
            148f,
            15f,
            u.mint,
            maxWidth = w * .48f,
        )
        u.text(
            c,
            "Au-delà de la dernière",
            30f,
            208f,
            34f,
            u.white,
            font = u.serif,
            maxWidth = w * .51f,
        )
        u.text(c, "lanterne.", 30f, 249f, 42f, u.white, font = u.serif)
        u.wrap(
            c,
            "Une forêt de voix. Un désert de souvenirs. Une couronne sous les aurores. Retrouve ceux que la nuit n’a pas emportés.",
            34f,
            287f,
            w * .44f,
            18f,
            u.muted,
            26f,
            3,
        )
        u.button(
            c,
            if (run != null) "f:resume" else "f:atlas",
            if (run != null) "Reprendre les Confins  ›" else "Ouvrir l’atlas des Confins  ›",
            32f,
            374f,
            365f,
            58f,
            true,
        )
        u.button(c, "journal", "Les Marches · aventures", 32f, 450f, 235f, 52f)
        u.button(c, "inventory", "Sac & équipement", 281f, 450f, 216f, 52f)
        u.button(c, "legacy-home", "Refuge & autres modes", w - 332, 476f, 300f, 48f)
        u.text(
            c,
            "${f.array("zones").objects().count{it.optBoolean("complete")}} / 3 régions libérées · ${f.array("quests").objects().count{it.optBoolean("claimed")}} / 12 missions",
            33f,
            529f,
            11f,
            u.muted,
        )
    }

    private fun pause(c: Canvas, w: Float) {
        u.rect(c, 0f, 0f, w, 540f, 0xd908131f.toInt(), 0f)
        val x = w / 2 - 395
        u.text(c, "UNE RESPIRATION", x, 84f, 12f, u.gold, font = u.bold)
        u.text(c, "Ta lumière t’attendra.", x, 126f, 34f, u.white, font = u.serif)
        u.button(c, "f:resume", "Reprendre l’aventure  ›", x, 159f, 350f, 56f, true)
        u.button(c, "inventory", "Sac & équipement", x, 229f, 350f, 48f)
        u.button(c, "f:journal", "Journal des missions", x, 291f, 350f, 48f)
        u.button(c, "settings", "Réglages & accessibilité", x, 353f, 350f, 48f)
        u.button(c, "f:leave", "Rentrer · terminer cette sortie", x, 427f, 350f, 48f)
        val right = x + 398
        u.text(c, "L’ART DE LA VEILLE", right, 181f, 12f, u.gold, font = u.bold)
        u.wrap(
            c,
            "Maintiens l’attaque : trois coups rapides préparent un quatrième coup lourd qui brise les gardes.",
            right,
            220f,
            370f,
            17f,
            u.white,
            25f,
            3,
        )
        u.wrap(
            c,
            "Change de posture pour accélérer l’assaut ou renforcer ta défense. Les éclats rouges annoncent un coup : esquive avant l’impact.",
            right,
            314f,
            370f,
            16f,
            u.muted,
            24f,
            4,
        )
        u.wrap(
            c,
            "Les balises soignent. Tes trois fioles et ton éclat de lumière sont précieux face aux gardiens.",
            right,
            425f,
            370f,
            15f,
            u.mint,
            23f,
            3,
        )
        u.text(
            c,
            "La fermeture de l’application conserve cette sortie. Rentrer conserve les découvertes et le butin.",
            w / 2,
            513f,
            12f,
            u.muted,
            true,
            maxWidth = w - 64,
        )
    }

    private fun result(c: Canvas, w: Float) {
        val run = data.obj("frontier").obj("run")
        val won = run.optBoolean("victory")
        u.rect(c, 0f, 0f, w, 540f, 0xdf08131f.toInt(), 0f)
        u.polygon(c, w / 2, 92f, 29f, 6, u.gold, 30f, 2f)
        u.polygon(c, w / 2, 92f, 13f, 4, if (won) u.mint else u.red)
        u.text(
            c,
            if (won) "LE MONDE SE SOUVIENT" else "LA FLAMME DEMEURE",
            w / 2,
            156f,
            12f,
            u.gold,
            true,
            u.bold,
        )
        u.text(
            c,
            if (won) "Une nouvelle route s’éveille." else "La nuit ne garde pas ton serment.",
            w / 2,
            211f,
            36f,
            u.white,
            true,
            u.serif,
            w - 80,
        )
        u.text(
            c,
            "${run.optInt("kills")} ombres dissipées · ${run.array("beacons").let{a->(0 until a.length()).count{a.optBoolean(it)}}} / 3 balises · ${run.obj("player").optInt("heavy_count")} coups finissants",
            w / 2,
            262f,
            17f,
            u.muted,
            true,
        )
        u.text(
            c,
            "Le butin et les mémoires recueillies sont conservés.",
            w / 2,
            298f,
            16f,
            u.mint,
            true,
        )
        u.button(c, "f:journal", "Missions & récompenses  ›", w / 2 - 210, 342f, 420f, 56f, true)
        u.button(c, "inventory", "Examiner le butin", w / 2 - 210, 412f, 202f, 50f)
        u.button(c, "f:leave", "Retrouver l’atlas", w / 2 + 8, 412f, 202f, 50f)
    }

    private fun dispatch(id: String) {
        when {
            id == "chronicle-next" -> chronicles.page++
            id == "chronicle-prev" -> chronicles.page--
            id == "chronicle-mode" -> {
                chronicles.bestiary = !chronicles.bestiary
                chronicles.page = 0
            }
            id.startsWith("zone:") -> atlas.selected = id.substringAfter(':').toInt()
            id.startsWith("journal-zone:") -> {
                journal.zone = id.substringAfter(':').toInt()
                journal.lore = false
            }
            id.startsWith("quest:") -> {
                journal.selected = id.substringAfter(':')
                journal.lore = false
            }
            id == "journal-lore" -> journal.lore = !journal.lore
            id.startsWith("slot:") -> inventory.selectSlot(id.substringAfter(':').toInt())
            id == "bag-prev" -> inventory.page--
            id == "bag-next" -> inventory.page++
            id == "attack" -> attack(true)
            id == "dash" -> dash()
            id == "skill" -> skill()
            else -> command(id)
        }
    }

    fun touch(e: MotionEvent, left: Float, top: Float, scale: Float): Boolean {
        if (e.actionMasked == MotionEvent.ACTION_CANCEL) {
            clear()
            movement(0, 0)
            attack(false)
            return true
        }
        for (i in 0 until e.pointerCount) {
            if (e.actionMasked != MotionEvent.ACTION_MOVE && i != e.actionIndex) continue
            val pointer = e.getPointerId(i)
            val x = (e.getX(i) - left) / scale
            val y = (e.getY(i) - top) / scale
            val down =
                e.actionMasked == MotionEvent.ACTION_DOWN ||
                    e.actionMasked == MotionEvent.ACTION_POINTER_DOWN
            val up =
                e.actionMasked == MotionEvent.ACTION_UP ||
                    e.actionMasked == MotionEvent.ACTION_POINTER_UP
            if (down) {
                if (screen == 40 && itemPointer < 0 && inventory.down(x, y)) {
                    itemPointer = pointer
                    continue
                }
                val hit = u.hits.lastOrNull { it.rect.contains(x, y) }
                if (hit != null) {
                    if (!hit.enabled) continue
                    if (screen == 52 && hit.id in listOf("attack", "dash", "skill")) {
                        if (hit.id == "attack") attackPointer = pointer
                        dispatch(hit.id)
                    } else if (buttonPointer < 0) {
                        buttonPointer = pointer
                        pressed = hit
                        u.focused = hit.id
                    }
                    continue
                }
                if (screen == 52 && movePointer < 0 && x < width * .45f && y > 170) {
                    movePointer = pointer
                    origin = PointF(x, y)
                }
            }
            if (pointer == itemPointer) {
                if (up) {
                    inventory.up(x, y, command)
                    itemPointer = -1
                } else inventory.move(x, y)
                continue
            }
            if (pointer == movePointer) {
                if (up) {
                    movePointer = -1
                    mx = 0f
                    my = 0f
                    origin = PointF(108f, 421f)
                    movement(0, 0)
                } else {
                    val dx = x - origin.x
                    val dy = y - origin.y
                    val len = hypot(dx, dy)
                    val strength = ((len - 3) / 40).coerceIn(0f, 1f)
                    mx = dx / len.coerceAtLeast(1f) * strength
                    my = dy / len.coerceAtLeast(1f) * strength
                    movement((mx * 1024).toInt().toShort(), (my * 1024).toInt().toShort())
                }
            }
            if (up && pointer == attackPointer) {
                attackPointer = -1
                attack(false)
            }
            if (up && pointer == buttonPointer) {
                val hit = pressed
                buttonPointer = -1
                pressed = null
                u.focused = ""
                if (hit != null && hit.rect.contains(x, y)) dispatch(hit.id)
            }
        }
        return true
    }

    fun hover(x: Float, y: Float, exit: Boolean) {
        if (screen == 40) inventory.hover(if (exit) -1f else x, if (exit) -1f else y)
    }
}
