module Main where

{-# LANGUAGE OverloadedStrings #-}
import qualified Data.ByteString.Lazy.Char8 as L8
import           Network.HTTP.Simple

main :: IO ()
main = do
  responce <- httpLBS "http://localhost:11111/custom"
  putStr:n $ "The status code was: " ++
             show (getResponceStatusCode responce)

  print $ getResponceHeader "Content-Type" responce
  LB.putStrLn $ getResponceBody responce          
  putStrLn "Hello, Haskell!"
