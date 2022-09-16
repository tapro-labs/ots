module Logger
    ( initLogger
    , initLoggerMiddleware
    , loggerName
    ) where

import           Network.Wai                          (Middleware)
import           Network.Wai.Middleware.RequestLogger (logStdout, logStdoutDev)
import           System.Log.Logger                    (Priority (..), setLevel, updateGlobalLogger)

loggerName :: String
loggerName = "OTS Backend"

initLogger :: Priority -> IO ()
initLogger = updateGlobalLogger loggerName . setLevel

initLoggerMiddleware :: Priority -> Middleware
initLoggerMiddleware priority = if priority == DEBUG then
                                  logStdoutDev
                                else
                                  logStdout
