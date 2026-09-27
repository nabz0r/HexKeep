package game.hexkeep.frontier

import android.graphics.Canvas
import org.json.JSONObject

class QuestJournalScreen(private val u: UiKit) {
    var zone = 0
    var selected = ""
    var lore = false
    var chroniclePage = 0

    fun draw(c: Canvas, w: Float, data: JSONObject) {
        val f = data.obj("frontier")
        val zones = f.array("zones").objects()
        if (zones.size < 3) return
        u.header(c, w, "CARNET DU VEILLEUR", "Missions & mémoires", "f:atlas")
        for (i in 0..2) u.button(
            c,
            "journal-zone:$i",
            arrayOf("I · La Sylve", "II · Les Dunes", "III · La Couronne")[i],
            32 + i * ((w - 64) / 3),
            104f,
            (w - 88) / 3,
            48f,
            zone == i,
        )
        val list = f.array("quests").objects().filter { it.optInt("zone") == zone }
        if (list.none { it.optString("id") == selected })
            selected = list.firstOrNull()?.optString("id") ?: ""
        val left = (w * .46f).coerceAtMost(570f)
        list.forEachIndexed { i, q ->
            val y = 171f + i * 70f
            val chosen = q.optString("id") == selected
            u.rect(
                c,
                32f,
                y,
                left - 48,
                60f,
                if (chosen) 0xff29404a.toInt() else u.panel,
                10f,
                if (chosen) u.alpha(u.gold, 140) else 0,
            )
            u.text(
                c,
                if (q.optBoolean("main")) "◆" else "◇",
                50f,
                y + 35,
                22f,
                if (q.optBoolean("claimed")) u.mint else u.gold,
            )
            u.text(
                c,
                q.optString("title"),
                80f,
                y + 25,
                16f,
                u.white,
                font = u.bold,
                maxWidth = left - 190,
            )
            u.text(
                c,
                if (q.optBoolean("locked")) "Région à découvrir"
                else if (q.optBoolean("claimed")) "Accomplie · récompenses reçues"
                else if (q.optBoolean("ready")) "Récompenses disponibles"
                else if (q.optBoolean("accepted"))
                    "${q.optInt("progress")} / ${q.optInt("target")}  •  ${if(q.optBoolean("tracked"))"suivie"else"en cours"}"
                else "Disponible",
                80f,
                y + 46,
                12f,
                if (q.optBoolean("ready")) u.mint else u.muted,
            )
            u.hits.add(
                UiKit.Hit(
                    "quest:${q.optString("id")}",
                    android.graphics.RectF(32f, y, left - 16, y + 60),
                )
            )
        }
        val x = left + 16
        val width = w - x - 32
        val q = list.firstOrNull { it.optString("id") == selected } ?: return
        u.text(
            c,
            if (lore) "MÉMOIRE DU MONDE"
            else if (q.optBoolean("main")) "LE FIL DE L’HISTOIRE" else "UNE LUMIÈRE SUR LE CHEMIN",
            x,
            192f,
            11f,
            u.gold,
            font = u.bold,
        )
        u.text(
            c,
            if (lore) zones[zone].optString("name") else q.optString("title"),
            x,
            230f,
            28f,
            u.white,
            font = u.serif,
            maxWidth = width,
        )
        u.wrap(
            c,
            if (lore) zones[zone].optString("lore") else q.optString("description"),
            x,
            264f,
            width,
            17f,
            u.white,
            26f,
            6,
        )
        if (!lore) {
            u.text(
                c,
                "${q.optInt("dust")} poussières  ·  ${q.optInt("xp")} expérience  ·  1 équipement",
                x,
                420f,
                14f,
                u.gold,
                maxWidth = width,
            )
            val id = q.optString("id")
            val action =
                if (q.optBoolean("ready")) "claim"
                else if (q.optBoolean("accepted")) "track" else "accept"
            u.button(
                c,
                "f:$action:$id",
                when {
                    q.optBoolean("claimed") -> "Mission accomplie"
                    q.optBoolean("ready") -> "Recevoir les récompenses"
                    q.optBoolean("accepted") -> "Suivre cette mission"
                    else -> "Accepter la mission"
                },
                x,
                455f,
                width,
                52f,
                true,
                !q.optBoolean("locked") && !q.optBoolean("claimed"),
            )
        }
        if (lore)
            u.button(c, "f:chronicles", "Chroniques & bestiaire  ›", x, 455f, width, 52f, true)
        u.button(
            c,
            "journal-lore",
            if (lore) "‹ Voir la mission" else "Lire les chroniques",
            32f,
            463f,
            left - 48,
            48f,
        )
    }
}
