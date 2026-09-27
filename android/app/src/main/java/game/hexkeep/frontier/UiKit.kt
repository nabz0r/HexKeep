package game.hexkeep.frontier

import android.graphics.*
import kotlin.math.*
import org.json.JSONArray
import org.json.JSONObject

/** Shared typography and 48+ logical-pixel touch targets across the Confins screens. */
class UiKit {
    val ink = Color.rgb(9, 18, 29)
    val panel = Color.rgb(17, 32, 44)
    val gold = Color.rgb(231, 195, 131)
    val white = Color.rgb(240, 237, 224)
    val muted = Color.rgb(161, 180, 187)
    val mint = Color.rgb(124, 221, 188)
    val red = Color.rgb(241, 140, 132)
    val p = Paint(Paint.ANTI_ALIAS_FLAG or Paint.FILTER_BITMAP_FLAG)
    val serif = Typeface.create("serif", Typeface.NORMAL)
    val sans = Typeface.create("sans-serif", Typeface.NORMAL)
    val bold = Typeface.create("sans-serif-medium", Typeface.NORMAL)

    data class Hit(val id: String, val rect: RectF, val enabled: Boolean = true)

    val hits = ArrayList<Hit>()
    var focused = ""
    var highContrast = false

    fun reset() {
        hits.clear()
    }

    fun color(hex: String) =
        try {
            Color.parseColor("#$hex")
        } catch (_: Exception) {
            mint
        }

    fun alpha(color: Int, a: Int) =
        Color.argb(a.coerceIn(0, 255), Color.red(color), Color.green(color), Color.blue(color))

    fun rect(c: Canvas, r: RectF, color: Int, radius: Float = 12f, stroke: Int = 0) {
        p.shader = null
        p.style = Paint.Style.FILL
        p.color = color
        c.drawRoundRect(r, radius, radius, p)
        if (stroke != 0) {
            p.style = Paint.Style.STROKE
            p.strokeWidth = 1f
            p.color = stroke
            c.drawRoundRect(r, radius, radius, p)
            p.style = Paint.Style.FILL
        }
    }

    fun rect(
        c: Canvas,
        x: Float,
        y: Float,
        w: Float,
        h: Float,
        color: Int = panel,
        radius: Float = 12f,
        stroke: Int = 0,
    ) = rect(c, RectF(x, y, x + w, y + h), color, radius, stroke)

    fun circle(c: Canvas, x: Float, y: Float, r: Float, color: Int, stroke: Float = 0f) {
        p.shader = null
        p.color = color
        p.style = if (stroke > 0) Paint.Style.STROKE else Paint.Style.FILL
        p.strokeWidth = stroke
        c.drawCircle(x, y, r, p)
        p.style = Paint.Style.FILL
    }

    fun line(c: Canvas, x: Float, y: Float, xx: Float, yy: Float, color: Int, width: Float = 1f) {
        p.shader = null
        p.color = color
        p.strokeWidth = width
        c.drawLine(x, y, xx, yy, p)
    }

    fun text(
        c: Canvas,
        s: String,
        x: Float,
        y: Float,
        size: Float = 16f,
        color: Int = white,
        center: Boolean = false,
        font: Typeface = sans,
        maxWidth: Float = Float.MAX_VALUE,
    ) {
        p.shader = null
        p.style = Paint.Style.FILL
        p.color = if (highContrast && color == muted) white else color
        p.typeface = font
        p.textSize = size
        p.textSize = min(size, size * maxWidth / p.measureText(s).coerceAtLeast(1f))
        p.textAlign = if (center) Paint.Align.CENTER else Paint.Align.LEFT
        c.drawText(s, x, y, p)
        p.textAlign = Paint.Align.LEFT
    }

    fun wrap(
        c: Canvas,
        s: String,
        x: Float,
        y: Float,
        w: Float,
        size: Float = 16f,
        color: Int = muted,
        line: Float = 24f,
        maxLines: Int = 8,
    ): Float {
        p.typeface = sans
        p.textSize = size
        val words = s.replace('\n', ' ').split(' ')
        var row = ""
        var yy = y
        var lines = 1
        for (word in words) {
            val next = if (row.isEmpty()) word else "$row $word"
            if (p.measureText(next) > w && row.isNotEmpty()) {
                if (lines >= maxLines) {
                    text(c, "$row…", x, yy, size, color, maxWidth = w)
                    return yy + line
                }
                text(c, row, x, yy, size, color)
                yy += line
                lines++
                row = word
            } else row = next
        }
        if (row.isNotEmpty()) text(c, row, x, yy, size, color)
        return yy + line
    }

