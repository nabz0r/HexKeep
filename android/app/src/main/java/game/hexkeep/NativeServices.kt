package game.hexkeep

import android.Manifest
import android.app.*
import android.bluetooth.*
import android.bluetooth.le.*
import android.content.*
import android.content.pm.PackageManager
import android.location.*
import android.net.*
import android.os.*
import android.security.keystore.*
import android.util.Base64
import game.hexkeep.core.Engine
import org.json.JSONArray
import org.json.JSONObject
import java.security.*
import java.security.spec.ECGenParameterSpec
import java.util.concurrent.Executors
import java.util.concurrent.TimeUnit
import com.android.billingclient.api.*

object GameRuntime { @Volatile var engine:Engine?=null; @Volatile var foreground=true }

object DeviceIdentity {
    fun certify(context:Context,engine:Engine):Boolean {
        val nom=engine.identityPublic().joinToString(""){"%02x".format(it)}
        val alias="hexkeep-device-$nom"
        val store=KeyStore.getInstance("AndroidKeyStore").apply{load(null)}
        if(!store.containsAlias(alias)) {
            fun generate(strong:Boolean,attest:Boolean){val spec=KeyGenParameterSpec.Builder(alias,KeyProperties.PURPOSE_SIGN or KeyProperties.PURPOSE_VERIFY)
                .setAlgorithmParameterSpec(ECGenParameterSpec("secp256r1")).setDigests(KeyProperties.DIGEST_SHA256)
                .apply{if(attest)setAttestationChallenge(engine.identityChallenge());if(Build.VERSION.SDK_INT>=28&&strong)setIsStrongBoxBacked(true)}.build()
                KeyPairGenerator.getInstance(KeyProperties.KEY_ALGORITHM_EC,"AndroidKeyStore").apply{initialize(spec)}.generateKeyPair()}
            try{generate(Build.VERSION.SDK_INT>=28&&context.packageManager.hasSystemFeature(PackageManager.FEATURE_STRONGBOX_KEYSTORE),true)}
            catch(e:Exception){try{generate(false,true)}catch(e:Exception){if(!BuildConfig.DEV_NETWORK)throw e;generate(false,false)}}
        }
        val chain=store.getCertificateChain(alias).map{cert->JSONArray().apply{cert.encoded.forEach{put(it.toInt()and 255)}}}
        val payload=engine.sessionRequest(store.getCertificate(alias).publicKey.encoded,JSONArray(chain).toString())
        val signature=Signature.getInstance("SHA256withECDSA").apply{initSign(store.getKey(alias,null)as PrivateKey);update(payload)}.sign()
        return engine.sessionCertify(signature)
    }
}

class BleLantern(private val context:Context,private val engine:Engine) {
    private val manager=context.getSystemService(BluetoothManager::class.java)
    private val uuid=ParcelUuid.fromString("95e148f0-67e2-4bc1-bb53-89e22d40a716")
    private var scanning=false
    private val handler=Handler(Looper.getMainLooper())
    private val scan=object:ScanCallback(){override fun onScanResult(type:Int,result:ScanResult){result.scanRecord?.getManufacturerSpecificData(0xFFFF)?.let{engine.bleObserved(it)}};override fun onScanFailed(code:Int){engine.bleStatus("Bluetooth : balayage indisponible ($code)")}}
    private val advertise=object:AdvertiseCallback(){override fun onStartFailure(code:Int){engine.bleStatus("Bluetooth : balise indisponible ($code)")}}
    private val rotation=object:Runnable{override fun run(){if(!scanning)return;try{manager.adapter?.bluetoothLeAdvertiser?.stopAdvertising(advertise);advertise()}catch(_:Exception){};handler.postDelayed(this,60000)}}
    private fun advertise(){manager.adapter?.bluetoothLeAdvertiser?.startAdvertising(AdvertiseSettings.Builder().setAdvertiseMode(AdvertiseSettings.ADVERTISE_MODE_LOW_LATENCY).setConnectable(false).setTxPowerLevel(AdvertiseSettings.ADVERTISE_TX_POWER_LOW).build(),AdvertiseData.Builder().addManufacturerData(0xFFFF,engine.beacon()).build(),advertise)}
    fun start(){if(scanning)return;try{val adapter=manager?.adapter;if(adapter==null||!adapter.isEnabled){engine.bleStatus("Active le Bluetooth pour la co-présence.");return};adapter.bluetoothLeScanner.startScan(listOf(ScanFilter.Builder().setManufacturerData(0xFFFF,byteArrayOf()).build()),ScanSettings.Builder().setScanMode(ScanSettings.SCAN_MODE_LOW_LATENCY).build(),scan);scanning=true;advertise();handler.postDelayed(rotation,60000);engine.bleStatus("Balises BLE actives • jetons renouvelés") }catch(_:SecurityException){engine.bleStatus("Autorisation Bluetooth nécessaire.")}catch(_:Exception){engine.bleStatus("Bluetooth non disponible sur cet appareil.")}}
    fun stop(){handler.removeCallbacks(rotation);if(!scanning)return;scanning=false;try{manager.adapter?.bluetoothLeScanner?.stopScan(scan);manager.adapter?.bluetoothLeAdvertiser?.stopAdvertising(advertise)}catch(_:Exception){}}
}

