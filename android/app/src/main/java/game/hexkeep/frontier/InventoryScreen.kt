package game.hexkeep.frontier

import android.graphics.*
import kotlin.math.*
import org.json.JSONObject

/**
 * Persistent grid order, six equipment drop targets, touch selection and pointer-hover comparison.
 */
class InventoryScreen(private val u: UiKit, private val layers: EquipmentLayers) {
    var selected = 0L
    var page = 0
    var hover = 0L
    private var pressed = 0L
    private var start = PointF()
    private var pointer = PointF()
    private var dragging = false
    private var snapshot = JSONObject()
    private val cells = LinkedHashMap<Long, RectF>()
    private val slots = LinkedHashMap<Int, RectF>()
    private var width = 960f

    fun clear() {
        pressed = 0
        dragging = false
        hover = 0
    }

    fun draw(c: Canvas, w: Float, data: JSONObject, t: Float) {
        snapshot = data
        width = w
        cells.clear()
        slots.clear()
        val j = data.obj("journey")
        val items = j.array("items").objects()
        val eq = j.array("equipped")
        if (items.none { it.optLong("id") == selected })
            selected = items.firstOrNull()?.optLong("id") ?: 0
        val gridX = 228f
        val gridW = w - gridX - 350f
        val cols = (gridW / 76).toInt().coerceIn(4, 8)
        val capacity = cols * 4
        val pages = max(1, (items.size + capacity - 1) / capacity)
        page = page.coerceIn(0, pages - 1)
        u.header(
            c,
            w,
            "ÉQUIPEMENT  /  ${items.size} SUR 60 EMPLACEMENTS",
            "Ce que la nuit te confie.",
            "inventory_back",
        )
        u.button(c, "codex", "Codex", w - 318, 24f, 134f)
        u.rect(c, 32f, 103f, 176f, 412f, u.panel, 14f, u.alpha(u.gold, 45))
        u.text(c, "TON VEILLEUR", 120f, 131f, 11f, u.gold, true, u.bold)
        layers.hero(c, 120f, 244f, 1.25f, data.obj("frontier").obj("run").obj("player"), j, t)
        val order = intArrayOf(3, 1, 0, 4, 5, 2)
        order.forEachIndexed { i, slot ->
            val x = 43f + (i % 2) * 80
            val y = 255f + (i / 2) * 72
            val item = items.firstOrNull { it.optLong("id") == eq.optLong(slot) }
            val r = RectF(x, y, x + 74, y + 63)
            slots[slot] = r
            u.rect(c, r, 0xff101f2d.toInt(), 8f, u.alpha(layers.tint(item), 100))
            if (item != null) layers.itemIcon(c, item, x + 37, y + 23, 22f)
            else layers.icon(c, slot, x + 37, y + 23, 20f, u.muted)
            u.text(c, slotNames[slot], x + 37, y + 53, 10f, u.muted, true)
            u.hits.add(UiKit.Hit("slot:$slot", r))
        }
        u.text(
            c,
            "${data.obj("frontier").obj("run").obj("player").optInt("max_hp",if(data.optJSONObject("battle")!=null)j.optInt("vitality")else data.obj("frontier").obj("loadout").optInt("max_hp"))} vitalité",
            120f,
            491f,
            14f,
            u.white,
            true,
        )
        u.text(c, "LE SAC", gridX, 119f, 11f, u.gold, font = u.bold)
        u.text(
            c,
            "${j.optInt("dust")} poussières",
            gridX + gridW - 150f,
            119f,
            12f,
            u.mint,
            maxWidth = 150f,
        )
        val cw = (gridW - (cols - 1) * 8) / cols
        for (i in 0 until capacity) {
            val x = gridX + (i % cols) * (cw + 8)
            val y = 137f + (i / cols) * 70f
            val r = RectF(x, y, x + cw, y + 62)
            val item = items.getOrNull(page * capacity + i)
            u.rect(
                c,
                r,
                if (item?.optLong("id") == selected) 0xff2c414d.toInt() else 0xff10232e.toInt(),
                8f,
                if (item == null) u.alpha(u.white, 15)
                else u.alpha(layers.tint(item), if (item.optLong("id") == selected) 220 else 80),
            )
            if (item != null) {
                val id = item.optLong("id")
                cells[id] = r
                layers.itemIcon(c, item, x + cw / 2, y + 29, 26f)
                u.rect(c, x + 8, y + 56, cw - 16, 2f, layers.tint(item), 1f)
                if ((0 until eq.length()).any { eq.optLong(it) == id }) {
                    u.circle(c, x + cw - 10, y + 10, 5f, u.gold)
                    u.text(c, "✓", x + cw - 10, y + 13, 8f, u.ink, true)
                }
                u.hits.add(UiKit.Hit("item:$id", r))
            }
        }
        u.text(
            c,
            "Glisse un objet vers son emplacement ou touche-le pour comparer.",
            gridX,
            435f,
            12f,
            u.muted,
            maxWidth = gridW,
        )
        u.button(c, "bag_sort", "Trier", gridX, 466f, 92f)
        u.button(c, "bag-prev", "‹", gridX + 104, 466f, 48f, enabled = page > 0)
        u.text(c, "${page+1} / $pages", gridX + 188, 497f, 13f, u.muted, true)
        u.button(c, "bag-next", "›", gridX + 224, 466f, 48f, enabled = page < pages - 1)
        if (!j.optBoolean("in_battle"))
            u.button(
                c,
                "forge",
                "Forger · 30",
                gridX + 284,
                466f,
                gridW - 284,
                48f,
                enabled = j.optInt("dust") >= 30 && items.size < 60,
            )
        val item = items.firstOrNull { it.optLong("id") == selected }
        val x = w - 322
        val detailW = 290f
        u.rect(c, x - 12, 103f, 302f, 412f, u.panel, 14f, u.alpha(u.gold, 45))
        if (item != null) {
            val slot = item.optInt("slot")
            val equipped = items.firstOrNull { it.optLong("id") == eq.optLong(slot) }
            val worn = eq.optLong(slot) == selected
            val locked =
                data.optBoolean("online") ||
                    (data.optJSONObject("battle") != null &&
                        data.optJSONObject("expedition") == null)
            val color = layers.tint(item)
            u.text(
                c,
                "${rarityNames[item.optInt("rarity").coerceIn(0,3)]} · ${slotNames[slot]}",
                x + 8,
                132f,
                12f,
                color,
                font = u.bold,
            )
            u.wrap(c, item.optString("name"), x + 8, 166f, detailW - 26, 23f, u.white, 27f, 2)
            u.text(
                c,
                if (worn) "ACTUELLEMENT ÉQUIPÉ" else "COMPARAISON AVEC L’OBJET ÉQUIPÉ",
                x + 8,
                218f,
                10f,
                u.gold,
                font = u.bold,
                maxWidth = detailW - 20,
            )
            val stats =
                listOf(
                    "vitality" to "Vitalité",
                    "power" to "Puissance",
                    "guard" to "Protection",
                    "haste" to "Célérité",
                )
            stats.forEachIndexed { i, (key, label) ->
                val y = 246f + i * 27
                val value = item.optInt(key)
                val delta = value - (equipped?.optInt(key) ?: 0)
                u.text(c, label, x + 8, y, 14f, u.muted)
                u.text(c, "+$value", x + 153, y, 16f, u.white)
                if (!worn)
                    u.text(
                        c,
                        if (delta > 0) "+$delta" else "$delta",
                        x + 229,
                        y,
                        14f,
                        if (delta > 0) u.mint else if (delta < 0) u.red else u.muted,
                    )
            }
            u.wrap(c, item.optString("lore"), x + 8, 362f, detailW - 26, 13f, u.muted, 19f, 2)
            u.button(
                c,
                "equip:$selected",
                if (worn) "Équipé" else "Équiper · ${slotNames[slot]}",
                x + 4,
                414f,
                detailW - 20,
                48f,
                true,
                !worn && !locked,
            )
            u.button(
                c,
                "salvage:$selected",
                "Recycler · +${4*(item.optInt("rarity")+1)} poussières",
                x + 4,
                467f,
                detailW - 20,
                48f,
                false,
                !worn && !j.optBoolean("in_battle"),
            )
        }
        if (dragging) {
            val dragged = items.firstOrNull { it.optLong("id") == pressed }
            if (dragged != null) {
                u.rect(c, pointer.x - 32, pointer.y - 35, 64f, 64f, 0xee294651.toInt(), 10f, u.gold)
                layers.itemIcon(c, dragged, pointer.x, pointer.y - 3, 28f)
                slots[dragged.optInt("slot")]?.let {
                    u.rect(c, it, u.alpha(u.gold, 35), 8f, u.gold)
                }
            }
        } else if (hover != 0L) {
            val hovered = items.firstOrNull { it.optLong("id") == hover }
            val r = cells[hover]
            if (hovered != null && r != null) {
                val tx = min(w - 300, r.left + 20)
                val ty = (r.top - 82).coerceAtLeast(92f)
                u.rect(c, tx, ty, 280f, 72f, 0xfa0c1b29.toInt(), 10f, layers.tint(hovered))
                u.text(
                    c,
                    hovered.optString("name"),
                    tx + 12,
                    ty + 26,
                    16f,
                    u.white,
                    maxWidth = 256f,
                )
                u.text(
                    c,
                    "${slotNames[hovered.optInt("slot")]} · ${rarityNames[hovered.optInt("rarity")]}",
                    tx + 12,
                    ty + 51,
                    12f,
                    layers.tint(hovered),
                )
            }
        }
    }

