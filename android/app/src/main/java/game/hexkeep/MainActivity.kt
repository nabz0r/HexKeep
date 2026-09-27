package game.hexkeep

import android.Manifest
import android.app.Activity
import android.app.KeyguardManager
import android.content.Context
import android.content.Intent
import android.content.pm.PackageManager
import android.hardware.biometrics.BiometricManager
import android.hardware.biometrics.BiometricPrompt
import android.location.Location
import android.location.LocationListener
import android.location.LocationManager
import android.media.AudioAttributes
import android.media.AudioFormat
import android.media.AudioTrack
import android.media.AudioManager
import android.net.wifi.WifiManager
import android.os.*
import android.security.keystore.KeyGenParameterSpec
import android.security.keystore.KeyProperties
import android.util.AtomicFile
import android.util.Log
import android.view.WindowManager
import game.hexkeep.core.Engine
import java.io.File
import java.security.KeyStore
import java.util.concurrent.atomic.AtomicBoolean
import javax.crypto.Cipher
import javax.crypto.KeyGenerator
import javax.crypto.SecretKey
import javax.crypto.spec.GCMParameterSpec

class Vault(private val activity: Context) {
    companion object { private val diskLock = Any() }
    private val store = KeyStore.getInstance("AndroidKeyStore").apply { load(null) }
    private val file = AtomicFile(File(activity.filesDir,"state.hk"))
    private val alias=if(BuildConfig.OFFLINE_EDITION)"hexkeep-offline-state-v1"else"hexkeep-state-v1"
    private fun key(): SecretKey {
        (store.getKey(alias,null) as? SecretKey)?.let { return it }
        val secure=!BuildConfig.OFFLINE_EDITION && activity.getSystemService(KeyguardManager::class.java).isDeviceSecure
        return KeyGenerator.getInstance(KeyProperties.KEY_ALGORITHM_AES,"AndroidKeyStore").apply {
            init(KeyGenParameterSpec.Builder(alias,KeyProperties.PURPOSE_ENCRYPT or KeyProperties.PURPOSE_DECRYPT)
                .setBlockModes(KeyProperties.BLOCK_MODE_GCM).setEncryptionPaddings(KeyProperties.ENCRYPTION_PADDING_NONE)
                .setKeySize(256).setUserAuthenticationRequired(secure)
                .apply { if(secure) setUserAuthenticationValidityDurationSeconds(86400) }
                .build())
        }.generateKey()
    }
    fun read():String { synchronized(diskLock) {
        if(!file.baseFile.exists()) return ""
        val bytes=file.readFully()
        require(bytes.size>29 && bytes[0]==1.toByte()) { "Format de sauvegarde incorrect" }
        val cipher=Cipher.getInstance("AES/GCM/NoPadding")
        cipher.init(Cipher.DECRYPT_MODE,key(),GCMParameterSpec(128,bytes.copyOfRange(1,13)))
        return cipher.doFinal(bytes.copyOfRange(13,bytes.size)).toString(Charsets.UTF_8)
    } }
    fun write(snapshot:String) { synchronized(diskLock) {
        val cipher=Cipher.getInstance("AES/GCM/NoPadding");cipher.init(Cipher.ENCRYPT_MODE,key())
        val encrypted=byteArrayOf(1)+cipher.iv+cipher.doFinal(snapshot.toByteArray(Charsets.UTF_8))
        val stream=file.startWrite()
        try {stream.write(encrypted);file.finishWrite(stream)}catch(e:Exception){file.failWrite(stream);throw e}
    } }
}

