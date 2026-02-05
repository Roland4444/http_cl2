{-# LANGUAGE OverloadedStrings #-}
module HttpTest where

import qualified Data.ByteString.Lazy.Char8 as L8

-- Пример функции для тестирования
add :: Int -> Int -> Int
add x y = x + y

-- Основная логика приложения
runApp :: IO ()
runApp = do
    putStrLn "Hello, Haskell!"
    print $ add 2 3