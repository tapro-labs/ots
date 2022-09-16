{-# LANGUAGE DeriveAnyClass #-}
{-# LANGUAGE DeriveGeneric  #-}

module SecretManager.Store
    ( SecretData (..)
    , SecretId (..)
    , deleteSecret
    , getSecret
    , setSecret
    ) where

import           Data.Aeson              (FromJSON, ToJSON)
import           Data.Text
import           Data.UUID               (UUID, toText)
import           GHC.Generics
import           StoreManager.RedisStore (deleteValue, getValue, setValue)
import           System.Random           (randomIO)

data SecretId = SecretId { secretId :: !Text
                         }
  deriving (FromJSON, Generic, Show, ToJSON)

data SecretData = SecretData { secret :: !Text
                             }
  deriving (FromJSON, Generic, Show, ToJSON)

newUUID :: IO UUID
newUUID = randomIO

setSecret :: SecretData -> IO SecretId
setSecret (SecretData secretValue) = do
  key <- toText <$> newUUID
  setValue key secretValue >> return SecretId { secretId = key }

getSecret :: SecretId -> IO (Maybe SecretData)
getSecret (SecretId sId) = do
  maybeSecret <- getValue sId

  return $ maybeSecret >>= \x -> return SecretData { secret = x }

deleteSecret :: SecretId -> IO ()
deleteSecret (SecretId sId) = deleteValue sId
