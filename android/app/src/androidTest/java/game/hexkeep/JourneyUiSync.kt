package game.hexkeep

import android.os.SystemClock
import androidx.test.platform.app.InstrumentationRegistry
import org.json.JSONObject

/** Wait for Canvas presentation, which follows the native state on a separate tick. */
internal object JourneyUiSync {
    private fun metrics(activity: MainActivity): JSONObject {
        var result = JSONObject()
        InstrumentationRegistry.getInstrumentation().runOnMainSync {
            result = JSONObject(activity.renderMetrics())
        }
        return result
    }

    fun homeViewport(activity: MainActivity): JSONObject {
        val deadline = SystemClock.uptimeMillis() + 10_000
        var last = JSONObject()
        while (SystemClock.uptimeMillis() < deadline) {
            last = metrics(activity)
            val frontier = last.getJSONObject("frontier")
            val hits = frontier.getJSONArray("buttons")
            val hasJournal =
                (0 until hits.length()).any {
                    val hit = hits.getJSONObject(it)
                    hit.getString("id") == "journal" && hit.optBoolean("enabled")
                }
            if (frontier.getInt("screen") == 7 && hasJournal) {
                // Read coordinates after the home layout, not during the launch rotation.
                return last.getJSONObject("viewport")
            }
            SystemClock.sleep(50)
        }
        error("Home did not render its adventure control: $last")
    }

    fun frames(activity: MainActivity) {
        val initial = metrics(activity).getLong("frames")
        val deadline = SystemClock.uptimeMillis() + 10_000
        var last = JSONObject()
        while (SystemClock.uptimeMillis() < deadline) {
            last = metrics(activity)
            if (last.getLong("frames") >= initial + 2) return
            SystemClock.sleep(50)
        }
        error("Renderer did not present two frames after input: $last")
    }
}
