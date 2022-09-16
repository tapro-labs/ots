{-# LANGUAGE DataKinds         #-}
{-# LANGUAGE TypeOperators     #-}

module SecretManager.API
    ( SecretManagerAPI
    , server
    ) where

import           Control.Monad           (void)
import           Control.Monad.IO.Class
import           Data.Text
import qualified SecretManager.Store     as Store (SecretData (..), SecretId (..), deleteSecret,
                                                   getSecret, setSecret)
import           Servant                 (Capture, Get, JSON, Post, ReqBody, (:>))
import qualified Servant
import           Servant.API.Alternative

type SecretId = Text

type SecretManagerAPI = "secret" :>
  (
    Capture "secretId" SecretId :> Get '[JSON] Store.SecretData :<|>
    ReqBody '[JSON] Store.SecretData :> Post '[JSON] Store.SecretId
  )

getSecret :: SecretId -> Servant.Handler Store.SecretData
getSecret sId = do
  value <- liftIO $ Store.getSecret secretId
  case value of
    Just v -> do
      void $ liftIO $ Store.deleteSecret secretId
      return v
    Nothing -> Servant.throwError Servant.err404

  where
    secretId = Store.SecretId { Store.secretId = sId }

createSecret :: Store.SecretData -> Servant.Handler Store.SecretId
createSecret = liftIO . Store.setSecret

server :: Servant.Server SecretManagerAPI
server =  getSecret :<|> createSecret
