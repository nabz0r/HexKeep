package game.hexkeep
import android.app.Activity
import game.hexkeep.core.Engine
import org.json.JSONObject
import com.android.billingclient.api.*

/** Billing is real API integration; DEV never launches a financial flow. */
class PlayShop(private val activity:Activity,private val engine:Engine):PurchasesUpdatedListener {
    private val client=BillingClient.newBuilder(activity).setListener(this).enablePendingPurchases(PendingPurchasesParams.newBuilder().enableOneTimeProducts().build()).enableAutoServiceReconnection().build()
    fun inspect(product:String){client.startConnection(object:BillingClientStateListener{override fun onBillingServiceDisconnected(){engine.notice("Google Play déconnecté.")};override fun onBillingSetupFinished(result:BillingResult){if(result.responseCode!=BillingClient.BillingResponseCode.OK){engine.notice("Catalogue Play indisponible. Boutique DEV gratuite.");return};val type=if(product=="patron_month")BillingClient.ProductType.SUBS else BillingClient.ProductType.INAPP;client.queryProductDetailsAsync(QueryProductDetailsParams.newBuilder().setProductList(listOf(QueryProductDetailsParams.Product.newBuilder().setProductId(product).setProductType(type).build())).build()){r,details->if(r.responseCode!=BillingClient.BillingResponseCode.OK||details.productDetailsList.isEmpty()){engine.notice("Produit non publié sur Play. Aucun débit.")}else if(BuildConfig.DEV_NETWORK){engine.notice("Produit Play trouvé. Paiement désactivé en DEV.")}else{val item=details.productDetailsList.first();val params=BillingFlowParams.ProductDetailsParams.newBuilder().setProductDetails(item);item.subscriptionOfferDetails?.firstOrNull()?.offerToken?.let{params.setOfferToken(it)};client.launchBillingFlow(activity,BillingFlowParams.newBuilder().setObfuscatedAccountId(engine.identityPublic().joinToString(""){"%02x".format(it)}).setProductDetailsParamsList(listOf(params.build())).build())}}}})}
    override fun onPurchasesUpdated(result:BillingResult,purchases:MutableList<Purchase>?){if(result.responseCode!=BillingClient.BillingResponseCode.OK)return;for(p in purchases.orEmpty()){if(p.purchaseState!=Purchase.PurchaseState.PURCHASED)continue;val signed=JSONObject(p.originalJson);val account=signed.optString("obfuscatedAccountId","");val expected=engine.identityPublic().joinToString(""){"%02x".format(it)};if(account!=expected){engine.notice("Reçu à présenter au Trône : compte non signé.");continue};engine.notice("Reçu à présenter au Trône pour attribution.")}}
    fun close(){client.endConnection()}
}
