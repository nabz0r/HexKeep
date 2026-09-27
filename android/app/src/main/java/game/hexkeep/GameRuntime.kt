package game.hexkeep
import game.hexkeep.core.Engine
object GameRuntime { @Volatile var engine:Engine?=null; @Volatile var foreground=true; val saveLock=Any() }
