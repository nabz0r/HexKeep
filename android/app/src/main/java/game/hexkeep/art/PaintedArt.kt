package game.hexkeep.art

import android.content.res.AssetManager
import android.graphics.*
import kotlin.math.*
import org.json.JSONObject

/** The only owner of v0.8 decoded art. Eviction releases references, never live Canvas bitmaps. */
class PaintedArt(private val assets: AssetManager) {
    data class Sheet(val bitmap: Bitmap, val frames: List<Rect>, val actorHeight: Float)

    private val cache = LinkedHashMap<String, Sheet>(32, .75f, true)
    private val paint = Paint(Paint.ANTI_ALIAS_FLAG or Paint.FILTER_BITMAP_FLAG)
    private val used = linkedSetOf<String>()
    private var bytes = 0L
    private var peak = 0L
    private var loads = 0
    private val limit = 88L * 1024 * 1024
    val heroNames = arrayOf("hero-aurelon", "hero-skarn", "hero-vylde")
    val enemyNames = arrayOf("enemy-ravager", "enemy-needle", "enemy-bastion", "enemy-swarm")
    val biomeNames = arrayOf("forest", "desert", "frost")
    val itemNames = arrayOf("weapons", "chests", "amulets", "helms", "gloves", "boots")

    fun sheet(name: String): Sheet {
        used.add(name)
        cache[name]?.let {
            return it
        }
        val opaque = name.startsWith("floor-") || name == "refuge" || name == "world-atlas"
        val options =
            BitmapFactory.Options().apply {
                inPreferredConfig = if (opaque) Bitmap.Config.RGB_565 else Bitmap.Config.ARGB_8888
            }
        val bitmap =
            assets.open("art/v08/$name.png").use {
                requireNotNull(BitmapFactory.decodeStream(it, null, options)) {
                    "Invalid art: $name"
                }
            }
        val cols =
            when {
                opaque -> 1
                name == "npcs" -> 3
                else -> 4
            }
        val rows =
            when {
                opaque -> 1
                name == "npcs" || name == "abilities" -> 2
                else -> 3
            }
        val frames =
            if (opaque) listOf(Rect(0, 0, bitmap.width, bitmap.height))
            else {
                // Read one row at a time; no full-image IntArray alongside the decoded bitmap.
                val pixels = IntArray(bitmap.width)
                val bounds = Array(cols * rows) { Rect(bitmap.width, bitmap.height, -1, -1) }
                for (y in 0 until bitmap.height) {
                    bitmap.getPixels(pixels, 0, bitmap.width, 0, y, bitmap.width, 1)
                    for (x in pixels.indices) if ((pixels[x] ushr 24) > 32) {
                        val col = (x * cols / bitmap.width).coerceAtMost(cols - 1)
                        val row = (y * rows / bitmap.height).coerceAtMost(rows - 1)
                        val r = bounds[row * cols + col]
                        r.left = min(r.left, x)
                        r.top = min(r.top, y)
                        r.right = max(r.right, x + 1)
                        r.bottom = max(r.bottom, y + 1)
                    }
                }
                bounds.mapIndexed { i, b ->
                    require(b.right > b.left && b.bottom > b.top) { "Empty frame $i in $name" }
                    b
                }
            }
        val height = frames.take(min(8, frames.size)).map { it.height() }.average().toFloat()
        val value = Sheet(bitmap, frames, height)
        cache[name] = value
        bytes += bitmap.allocationByteCount
        loads++
        while (bytes > limit && cache.size > 1) {
            val first = cache.entries.iterator()
            val old = first.next().value
            bytes -= old.bitmap.allocationByteCount
            first.remove()
        }
        peak = max(peak, bytes)
        return value
    }

    fun frame(
        c: Canvas,
        name: String,
        index: Int,
        dest: RectF,
        alpha: Int = 255,
        filter: ColorFilter? = null,
    ) {
        val s = sheet(name)
        paint.shader = null
        paint.color = Color.WHITE
        paint.alpha = alpha.coerceIn(0, 255)
        paint.colorFilter = filter
        c.drawBitmap(s.bitmap, s.frames[index.coerceIn(0, s.frames.lastIndex)], dest, paint)
        paint.alpha = 255
        paint.colorFilter = null
    }

    fun fit(c: Canvas, name: String, index: Int, box: RectF, alpha: Int = 255) {
        val r = sheet(name).frames[index.coerceIn(0, sheet(name).frames.lastIndex)]
        val scale = min(box.width() / r.width(), box.height() / r.height())
        val w = r.width() * scale
        val h = r.height() * scale
        frame(
            c,
            name,
            index,
            RectF(
                box.centerX() - w / 2,
                box.centerY() - h / 2,
                box.centerX() + w / 2,
                box.centerY() + h / 2,
            ),
            alpha,
        )
    }

    fun anchored(
        c: Canvas,
        name: String,
        index: Int,
        x: Float,
        y: Float,
        height: Float,
        flip: Boolean = false,
        alpha: Int = 255,
        filter: ColorFilter? = null,
    ): RectF {
        val s = sheet(name)
        val r = s.frames[index.coerceIn(0, s.frames.lastIndex)]
        val scale = height / s.actorHeight
        val dest =
            RectF(x - r.width() * scale / 2, y - r.height() * scale, x + r.width() * scale / 2, y)
        c.save()
        if (flip) c.scale(-1f, 1f, x, y)
        frame(c, name, index, dest, alpha, filter)
        c.restore()
        return dest
    }

    fun cover(c: Canvas, name: String, dest: RectF, alpha: Int = 255) {
        val bitmap = sheet(name).bitmap
        val scale = max(dest.width() / bitmap.width, dest.height() / bitmap.height)
        val w = bitmap.width * scale
        val h = bitmap.height * scale
        c.save()
        c.clipRect(dest)
        paint.color = Color.WHITE
        paint.alpha = alpha
        paint.colorFilter = null
        c.drawBitmap(
            bitmap,
            null,
            RectF(
                dest.centerX() - w / 2,
                dest.centerY() - h / 2,
                dest.centerX() + w / 2,
                dest.centerY() + h / 2,
            ),
            paint,
        )
        paint.alpha = 255
        c.restore()
    }

    fun item(c: Canvas, item: JSONObject, box: RectF, alpha: Int = 255) {
        val slot = item.optInt("slot").coerceIn(0, 5)
        fit(c, "items-${itemNames[slot]}", item.optInt("catalog") % 12, box, alpha)
    }

    fun metrics() =
        JSONObject()
            .put("style", "painted-fantasy-v08")
            .put("resident_bytes", bytes)
            .put("peak_resident_bytes", peak)
            .put("budget_bytes", limit)
            .put("loads", loads)
            .put("assets", org.json.JSONArray(used.toList()))

    fun close() {
        cache.values.forEach { it.bitmap.recycle() }
        cache.clear()
        bytes = 0
    }
}
