package game.hexkeep.frontier

import kotlin.math.*
import org.json.JSONObject

/** A bounded 2–4 logical-pixel impulse. HUD is deliberately outside this transform. */
class CombatFeel {
    private var last = -1
    private var born = 0L

    fun offset(run: JSONObject, now: Long, reduced: Boolean): Pair<Float, Float> {
        val id = run.optInt("impact_id")
        if (last != id) {
            if (last >= 0) born = now
            last = id
        }
        if (reduced) return 0f to 0f
        val elapsed = (now - born).coerceAtLeast(0)
        if (elapsed > 150) return 0f to 0f
        val fall = 1 - elapsed / 150f
        return sin(elapsed * .19).toFloat() * 3f * fall to cos(elapsed * .23).toFloat() * 2f * fall
    }
}