class PhareService:Service(),LocationListener {
    private val worker=Executors.newSingleThreadScheduledExecutor()
    private var ble:BleLantern?=null
    private var multicast:android.net.wifi.WifiManager.MulticastLock?=null
    private var anchor:Location?=null
    private var seconds=0
    private var watching=false
    private val locations by lazy{getSystemService(LocationManager::class.java)}
    override fun onBind(intent:Intent?)=null
    override fun onCreate(){super.onCreate();getSystemService(NotificationManager::class.java).createNotificationChannel(NotificationChannel("watch","Veille de la lanterne",NotificationManager.IMPORTANCE_LOW))}
    override fun onStartCommand(intent:Intent?,flags:Int,startId:Int):Int{val engine=GameRuntime.engine?:run{stopSelf();return START_NOT_STICKY};val pending=PendingIntent.getActivity(this,0,Intent(this,MainActivity::class.java),PendingIntent.FLAG_IMMUTABLE or PendingIntent.FLAG_UPDATE_CURRENT);val stop=PendingIntent.getService(this,1,Intent(this,PhareService::class.java).setAction("STOP"),PendingIntent.FLAG_IMMUTABLE or PendingIntent.FLAG_UPDATE_CURRENT)
        if(intent?.action=="STOP"){engine.stopPhare("Phare arrêté.");stopSelf();return START_NOT_STICKY}
        if(watching)return START_NOT_STICKY
        watching=true
        startForeground(12,Notification.Builder(this,"watch").setSmallIcon(game.hexkeep.R.drawable.icon).setContentTitle("HEXKEEP • le Phare veille").setContentText("Ancre fixe • secteur et Wi-Fi requis").setContentIntent(pending).addAction(Notification.Action.Builder(null,"Arrêter",stop).build()).setOngoing(true).build())
        multicast=(applicationContext.getSystemService(Context.WIFI_SERVICE)as android.net.wifi.WifiManager).createMulticastLock("HEXKEEP Phare").apply{setReferenceCounted(false);acquire()}
        ble=BleLantern(this,engine).also{it.start()}
        try{locations.requestLocationUpdates(LocationManager.GPS_PROVIDER,3000,2f,this)}catch(_:Exception){}
        worker.scheduleAtFixedRate({try{if(!engine.phare()){stopSelf();return@scheduleAtFixedRate};if(!GameRuntime.foreground){engine.tick((System.currentTimeMillis()/1000).toULong())};seconds++;if(seconds%60==0){val battery=registerReceiver(null,IntentFilter(Intent.ACTION_BATTERY_CHANGED));val plugged=(battery?.getIntExtra(BatteryManager.EXTRA_PLUGGED,0)?:0)!=0;val net=getSystemService(ConnectivityManager::class.java);val wifi=net.getNetworkCapabilities(net.activeNetwork)?.hasTransport(NetworkCapabilities.TRANSPORT_WIFI)==true;if(!plugged||!wifi){engine.stopPhare("Phare arrêté : secteur et Wi-Fi requis.");stopSelf()};if(!GameRuntime.foreground&&engine.dirty())Vault(this).write(engine.snapshot())}}catch(_:Exception){engine.stopPhare("La veille s'est arrêtée.");stopSelf()}},0,33,TimeUnit.MILLISECONDS)
        return START_NOT_STICKY}
    override fun onLocationChanged(location:Location){val engine=GameRuntime.engine?:return;val a=anchor;if(a==null)anchor=location else if(a.distanceTo(location)>200){engine.stopPhare("Ancre déplacée de plus de 200 m.");stopSelf();return};engine.memorySpeed((location.speed.coerceAtLeast(0f)*1000).toUInt())}
    override fun onDestroy(){worker.shutdownNow();multicast?.let{if(it.isHeld)it.release()};ble?.stop();locations.removeUpdates(this);GameRuntime.engine?.let{it.stopPhare("Phare arrêté.");if(!GameRuntime.foreground)it.pauseNetwork();try{Vault(this).write(it.snapshot())}catch(_:Exception){}};super.onDestroy()}
    override fun onProviderEnabled(provider:String){};override fun onProviderDisabled(provider:String){}
}

