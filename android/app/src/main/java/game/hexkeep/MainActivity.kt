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

class Vault(private val activity: Activity) {
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
    @Synchronized fun read():String {
        if(!file.baseFile.exists()) return ""
        val bytes=file.readFully()
        require(bytes.size>29 && bytes[0]==1.toByte()) { "Format de sauvegarde incorrect" }
        val cipher=Cipher.getInstance("AES/GCM/NoPadding")
        cipher.init(Cipher.DECRYPT_MODE,key(),GCMParameterSpec(128,bytes.copyOfRange(1,13)))
        return cipher.doFinal(bytes.copyOfRange(13,bytes.size)).toString(Charsets.UTF_8)
    }
    @Synchronized fun write(snapshot:String) {
        val cipher=Cipher.getInstance("AES/GCM/NoPadding");cipher.init(Cipher.ENCRYPT_MODE,key())
        val encrypted=byteArrayOf(1)+cipher.iv+cipher.doFinal(snapshot.toByteArray(Charsets.UTF_8))
        val stream=file.startWrite()
        try {stream.write(encrypted);file.finishWrite(stream)}catch(e:Exception){file.failWrite(stream);throw e}
    }
}

class MainActivity : Activity(), LocationListener {
    internal lateinit var engine: Engine
    private lateinit var surface: GameSurface
    private lateinit var vault: Vault
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
    override fun onActivityResult(requestCode:Int,resultCode:Int,data:Intent?){super.onActivityResult(requestCode,resultCode,data);if(requestCode==78 && resultCode==RESULT_OK && data?.data!=null){contentResolver.openOutputStream(data.data!!)?.use{it.write(engine.proof().toByteArray())};return};if(requestCode==77){if(resultCode==RESULT_OK){if(!ready)openGame()}else if(!ready)finish()}}
    private fun openGame() {
        var failed=false
        val snapshot=try{vault.read()}catch(e:Exception){failed=true;Log.e("HEXKEEP","Encrypted save could not be opened",e);""}
        engine=Engine(snapshot,BuildConfig.DEV_NETWORK)
        if(failed)engine.setStorageError()
        surface=GameSurface(this,engine,::persist,::action,::feedback,::protect)
        setContentView(surface);ready=true
        if(resumed)startAudio()
    }
    internal fun persist() {
        if(!ready)return
        try{vault.write(engine.snapshot())}catch(e:Exception){Log.e("HEXKEEP","Save failed",e);engine.notice("Sauvegarde impossible : déverrouille l'appareil.")}
    }
    private fun protect(sensitive:Boolean){runOnUiThread{if(sensitive)window.addFlags(WindowManager.LayoutParams.FLAG_SECURE)else window.clearFlags(WindowManager.LayoutParams.FLAG_SECURE)}}
    private fun action(id:Int){runOnUiThread{when(id){1->requestGps();2->authenticate{};3->{if(multicast==null){multicast=(applicationContext.getSystemService(Context.WIFI_SERVICE)as WifiManager).createMulticastLock("HEXKEEP peers").apply{setReferenceCounted(false);acquire()}}};6->startActivityForResult(Intent(Intent.ACTION_CREATE_DOCUMENT).addCategory(Intent.CATEGORY_OPENABLE).setType("application/json").putExtra(Intent.EXTRA_TITLE,"hexkeep-rejeu.json"),78)}}}
    private fun feedback(){runOnUiThread{getSystemService(Vibrator::class.java)?.vibrate(VibrationEffect.createOneShot(18,80))}}
    private fun requestGps(){
        if(checkSelfPermission(Manifest.permission.ACCESS_FINE_LOCATION)!=PackageManager.PERMISSION_GRANTED){requestPermissions(arrayOf(Manifest.permission.ACCESS_FINE_LOCATION,Manifest.permission.ACCESS_COARSE_LOCATION),31);return}
        try{locationManager.requestLocationUpdates(LocationManager.GPS_PROVIDER,3000,4f,this);engine.notice("Recherche GPS. La partie continue hors ligne.")}
        catch(e:Exception){engine.notice("GPS indisponible. La carte DEV reste jouable.")}
    }
    override fun onRequestPermissionsResult(requestCode:Int,permissions:Array<out String>,results:IntArray){super.onRequestPermissionsResult(requestCode,permissions,results);if(requestCode==31){if(results.isNotEmpty()&&results[0]==PackageManager.PERMISSION_GRANTED)requestGps()else engine.notice("GPS refusé : utilise la carte de développement.")}}
    override fun onLocationChanged(location:Location){
        if(!ready||!resumed||location.accuracy>100f)return
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
    override fun onPause(){resumed=false;audioRunning.set(false);if(ready){engine.touch(0,3u,0,0);engine.pauseNetwork();multicast?.let{if(it.isHeld)it.release()};multicast=null;surface.onPause();if(engine.dirty())persist()};locationManager.removeUpdates(this);super.onPause()}
    override fun onResume(){super.onResume();resumed=true;window.decorView.systemUiVisibility=5894;if(ready){surface.onResume();startAudio()}}
    override fun onDestroy(){audioRunning.set(false);authSignal?.cancel();super.onDestroy()}
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
        val now=System.nanoTime();accumulator+=(now-last).coerceAtMost(166_666_665L);last=now
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