    fun down(x: Float, y: Float): Boolean {
        val id = cells.entries.firstOrNull { it.value.contains(x, y) }?.key ?: return false
        pressed = id
        selected = id
        start = PointF(x, y)
        pointer = PointF(x, y)
        dragging = false
        return true
    }

    fun move(x: Float, y: Float): Boolean {
        if (pressed == 0L) return false
        pointer = PointF(x, y)
        if (hypot(x - start.x, y - start.y) > 8) dragging = true
        return true
    }

    fun up(x: Float, y: Float, command: (String) -> Unit): Boolean {
        if (pressed == 0L) return false
        val id = pressed
        if (dragging) {
            val slot = slots.entries.firstOrNull { it.value.contains(x, y) }?.key
            val item =
                snapshot.obj("journey").array("items").objects().firstOrNull {
                    it.optLong("id") == id
                }
            if (slot != null && item?.optInt("slot") == slot) {
                command("equip:$id")
            } else {
                val target = cells.entries.firstOrNull { it.value.contains(x, y) }?.key
                if (target != null && target != id) command("bag_move:$id:$target")
            }
        }
        clear()
        return true
    }

    fun hover(x: Float, y: Float) {
        hover = cells.entries.firstOrNull { it.value.contains(x, y) }?.key ?: 0
        if (hover != 0L) selected = hover
    }

    fun selectSlot(slot: Int) {
        selected = snapshot.obj("journey").array("equipped").optLong(slot)
    }
}
