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
import android.opengl.GLSurfaceView
import android.opengl.GLES20.*
import android.security.keystore.KeyGenParameterSpec
import android.security.keystore.KeyProperties
import android.util.AtomicFile
import android.util.Log
import android.view.KeyEvent
import android.view.MotionEvent
import android.view.WindowManager
import game.hexkeep.core.Engine
import java.io.File
import java.nio.ByteBuffer
import java.nio.ByteOrder
import java.security.KeyStore
import java.util.concurrent.atomic.AtomicBoolean
import javax.crypto.Cipher
import javax.crypto.KeyGenerator
import javax.crypto.SecretKey
import javax.crypto.spec.GCMParameterSpec
import javax.microedition.khronos.egl.EGLConfig
import javax.microedition.khronos.opengles.GL10

class Vault(private val activity: Context) {
    companion object { private val diskLock = Any() }
    private val store = KeyStore.getInstance("AndroidKeyStore").apply { load(null) }
    private val file = AtomicFile(File(activity.filesDir,"state.hk"))
    private val alias="hexkeep-state-v1"
    private fun key(): SecretKey {
        (store.getKey(alias,null) as? SecretKey)?.let { return it }
        val secure=activity.getSystemService(KeyguardManager::class.java).isDeviceSecure
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
    private lateinit var surface: GameSurface
    private lateinit var vault: Vault
    private var authContinuation:(()->Unit)?=null
    private var ble:BleLantern?=null
    private var shop:PlayShop?=null
    private var ready=false
    private var audioRunning=AtomicBoolean(false)
    private var authSignal: CancellationSignal?=null
    private var resumed=false
    private var multicast:WifiManager.MulticastLock?=null
    private val locationManager by lazy {getSystemService(LocationManager::class.java)}
    private var lastLocation:Location?=null
    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)
        window.addFlags(WindowManager.LayoutParams.FLAG_KEEP_SCREEN_ON)
        if(Build.VERSION.SDK_INT>=28)window.attributes=window.attributes.apply{layoutInDisplayCutoutMode=WindowManager.LayoutParams.LAYOUT_IN_DISPLAY_CUTOUT_MODE_SHORT_EDGES}
        window.decorView.systemUiVisibility=5894
        volumeControlStream=AudioManager.STREAM_MUSIC
        vault=Vault(this)
        authenticate {openGame()}
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
        engine=GameRuntime.engine?.takeIf{it.phare()}?:Engine(snapshot,BuildConfig.DEV_NETWORK).also{if(it.phare())it.stopPhare("Phare interrompu : réactive-le après la réouverture.")}
        GameRuntime.engine=engine
        try{DeviceIdentity.certify(this,engine)}catch(e:Exception){engine.notice("Keystore indisponible : identité DEV active.")}
        if(failed)engine.setStorageError()
        surface=GameSurface(this,engine,::persist,::action,::feedback,::protect)
        setContentView(surface);ready=true
        if(resumed)startAudio()
    }
    fun persist() {
        if(!ready)return
        try{vault.write(engine.snapshot())}catch(e:Exception){Log.e("HEXKEEP","Save failed",e);engine.notice("Sauvegarde impossible : déverrouille l'appareil.")}
    }
    fun renderMetrics():String=surface.pixel.metrics()
    private fun protect(sensitive:Boolean){runOnUiThread{if(sensitive)window.addFlags(WindowManager.LayoutParams.FLAG_SECURE)else window.clearFlags(WindowManager.LayoutParams.FLAG_SECURE)}}
    private fun action(id:Int){runOnUiThread{when(id){
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
        14->{val input=android.widget.EditText(this);input.hint="/ip4/.../tcp/.../p2p/...";android.app.AlertDialog.Builder(this).setTitle("Pair ou relais").setView(input).setPositiveButton("Connecter"){_,_->connectAddress(input.text.toString(),false)}.setNeutralButton("Réserver un relais"){_,_->connectAddress(input.text.toString(),true)}.setNegativeButton("Retour",null).show()}
    }}}
    private fun connectAddress(value:String,relay:Boolean){Thread({try{var address=value.trim();val parts=address.split("/");if(parts.size>3&&parts[1]in listOf("dns","dns4","dns6")){val resolved=java.net.InetAddress.getAllByName(parts[2]).firstOrNull{parts[1]=="dns"||(parts[1]=="dns4"&&it is java.net.Inet4Address)||(parts[1]=="dns6"&&it is java.net.Inet6Address)}?:throw IllegalArgumentException("DNS");address="/${if(resolved is java.net.Inet4Address)"ip4"else"ip6"}/${resolved.hostAddress}/"+parts.drop(3).joinToString("/")};if(relay)engine.reserveRelay(address)else engine.connect(address)}catch(_:Exception){engine.notice("Adresse indisponible. Vérifie le pair ou le relais.")}},"HEXKEEP system DNS").start()}
    private fun requestBle(){val permissions=if(Build.VERSION.SDK_INT>=31)arrayOf(Manifest.permission.BLUETOOTH_SCAN,Manifest.permission.BLUETOOTH_ADVERTISE,Manifest.permission.BLUETOOTH_CONNECT)else arrayOf(Manifest.permission.ACCESS_FINE_LOCATION);if(permissions.any{checkSelfPermission(it)!=PackageManager.PERMISSION_GRANTED}){requestPermissions(permissions,32);return};ble?.stop();ble=BleLantern(this,engine).also{it.start()}}
    private fun feedback(){runOnUiThread{getSystemService(Vibrator::class.java)?.vibrate(VibrationEffect.createOneShot(18,80))}}
    private fun requestGps(){
        if(checkSelfPermission(Manifest.permission.ACCESS_FINE_LOCATION)!=PackageManager.PERMISSION_GRANTED){requestPermissions(arrayOf(Manifest.permission.ACCESS_FINE_LOCATION,Manifest.permission.ACCESS_COARSE_LOCATION),31);return}
        try{locationManager.requestLocationUpdates(LocationManager.GPS_PROVIDER,3000,4f,this);engine.notice("Recherche GPS. La partie continue hors ligne.")}
        catch(e:Exception){engine.notice("GPS indisponible. La carte DEV reste jouable.")}
    }
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
        val running=AtomicBoolean(true);audioRunning=running
        Thread({
            android.os.Process.setThreadPriority(android.os.Process.THREAD_PRIORITY_AUDIO)
            val track=AudioTrack.Builder().setAudioAttributes(AudioAttributes.Builder().setUsage(AudioAttributes.USAGE_GAME).setContentType(AudioAttributes.CONTENT_TYPE_MUSIC).build())
                .setAudioFormat(AudioFormat.Builder().setSampleRate(22050).setChannelMask(AudioFormat.CHANNEL_OUT_MONO).setEncoding(AudioFormat.ENCODING_PCM_16BIT).build())
                .setBufferSizeInBytes(maxOf(4410,AudioTrack.getMinBufferSize(22050,AudioFormat.CHANNEL_OUT_MONO,AudioFormat.ENCODING_PCM_16BIT)))
                .setTransferMode(AudioTrack.MODE_STREAM).build()
            try{track.play();while(running.get()){val pcm=engine.audio(735u);track.write(pcm,0,pcm.size)}}catch(e:Exception){Log.w("HEXKEEP","Audio interrupted",e)}finally{track.stop();track.release()}
        },"HEXKEEP audio").start()
    }
    override fun onPause(){resumed=false;GameRuntime.foreground=false;audioRunning.set(false);if(ready){engine.touch(0,3u,0,0);if(!engine.phare()){engine.pauseNetwork();ble?.stop()};multicast?.let{if(it.isHeld)it.release()};multicast=null;surface.onPause();if(engine.dirty())persist()};locationManager.removeUpdates(this);super.onPause()}
    override fun onResume(){super.onResume();resumed=true;GameRuntime.foreground=true;window.decorView.systemUiVisibility=5894;if(ready){surface.onResume();startAudio()}}
    override fun onDestroy(){shop?.close();ble?.stop();audioRunning.set(false);authSignal?.cancel();super.onDestroy()}
    @Deprecated("Back compatibility") override fun onBackPressed(){if(ready){engine.back()}else super.onBackPressed()}
}

