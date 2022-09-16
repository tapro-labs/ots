{-# LANGUAGE BlockArguments #-}

module StoreManager.RedisStore
    ( RedisKey
    , RedisValue
    , deleteValue
    , getValue
    , setValue
    ) where

import           Control.Monad          (void)
import           Control.Monad.IO.Class
import           Data.Either            (fromRight)
import           Data.Maybe             (fromMaybe)
import           Data.Text
import           Data.Text.Encoding     (decodeUtf8, encodeUtf8)
import           Database.Redis
import           Logger                 (loggerName)
import           System.Environment     (lookupEnv)
import           System.Log.Logger      (debugM)
import           Text.Printf            (printf)

type RedisKey = Text
type RedisValue = Text

secondsToMs :: Integer -> Integer
secondsToMs = (* 1000)

minutesToMs :: Integer -> Integer
minutesToMs = secondsToMs . (* 60)

hoursToMs :: Integer -> Integer
hoursToMs = minutesToMs . (* 60)

daysToMs :: Integer -> Integer
daysToMs = hoursToMs . (* 24)

redisConnInfo :: IO ConnectInfo
redisConnInfo = do
  hostEnv <- lookupEnv "REDIS_HOST"
  redisPassEnv <- lookupEnv "REDIS_PASS"

  let host = fromMaybe "127.0.0.1" hostEnv
      redisPass = encodeUtf8 . pack <$> redisPassEnv

  return $ defaultConnectInfo { connectHost = host, connectPort = PortNumber 6379, connectAuth = redisPass }

setValue :: RedisKey -> RedisValue -> IO ()
setValue key value = do
  connInfo <- redisConnInfo
  withConnect
    connInfo
    \conn -> runRedis conn do
        void $ set encodedKey (encodeUtf8 value)
        void $ liftIO $ debugM loggerName $ printf "Set key: %s" key

        void $ pexpire encodedKey (daysToMs 1)
        void $ liftIO $ debugM loggerName $ printf "Set expritation for key: %s" key
    where encodedKey = encodeUtf8 key

getValue :: RedisKey -> IO (Maybe RedisValue)
getValue key = do
  connInfo <- redisConnInfo
  withConnect
    connInfo
    \conn -> runRedis conn do
      value <- get $ encodeUtf8 key
      void $
        liftIO $
          debugM loggerName $
            printf "Fetched value with key %s: %s" key (maybe "Nothing" unpack $ normalizeRedisValue value)
      return $ normalizeRedisValue value
        where normalizeRedisValue v = (decodeUtf8 <$>) $ fromRight Nothing v

deleteValue :: RedisKey -> IO ()
deleteValue key = do
  connInfo <- redisConnInfo
  void $ withConnect
    connInfo
    \conn -> runRedis conn $ del [encodeUtf8 key]
