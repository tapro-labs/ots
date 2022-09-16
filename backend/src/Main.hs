{-# LANGUAGE DataKinds         #-}
{-# LANGUAGE OverloadedStrings #-}
{-# LANGUAGE TypeOperators     #-}

module Main
    ( main
    ) where

import qualified CommandLineOptions          as CLO (MainOptions (..), mainOptionsParserExec)
import           Control.Monad               (void)
import           Data.String                 (IsString (fromString))
import           Data.Text
import           Logger                      (initLogger, initLoggerMiddleware)
import           Network.Wai                 (Application)
import           Network.Wai.Handler.Warp    (defaultSettings, runSettings, setBeforeMainLoop,
                                              setHost, setPort)
import           Network.Wai.Middleware.Cors (CorsResourcePolicy (..), cors,
                                              simpleCorsResourcePolicy)
import           SecretManager.API           (SecretManagerAPI)
import qualified SecretManager.API           as SecretManagerAPI
import           Servant                     (PlainText, serve, (:>))
import qualified Servant
import           Servant.API.Alternative

type BaseAPI = Servant.Get '[Servant.PlainText] Text
type API = BaseAPI :<|> ("api" :> SecretManagerAPI)

corsResourcePolicy :: CorsResourcePolicy
corsResourcePolicy = simpleCorsResourcePolicy { corsRequestHeaders = ["Content-Type"] }

baseApiServer :: Servant.Server BaseAPI
baseApiServer = getBase
  where  getBase :: Servant.Handler Text
         getBase = return "Hello World!"

api :: Servant.Proxy API
api = Servant.Proxy

app :: CLO.MainOptions -> Application
app options = initLoggerMiddleware logLevel $
  cors (const $ Just corsResourcePolicy) $
    serve api $
      baseApiServer :<|> SecretManagerAPI.server

  where logLevel = CLO.logLevel options

printPort :: Int -> IO ()
printPort port = putStrLn $ "Listening on port " ++ show port

main :: IO ()
main = CLO.mainOptionsParserExec bootstrap

bootstrap :: CLO.MainOptions -> IO ()
bootstrap options = do
  void $ initLogger logLevel
  let settings =
        setPort port $
        setHost (fromString host) $
        setBeforeMainLoop (printPort port) defaultSettings
  runSettings settings $ app options

  where port = CLO.port options
        host = CLO.host options
        logLevel = CLO.logLevel options