    fun button(
        c: Canvas,
        id: String,
        label: String,
        x: Float,
        y: Float,
        w: Float,
        h: Float = 48f,
        primary: Boolean = false,
        enabled: Boolean = true,
    ) {
        val fill =
            if (!enabled) alpha(panel, 160)
            else if (primary) gold else if (focused == id) 0xff314b59.toInt() else panel
        rect(c, x, y, w, h, fill, 10f, if (primary) 0 else alpha(gold, if (enabled) 75 else 25))
        text(
            c,
            label,
            x + w / 2,
            y + h / 2 + 6,
            16f,
            if (!enabled) muted else if (primary) ink else white,
            true,
            bold,
            w - 20,
        )
        hits.add(Hit(id, RectF(x, y, x + w, y + h), enabled))
    }

    fun polygon(
        c: Canvas,
        x: Float,
        y: Float,
        r: Float,
        n: Int,
        color: Int,
        angle: Float = -90f,
        stroke: Float = 0f,
    ) {
        val path = Path()
        for (i in 0 until n) {
            val a = (angle + i * 360f / n) * Math.PI / 180
            val xx = x + cos(a).toFloat() * r
            val yy = y + sin(a).toFloat() * r
            if (i == 0) path.moveTo(xx, yy) else path.lineTo(xx, yy)
        }
        path.close()
        p.shader = null
        p.color = color
        p.style = if (stroke > 0) Paint.Style.STROKE else Paint.Style.FILL
        p.strokeWidth = stroke
        c.drawPath(path, p)
        p.style = Paint.Style.FILL
    }

    fun backdrop(c: Canvas, w: Float, t: Float, motion: Boolean) {
        c.drawColor(ink)
        p.shader =
            LinearGradient(
                0f,
                0f,
                w,
                540f,
                intArrayOf(0xff172f3a.toInt(), ink, 0xff19202f.toInt()),
                null,
                Shader.TileMode.CLAMP,
            )
        c.drawRect(0f, 0f, w, 540f, p)
        p.shader = null
        for (i in 0..14) {
            val x = (i * 173f + 31) % w
            val y = 60f + (i * 79) % 420
            polygon(c, x, y, 45f + (i % 4) * 15, 6, alpha(mint, 9), 30f, 1f)
        }
        if (motion)
            for (i in 0..19) {
                circle(
                    c,
                    (i * 139.1f + sin(t * .3f + i) * 12) % w,
                    540f - (i * 41 + t * (2 + i % 4)) % 540,
                    1f,
                    alpha(gold, 75),
                )
            }
    }

    fun header(c: Canvas, w: Float, kicker: String, title: String, back: String = "f:home") {
        text(c, kicker, 32f, 30f, 11f, gold, font = bold)
        text(c, title, 30f, 68f, 30f, white, font = serif, maxWidth = w - 225)
        button(c, back, "‹ Retour", w - 170, 24f, 138f)
        line(c, 32f, 86f, w - 32, 86f, alpha(gold, 50))
    }

    fun metrics() =
        JSONArray().apply {
            hits.forEach {
                put(
                    JSONObject()
                        .put("id", it.id)
                        .put("enabled", it.enabled)
                        .put("left", it.rect.left)
                        .put("top", it.rect.top)
                        .put("right", it.rect.right)
                        .put("bottom", it.rect.bottom)
                )
            }
        }
}

internal fun JSONArray.objects(): List<JSONObject> =
    (0 until length()).mapNotNull { optJSONObject(it) }

internal fun JSONObject.array(name: String) = optJSONArray(name) ?: JSONArray()

internal fun JSONObject.obj(name: String) = optJSONObject(name) ?: JSONObject()

internal val slotNames = arrayOf("Arme", "Plastron", "Amulette", "Casque", "Gants", "Bottes")
internal val rarityNames = arrayOf("Commun", "Rare", "Épique", "Légendaire")