/** Billing is real API integration; DEV never launches a financial flow. */
class PlayShop(private val activity:Activity,private val engine:Engine):PurchasesUpdatedListener {
    private val client=BillingClient.newBuilder(activity).setListener(this).enablePendingPurchases(PendingPurchasesParams.newBuilder().enableOneTimeProducts().build()).enableAutoServiceReconnection().build()
    fun inspect(product:String){client.startConnection(object:BillingClientStateListener{override fun onBillingServiceDisconnected(){engine.notice("Google Play déconnecté.")};override fun onBillingSetupFinished(result:BillingResult){if(result.responseCode!=BillingClient.BillingResponseCode.OK){engine.notice("Catalogue Play indisponible. Boutique DEV gratuite.");return};val type=if(product=="patron_month")BillingClient.ProductType.SUBS else BillingClient.ProductType.INAPP;client.queryProductDetailsAsync(QueryProductDetailsParams.newBuilder().setProductList(listOf(QueryProductDetailsParams.Product.newBuilder().setProductId(product).setProductType(type).build())).build()){r,details->if(r.responseCode!=BillingClient.BillingResponseCode.OK||details.productDetailsList.isEmpty()){engine.notice("Produit non publié sur Play. Aucun débit.")}else if(BuildConfig.DEV_NETWORK){engine.notice("Produit Play trouvé. Paiement désactivé en DEV.")}else{val item=details.productDetailsList.first();val params=BillingFlowParams.ProductDetailsParams.newBuilder().setProductDetails(item);item.subscriptionOfferDetails?.firstOrNull()?.offerToken?.let{params.setOfferToken(it)};client.launchBillingFlow(activity,BillingFlowParams.newBuilder().setObfuscatedAccountId(engine.identityPublic().joinToString(""){"%02x".format(it)}).setProductDetailsParamsList(listOf(params.build())).build())}}}})}
    override fun onPurchasesUpdated(result:BillingResult,purchases:MutableList<Purchase>?){if(result.responseCode!=BillingClient.BillingResponseCode.OK)return;for(p in purchases.orEmpty()){if(p.purchaseState!=Purchase.PurchaseState.PURCHASED)continue;val signed=JSONObject(p.originalJson);val account=signed.optString("obfuscatedAccountId","");val expected=engine.identityPublic().joinToString(""){"%02x".format(it)};if(account!=expected){engine.notice("Reçu à présenter au Trône : compte non signé.");continue};engine.notice("Reçu à présenter au Trône pour attribution.")}}
    fun close(){client.endConnection()}
}
