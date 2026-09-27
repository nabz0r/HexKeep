package game.hexkeep.frontier

import android.graphics.Canvas
import org.json.JSONObject

/** Camera and typewriter progress come from the saved Rust timeline, never the wall clock. */
class CutsceneDirector(private val u: UiKit) {
    data class Camera(val x: Float, val y: Float, val zoom: Float)

    fun camera(scene: JSONObject, reduced: Boolean): Camera {
        val progress =
            if (reduced) 1f
            else (scene.optInt("elapsed") / 90f).coerceIn(0f, 1f).let { it * it * (3 - 2 * it) }
        val from = scene.array("previous_camera")
        val to = scene.array("camera")
        fun axis(i: Int) =
            (from.optDouble(i) + (to.optDouble(i) - from.optDouble(i)) * progress).toFloat() + .5f
        return Camera(
            axis(0),
            axis(1),
            (scene.optDouble("previous_zoom", 1.0) +
                    (scene.optDouble("zoom", 1.0) - scene.optDouble("previous_zoom", 1.0)) *
                        progress)
                .toFloat(),
        )
    }

    fun overlay(c: Canvas, w: Float, scene: JSONObject, reduced: Boolean) {
        u.rect(c, 0f, 0f, w, 78f, 0xed07111c.toInt(), 0f)
        u.rect(c, 0f, 349f, w, 191f, 0xf207111c.toInt(), 0f)
        u.text(c, "LES ÉCHOS DES CONFINS", 34f, 34f, 11f, u.gold, font = u.bold)
        u.text(c, "${scene.optInt("beat")+1} / ${scene.optInt("total")}", 34f, 59f, 12f, u.muted)
        u.button(c, "f:pause", "Pause", w - 267, 18f, 105f)
        u.button(c, "f:skip", "Passer", w - 148, 18f, 114f)
        u.text(c, scene.optString("speaker").uppercase(), 42f, 385f, 12f, u.gold, font = u.bold)
        val full = scene.optString("text")
        val reveal =
            if (reduced) full.length
            else (scene.optInt("elapsed") * 1.8).toInt().coerceAtMost(full.length)
        u.wrap(c, full.take(reveal), 42f, 421f, w - 330, 18f, u.white, 28f, 4)
        u.button(
            c,
            "f:next",
            if (scene.optInt("beat") == scene.optInt("total") - 1) "Reprendre la route  ›"
            else "Continuer  ›",
            w - 267,
            445f,
            225f,
            54f,
            true,
        )
        u.rect(c, 42f, 520f, w - 84, 2f, u.alpha(u.gold, 30), 0f)
        u.rect(
            c,
            42f,
            520f,
            (w - 84) *
                (scene.optInt("elapsed").toFloat() / scene.optInt("duration", 1).coerceAtLeast(1))
                    .coerceIn(0f, 1f),
            2f,
            u.gold,
            0f,
        )
    }
}
