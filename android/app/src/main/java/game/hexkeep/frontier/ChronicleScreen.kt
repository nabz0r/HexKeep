package game.hexkeep.frontier

import android.graphics.Canvas
import org.json.JSONObject

class ChronicleScreen(private val u: UiKit, private val art: game.hexkeep.art.PaintedArt) {
    var page = 0
    var bestiary = false
    private val entities = EntityRenderer(u, art)

    fun draw(c: Canvas, w: Float, data: JSONObject) {
        val f = data.obj("frontier")
        u.header(
            c,
            w,
            "LES ARCHIVES DU REFUGE",
            if (bestiary) "Les formes de la nuit." else "Ceux dont on garde le nom.",
            "f:journal",
        )
        u.button(
            c,
            "chronicle-mode",
            if (bestiary) "Lire les chroniques" else "Ouvrir le bestiaire",
            32f,
            104f,
            224f,
        )
        val chapters = f.optString("chronicles").split("\n\n").filter { it.isNotBlank() }
        val monsters = f.array("monsters").objects()
        val count = if (bestiary) monsters.size else chapters.size
        page = page.coerceIn(0, (count - 1).coerceAtLeast(0))
        u.rect(c, 32f, 163f, w - 64, 273f, 0xef111923.toInt(), 8f, u.alpha(u.gold, 60))
        if (bestiary && monsters.isNotEmpty()) {
            val m = monsters[page]
            c.save()
            c.translate(w * .25f, 416f)
            c.scale(if (page == 4) 1.65f else 2.5f, if (page == 4) 1.65f else 2.5f)
            entities.monster(
                c,
                JSONObject()
                    .put("kind", page)
                    .put("hp", 1)
                    .put("max_hp", 1)
                    .put("active", true)
                    .put("state", "idle")
                    .put("stance", if (page == 2) "bulwark" else "balanced"),
                0f,
                0f,
                0f,
                0,
            )
            c.restore()
            val x = w * .46f
            u.text(
                c,
                m.optString("name"),
                x,
                220f,
                32f,
                u.white,
                font = u.serif,
                maxWidth = w - x - 45,
            )
            u.text(c, m.optString("role"), x, 256f, 15f, u.gold)
            u.wrap(c, m.optString("tactic"), x, 298f, w - x - 45, 20f, u.white, 30f, 5)
        } else if (chapters.isNotEmpty())
            u.wrap(c, chapters[page], 64f, 206f, w - 128, 22f, u.white, 34f, 7)
        u.button(c, "chronicle-prev", "‹ Précédent", 32f, 458f, 178f, enabled = page > 0)
        u.text(c, "${page+1} / $count", w / 2, 489f, 14f, u.muted, true)
        u.button(c, "chronicle-next", "Suivant ›", w - 210, 458f, 178f, enabled = page < count - 1)
    }
}