class GameSurface(activity:Activity,private val engine:Engine,save:()->Unit,action:(Int)->Unit,haptic:()->Unit,secure:(Boolean)->Unit):GLSurfaceView(activity){
    val pixel=PixelRenderer(engine,save,action,haptic,secure)
    init{setEGLContextClientVersion(2);preserveEGLContextOnPause=true;setRenderer(pixel);isFocusable=true;isFocusableInTouchMode=true}
    override fun onTouchEvent(event:MotionEvent):Boolean{
        val action=event.actionMasked
        if(action==MotionEvent.ACTION_CANCEL){engine.touch(0,3u,0,0);return true}
        for(i in 0 until event.pointerCount){if(action!=MotionEvent.ACTION_MOVE&&i!=event.actionIndex)continue
            val phase:UByte=when(action){MotionEvent.ACTION_DOWN,MotionEvent.ACTION_POINTER_DOWN->0u;MotionEvent.ACTION_UP,MotionEvent.ACTION_POINTER_UP->2u;else->1u}
            val x=((event.getX(i)-pixel.left)/pixel.scale).toInt();val y=((event.getY(i)-pixel.top)/pixel.scale).toInt()
            engine.touch(event.getPointerId(i),phase,x,y)
        };return true
    }
}

class PixelRenderer(private val engine:Engine,private val save:()->Unit,private val action:(Int)->Unit,private val haptic:()->Unit,private val secure:(Boolean)->Unit):GLSurfaceView.Renderer{
    @Volatile var logicalWidth=520;private set
    @Volatile var scale=1;private set
    @Volatile var left=0;private set
    @Volatile var top=0;private set
    private var physicalHeight=0
    private var program=0;private var texture=0;private var last=0L;private var accumulator=0L;private var frames=0;private var protected=false
    private var metricStart=0L
    private var metricElapsed=0L
    private var metricFrames=0L
    private var slowFrames=0L
    @Synchronized fun metrics():String=org.json.JSONObject().put("frames",metricFrames).put("seconds",if(metricStart==0L)0.0 else metricElapsed/1e9).put("over_33ms",slowFrames).toString()
    private var buffer:ByteBuffer=ByteBuffer.allocateDirect(640*240*4)
    private val vertices=ByteBuffer.allocateDirect(64).order(ByteOrder.nativeOrder()).asFloatBuffer().apply{put(floatArrayOf(-1f,-1f,0f,1f,1f,-1f,1f,1f,-1f,1f,0f,0f,1f,1f,1f,0f));position(0)}
    override fun onSurfaceCreated(gl:GL10?,config:EGLConfig?){
        fun shader(type:Int,src:String):Int{val id=glCreateShader(type);glShaderSource(id,src);glCompileShader(id);val ok=IntArray(1);glGetShaderiv(id,GL_COMPILE_STATUS,ok,0);check(ok[0]!=0){glGetShaderInfoLog(id)};return id}
        program=glCreateProgram();glAttachShader(program,shader(GL_VERTEX_SHADER,"attribute vec2 p;attribute vec2 t;varying vec2 uv;void main(){gl_Position=vec4(p,0.,1.);uv=t;}"));glAttachShader(program,shader(GL_FRAGMENT_SHADER,"precision mediump float;varying vec2 uv;uniform sampler2D tex;void main(){gl_FragColor=texture2D(tex,uv);}"));glLinkProgram(program)
        val ids=IntArray(1);glGenTextures(1,ids,0);texture=ids[0];glBindTexture(GL_TEXTURE_2D,texture)
        glTexParameteri(GL_TEXTURE_2D,GL_TEXTURE_MIN_FILTER,GL_NEAREST);glTexParameteri(GL_TEXTURE_2D,GL_TEXTURE_MAG_FILTER,GL_NEAREST);glTexParameteri(GL_TEXTURE_2D,GL_TEXTURE_WRAP_S,GL_CLAMP_TO_EDGE);glTexParameteri(GL_TEXTURE_2D,GL_TEXTURE_WRAP_T,GL_CLAMP_TO_EDGE)
        glClearColor(0f,0f,0f,1f);last=System.nanoTime()
    }
    override fun onSurfaceChanged(gl:GL10?,w:Int,h:Int){scale=minOf(h/240,w/400).coerceAtLeast(1);logicalWidth=(w/scale/8*8).coerceIn(400,640);left=(w-logicalWidth*scale)/2;top=(h-240*scale)/2;physicalHeight=h;last=System.nanoTime();accumulator=0
        glBindTexture(GL_TEXTURE_2D,texture);glTexImage2D(GL_TEXTURE_2D,0,GL_RGBA,logicalWidth,240,0,GL_RGBA,GL_UNSIGNED_BYTE,null)
    }
    override fun onDrawFrame(gl:GL10?){
        val now=System.nanoTime();if(metricStart==0L)metricStart=now;metricFrames++;metricElapsed+=(now-last).coerceIn(0,100_000_000L);if(now-last>33_333_333L)slowFrames++;accumulator+=(now-last).coerceAtMost(166_666_665L);last=now
        while(accumulator>=33_333_333L){engine.tick((System.currentTimeMillis()/1000).toULong());accumulator-=33_333_333L}
        val data=engine.frame(logicalWidth);buffer.clear();buffer.put(data);buffer.position(0)
        glClear(GL_COLOR_BUFFER_BIT);glViewport(left,physicalHeight-top-240*scale,logicalWidth*scale,240*scale)
        glUseProgram(program);glBindTexture(GL_TEXTURE_2D,texture);glTexSubImage2D(GL_TEXTURE_2D,0,0,0,logicalWidth,240,GL_RGBA,GL_UNSIGNED_BYTE,buffer)
        val p=glGetAttribLocation(program,"p");val t=glGetAttribLocation(program,"t");vertices.position(0);glEnableVertexAttribArray(p);glVertexAttribPointer(p,2,GL_FLOAT,false,16,vertices);vertices.position(2);glEnableVertexAttribArray(t);glVertexAttribPointer(t,2,GL_FLOAT,false,16,vertices);glDrawArrays(GL_TRIANGLE_STRIP,0,4)
        val sensitive=engine.sensitive();if(sensitive!=protected){protected=sensitive;secure(sensitive)}
        val a=engine.action().toInt();if(a!=0)action(a)
        if(engine.haptic().toInt()!=0)haptic()
        frames++;if(frames%60==0&&engine.dirty())save()
    }
}
