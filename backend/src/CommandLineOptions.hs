module CommandLineOptions
    ( MainOptions (..)
    , mainOptionsParser
    , mainOptionsParserExec
    ) where

import           Options.Applicative
import           System.Log.Logger   (Priority (..))
import           Text.Printf         (printf)
import           Text.Read           (readMaybe)

data MainOptions = MainOptions { logLevel :: Priority
                               , port     :: Int
                               , host     :: String
                               }

type DefaultMessage = String

maybeToEither :: DefaultMessage -> Maybe a -> Either DefaultMessage a
maybeToEither dMsg mValue = case mValue of
                             Just v  -> Right v
                             Nothing -> Left dMsg

parsePriority :: ReadM Priority
parsePriority = eitherReader $ \s -> case s of
                                      "ALL" -> return DEBUG -- DEBUG is the highest level
                                      _     -> maybeToEither (printf "Option \"%s\" unrecognized" s) (readMaybe s :: Maybe Priority)

mainOptionsParser :: Parser MainOptions
mainOptionsParser = MainOptions
  <$>
      option parsePriority
        ( long "log-level" <>
          short 'l' <>
          metavar "LOG_LEVEL" <>
          value EMERGENCY <>
          showDefault <>
          help "Set the logging level of the program. Options are: ALL, DEBUG, INFO, NOTICE, WARNING, ERROR, CRITICAL, ALERT and EMERGENCY"
        )
      <*>
      option auto
        ( long "port" <>
          short 'p' <>
          help "Port to run server" <>
          value 3000 <>
          showDefault <>
          metavar "INT"
        )
      <*>
      strOption
        ( long "host"<>
          short 'h' <>
          help "Host to run server" <>
          value "0.0.0.0" <>
          showDefault <>
          metavar "HOST"
        )

mainOptionsParserExec :: (MainOptions -> IO ()) -> IO ()
mainOptionsParserExec = (execParser opts >>=)
  where opts = info (mainOptionsParser <**> helper)
              ( fullDesc <>
                header "Backend server for OTS"
              )