class MainActivity : Activity(), LocationListener {
    lateinit var engine: Engine
    private lateinit var surface: NightSurface
    private lateinit var vault: Vault
    private var authContinuation:(()->Unit)?=null
    private var ble:BleLantern?=null
    private var shop:PlayShop?=null
    private var ready=false
    @Volatile private var protectedWindow=false
    private var audioRunning=AtomicBoolean(false)
    private var authSignal: CancellationSignal?=null
    private var resumed=false
    private var multicast:WifiManager.MulticastLock?=null
    private val locationManager by lazy {getSystemService(LocationManager::class.java)}
    private var lastLocation:Location?=null
    @Volatile private var audioVolume=1f
    private val audioManager by lazy { getSystemService(AudioManager::class.java) }
    private val focusRequest by lazy { android.media.AudioFocusRequest.Builder(AudioManager.AUDIOFOCUS_GAIN)
        .setAudioAttributes(AudioAttributes.Builder().setUsage(AudioAttributes.USAGE_GAME).setContentType(AudioAttributes.CONTENT_TYPE_MUSIC).build())
        .setOnAudioFocusChangeListener { focus ->
            audioVolume=if(focus==AudioManager.AUDIOFOCUS_GAIN)1f else 0f
            if(focus!=AudioManager.AUDIOFOCUS_GAIN && ready) surface.suspendForInterruption()
        }.build() }
    private val noisyReceiver=object:android.content.BroadcastReceiver(){override fun onReceive(context:Context?,intent:Intent?){audioVolume=0f;if(ready)surface.suspendForInterruption()}}

    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)
        window.addFlags(WindowManager.LayoutParams.FLAG_KEEP_SCREEN_ON)
        if(Build.VERSION.SDK_INT>=28)window.attributes=window.attributes.apply{layoutInDisplayCutoutMode=WindowManager.LayoutParams.LAYOUT_IN_DISPLAY_CUTOUT_MODE_SHORT_EDGES}
        window.decorView.systemUiVisibility=5894
        volumeControlStream=AudioManager.STREAM_MUSIC
        vault=Vault(this)
        if(Build.VERSION.SDK_INT>=33) onBackInvokedDispatcher.registerOnBackInvokedCallback(android.window.OnBackInvokedDispatcher.PRIORITY_DEFAULT){if(ready)surface.back()else finish()}
        if(Build.VERSION.SDK_INT>=33)registerReceiver(noisyReceiver,android.content.IntentFilter(AudioManager.ACTION_AUDIO_BECOMING_NOISY),Context.RECEIVER_NOT_EXPORTED)
        else registerReceiver(noisyReceiver,android.content.IntentFilter(AudioManager.ACTION_AUDIO_BECOMING_NOISY))
        if(BuildConfig.OFFLINE_EDITION)openGame()else authenticate {openGame()}
    }
    private fun authenticate(done:()->Unit) {
        authContinuation=done
        val keyguard=getSystemService(KeyguardManager::class.java)
        if(!keyguard.isDeviceSecure){done();return}
        if(Build.VERSION.SDK_INT>=30){
            authSignal=CancellationSignal()
            BiometricPrompt.Builder(this).setTitle("Prononcer son Nom")
                .setSubtitle("Déverrouille ta lanterne HEXKEEP")
                .setAllowedAuthenticators(BiometricManager.Authenticators.BIOMETRIC_STRONG or BiometricManager.Authenticators.DEVICE_CREDENTIAL)
                .build().authenticate(authSignal!!,mainExecutor,object:BiometricPrompt.AuthenticationCallback(){
                    override fun onAuthenticationSucceeded(result:BiometricPrompt.AuthenticationResult?){done()}
                    override fun onAuthenticationError(code:Int,message:CharSequence?){if(!ready)finish()}
                })
        }else{
            startActivityForResult(keyguard.createConfirmDeviceCredentialIntent("HEXKEEP","Prononcer son Nom"),77)
        }
    }
    override fun onActivityResult(requestCode:Int,resultCode:Int,data:Intent?){
        super.onActivityResult(requestCode,resultCode,data)
        if(resultCode!=RESULT_OK){if(requestCode==77&&!ready)finish();return}
        try{when(requestCode){
            78->data?.data?.let{uri->contentResolver.openOutputStream(uri)?.use{it.write(engine.proof().toByteArray())}}
            79->data?.data?.let{uri->contentResolver.openInputStream(uri)?.use{stream->val out=java.io.ByteArrayOutputStream();val chunk=ByteArray(8192);while(out.size()<=16*1024*1024){val n=stream.read(chunk);if(n<0)break;out.write(chunk,0,n)};val bytes=out.toByteArray();if(bytes.size>16*1024*1024)engine.notice("Dossier trop volumineux.")else engine.importExchange(bytes.toString(Charsets.UTF_8))}}
            77->authContinuation?.invoke()
        }}catch(e:Exception){if(ready)engine.notice("Fichier inaccessible : aucun état remplacé.")}
    }
    private fun openGame() {
        var failed=false
        val snapshot=try{vault.read()}catch(e:Exception){failed=true;Log.e("HEXKEEP","Encrypted save could not be opened",e);""}
        engine=if(BuildConfig.OFFLINE_EDITION)Engine.offline(snapshot)else GameRuntime.engine?.takeIf{it.phare()}?:Engine(snapshot,BuildConfig.DEV_NETWORK).also{if(it.phare())it.stopPhare("Phare interrompu : réactive-le après la réouverture.")}
        GameRuntime.engine=engine
        if(!BuildConfig.OFFLINE_EDITION)try{DeviceIdentity.certify(this,engine)}catch(e:Exception){engine.notice("Keystore indisponible : identité DEV active.")}
        if(failed)engine.setStorageError()
        surface=NightSurface(this,engine,::persist,::action,::feedback,::protect)
        setContentView(surface);ready=true
        if(resumed){startAudio();if(!BuildConfig.OFFLINE_EDITION)resumeGps()}
    }
    fun persist() { synchronized(GameRuntime.saveLock) {
        if(!ready||!engine.canSave())return
        try{vault.write(engine.snapshot())}catch(e:Exception){Log.e("HEXKEEP","Save failed",e);engine.retrySave();engine.notice("Sauvegarde impossible : déverrouille l'appareil.")}
    }
    }
    fun renderMetrics():String=surface.metrics()
    private fun protect(sensitive:Boolean){if(sensitive==protectedWindow)return;protectedWindow=sensitive;runOnUiThread{if(sensitive)window.addFlags(WindowManager.LayoutParams.FLAG_SECURE)else window.clearFlags(WindowManager.LayoutParams.FLAG_SECURE)}}
    private fun action(id:Int){runOnUiThread{
        if(BuildConfig.OFFLINE_EDITION && id !in listOf(15,16,17,19))return@runOnUiThread
        when(id){
        16->showReading("Vie privée",if(BuildConfig.OFFLINE_EDITION)"privacy-play.txt"else"privacy-dev.txt")
        17->showReading("Crédits & licences","credits.txt")
        19->{audioVolume=if(audioManager.requestAudioFocus(focusRequest)==AudioManager.AUDIOFOCUS_REQUEST_GRANTED)1f else 0f}

        1->requestGps()
        2->authenticate{}
        3->{if(multicast==null){multicast=(applicationContext.getSystemService(Context.WIFI_SERVICE)as WifiManager).createMulticastLock("HEXKEEP peers").apply{setReferenceCounted(false);acquire()}}}
        6->startActivityForResult(Intent(Intent.ACTION_CREATE_DOCUMENT).addCategory(Intent.CATEGORY_OPENABLE).setType("application/json").putExtra(Intent.EXTRA_TITLE,"hexkeep-dossier.json"),78)
        8->authenticate{engine.throneUnlock()}
        9->startActivityForResult(Intent(Intent.ACTION_OPEN_DOCUMENT).addCategory(Intent.CATEGORY_OPENABLE).setType("application/json"),79)
        10->try{DeviceIdentity.certify(this,engine);engine.notice("Appareil certifié. Session renouvelée.")}catch(_:Exception){engine.notice("Clé matérielle indisponible en DEV.")}
        11->requestBle()
        12->{shop?.close();shop=PlayShop(this,engine).also{it.inspect(engine.nativeText())}}
        13->{if(engine.phare()){if(checkSelfPermission(Manifest.permission.ACCESS_FINE_LOCATION)!=PackageManager.PERMISSION_GRANTED){engine.stopPhare("Active le GPS avant de lancer le Phare.");requestGps()}else{try{startForegroundService(Intent(this,PhareService::class.java))}catch(_:Exception){engine.stopPhare("Le système refuse la veille en arrière-plan.")}}}else stopService(Intent(this,PhareService::class.java))}
        15->{getPreferences(0).edit().putBoolean("gps-enabled",false).apply();locationManager.removeUpdates(this);lastLocation=null}
        14->{val input=android.widget.EditText(this);input.hint="/ip4/.../tcp/.../p2p/...";android.app.AlertDialog.Builder(this).setTitle("Pair ou relais").setView(input).setPositiveButton("Connecter"){_,_->connectAddress(input.text.toString(),false)}.setNeutralButton("Réserver un relais"){_,_->connectAddress(input.text.toString(),true)}.setNegativeButton("Retour",null).show()}
    }}}
    private fun showReading(title:String,asset:String){
        val body=android.widget.TextView(this).apply{ text=assets.open(asset).bufferedReader().use{it.readText()};textSize=16f;setTextColor(android.graphics.Color.rgb(238,234,220));setPadding(32,24,32,24);setTextIsSelectable(true) }
        val scroll=android.widget.ScrollView(this).apply{addView(body);setBackgroundColor(android.graphics.Color.rgb(15,31,41))}
        val dialog=android.app.AlertDialog.Builder(this).setTitle(title).setView(scroll).setPositiveButton("Fermer",null)
        if(asset=="credits.txt")dialog.setNeutralButton("Composants tiers"){_,_->val files=assets.list("licenses")?:emptyArray();android.app.AlertDialog.Builder(this).setTitle("Licences des composants").setItems(files){_,which->showReading(files[which],"licenses/"+files[which])}.setNegativeButton("Fermer",null).show()}
        dialog.show()
    }
    private fun connectAddress(value:String,relay:Boolean){Thread({try{var address=value.trim();val parts=address.split("/");if(parts.size>3&&parts[1]in listOf("dns","dns4","dns6")){val resolved=java.net.InetAddress.getAllByName(parts[2]).firstOrNull{parts[1]=="dns"||(parts[1]=="dns4"&&it is java.net.Inet4Address)||(parts[1]=="dns6"&&it is java.net.Inet6Address)}?:throw IllegalArgumentException("DNS");address="/${if(resolved is java.net.Inet4Address)"ip4"else"ip6"}/${resolved.hostAddress}/"+parts.drop(3).joinToString("/")};if(relay)engine.reserveRelay(address)else engine.connect(address)}catch(_:Exception){engine.notice("Adresse indisponible. Vérifie le pair ou le relais.")}},"HEXKEEP system DNS").start()}
    private fun requestBle(){val permissions=if(Build.VERSION.SDK_INT>=31)arrayOf(Manifest.permission.BLUETOOTH_SCAN,Manifest.permission.BLUETOOTH_ADVERTISE,Manifest.permission.BLUETOOTH_CONNECT)else arrayOf(Manifest.permission.ACCESS_FINE_LOCATION);if(permissions.any{checkSelfPermission(it)!=PackageManager.PERMISSION_GRANTED}){requestPermissions(permissions,32);return};ble?.stop();ble=BleLantern(this,engine).also{it.start()}}
    private fun feedback(){runOnUiThread{getSystemService(Vibrator::class.java)?.vibrate(VibrationEffect.createOneShot(18,80))}}
    private fun requestGps(){
        if(checkSelfPermission(Manifest.permission.ACCESS_FINE_LOCATION)!=PackageManager.PERMISSION_GRANTED){requestPermissions(arrayOf(Manifest.permission.ACCESS_FINE_LOCATION,Manifest.permission.ACCESS_COARSE_LOCATION),31);return}
        try{locationManager.requestLocationUpdates(LocationManager.GPS_PROVIDER,3000,4f,this);getPreferences(0).edit().putBoolean("gps-enabled",true).apply();engine.notice("Recherche GPS. La partie continue hors ligne.")}
        catch(e:Exception){engine.notice("GPS indisponible. La carte DEV reste jouable.")}
    }
    private fun resumeGps(){if(getPreferences(0).getBoolean("gps-enabled",false)&&checkSelfPermission(Manifest.permission.ACCESS_FINE_LOCATION)==PackageManager.PERMISSION_GRANTED){lastLocation=null;requestGps()}}
    override fun onRequestPermissionsResult(requestCode:Int,permissions:Array<out String>,results:IntArray){super.onRequestPermissionsResult(requestCode,permissions,results);if(requestCode==32){if(results.all{it==PackageManager.PERMISSION_GRANTED})requestBle()else engine.bleStatus("Co-présence BLE non autorisée.")};if(requestCode==31){if(results.isNotEmpty()&&results[0]==PackageManager.PERMISSION_GRANTED)requestGps()else engine.notice("GPS refusé : utilise la carte de développement.")}}
    override fun onLocationChanged(location:Location){
        if(!ready||!resumed||location.accuracy>100f)return
        engine.memorySpeed((location.speed.coerceAtLeast(0f)*1000).toUInt())
        val old=lastLocation
        if(old!=null){val seconds=(location.elapsedRealtimeNanos-old.elapsedRealtimeNanos)/1_000_000_000.0;if(seconds<=0)return;if(old.distanceTo(location)/seconds>8){engine.notice("Déplacement trop rapide : position ignorée.");return}}
        lastLocation=location
        engine.location((location.latitude*1e7).toInt(),(location.longitude*1e7).toInt(),if(Build.VERSION.SDK_INT>=31)location.isMock else location.isFromMockProvider)
    }
    override fun onProviderDisabled(provider:String){if(ready)engine.notice("GPS désactivé. La carte DEV reste jouable.")}
    override fun onProviderEnabled(provider:String){}
    @Deprecated("Platform callback") override fun onStatusChanged(provider:String?,status:Int,extras:Bundle?){}
    private fun startAudio(){
        if(!ready||audioRunning.get())return
        audioVolume=if(audioManager.requestAudioFocus(focusRequest)==AudioManager.AUDIOFOCUS_REQUEST_GRANTED)1f else 0f
        val running=AtomicBoolean(true);audioRunning=running
        Thread({
            android.os.Process.setThreadPriority(android.os.Process.THREAD_PRIORITY_AUDIO)
            val track=AudioTrack.Builder().setAudioAttributes(AudioAttributes.Builder().setUsage(AudioAttributes.USAGE_GAME).setContentType(AudioAttributes.CONTENT_TYPE_MUSIC).build())
                .setAudioFormat(AudioFormat.Builder().setSampleRate(44100).setChannelMask(AudioFormat.CHANNEL_OUT_MONO).setEncoding(AudioFormat.ENCODING_PCM_16BIT).build())
                .setBufferSizeInBytes(maxOf(8820,AudioTrack.getMinBufferSize(44100,AudioFormat.CHANNEL_OUT_MONO,AudioFormat.ENCODING_PCM_16BIT)))
                .setTransferMode(AudioTrack.MODE_STREAM).build()
            try{track.play();while(running.get()){track.setVolume(audioVolume);val pcm=engine.audio(1470u);track.write(pcm,0,pcm.size)}}catch(e:Exception){Log.w("HEXKEEP","Audio interrupted",e)}finally{track.stop();track.release()}
        },"HEXKEEP audio").start()
    }
    override fun onPause(){resumed=false;GameRuntime.foreground=false;audioRunning.set(false);if(ready){engine.touch(0,3u,0,0);if(!engine.phare()){engine.pauseNetwork();ble?.stop()};multicast?.let{if(it.isHeld)it.release()};multicast=null;surface.onPause();persist()};if(!BuildConfig.OFFLINE_EDITION)locationManager.removeUpdates(this);audioManager.abandonAudioFocusRequest(focusRequest);super.onPause()}
    override fun onResume(){super.onResume();resumed=true;GameRuntime.foreground=true;window.decorView.systemUiVisibility=5894;if(ready){surface.onResume();startAudio();if(!BuildConfig.OFFLINE_EDITION)resumeGps()}}
    override fun onDestroy(){if(ready)surface.close();shop?.close();ble?.stop();audioRunning.set(false);authSignal?.cancel();unregisterReceiver(noisyReceiver);super.onDestroy()}
    // API 33+ is handled by the OnBackInvokedDispatcher registered in onCreate.
    @android.annotation.SuppressLint("GestureBackNavigation")
    @Deprecated("Back compatibility") override fun onBackPressed(){if(ready){surface.back()}else super.onBackPressed()}
}
