package game.hexkeep
import android.app.Activity
import game.hexkeep.core.Engine
/** Offline edition has no Billing SDK and cannot start a purchase. */
class PlayShop(activity:Activity, private val engine:Engine) {
    fun inspect(product:String) { engine.notice("Cette édition ne contient aucun achat intégré.") }
    fun close() {}
}
