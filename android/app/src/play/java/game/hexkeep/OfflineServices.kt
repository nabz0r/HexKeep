package game.hexkeep
import android.app.Service
import android.content.Context
import android.content.Intent
import android.os.IBinder
import game.hexkeep.core.Engine
// Compile-time facades: the offline application contains no radio or attestation service.
object DeviceIdentity { fun certify(context:Context,engine:Engine):Boolean=false }
class BleLantern(context:Context,engine:Engine) { fun start(){}; fun stop(){} }
class PhareService:Service() { override fun onBind(intent:Intent?):IBinder?=null }
