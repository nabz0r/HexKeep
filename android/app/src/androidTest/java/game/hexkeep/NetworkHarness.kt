package game.hexkeep

import androidx.test.core.app.ActivityScenario
import androidx.test.ext.junit.runners.AndroidJUnit4
import androidx.test.platform.app.InstrumentationRegistry
import android.content.Intent
import android.os.SystemClock
import org.json.JSONObject
import org.junit.Assert.*
import org.junit.Assume.assumeTrue
import org.junit.Test
import org.junit.runner.RunWith
import java.io.File

@RunWith(AndroidJUnit4::class)
class NetworkHarness {
    @Test fun androidPeerSoak(){
        val instrumentation=InstrumentationRegistry.getInstrumentation()
        val address=InstrumentationRegistry.getArguments().getString("peerAddress")
        assumeTrue("Requires the external peer harness",address!=null)
        val output=File(instrumentation.targetContext.getExternalFilesDir(null),"network-checks.jsonl")
        output.writeText("")
        ActivityScenario.launch<MainActivity>(Intent(instrumentation.targetContext,MainActivity::class.java)).use{scenario->
            SystemClock.sleep(800)
            scenario.onActivity{it.engine.connect(address!!)}
            var last=0;var participants=0;val until=SystemClock.elapsedRealtime()+65000
            while(SystemClock.elapsedRealtime()<until){
                var json=""
                scenario.onActivity{json=it.engine.networkReport()}
                val state=JSONObject(json)
                assertTrue("Network failure: $json",state.isNull("error"))
                participants=maxOf(participants,state.getInt("participants"))
                val tick=state.getInt("tick")
                if(tick>last){output.appendText(state.put("elapsed_ms",SystemClock.elapsedRealtime()).toString()+"\n");last=tick}
                SystemClock.sleep(100)
            }
            scenario.onActivity{File(it.getExternalFilesDir(null),"v02-network-render.json").writeText(it.renderMetrics())}
            assertEquals(10,participants)
            assertTrue("Too few ticks: $last",last>=1200)
        }
    }
}
