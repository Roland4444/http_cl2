{-# LANGUAGE CPP #-}
{-# LANGUAGE NoRebindableSyntax #-}
#if __GLASGOW_HASKELL__ >= 810
{-# OPTIONS_GHC -Wno-prepositive-qualified-module #-}
#endif
{-# OPTIONS_GHC -fno-warn-missing-import-lists #-}
{-# OPTIONS_GHC -w #-}
module Paths_httpTest (
    version,
    getBinDir, getLibDir, getDynLibDir, getDataDir, getLibexecDir,
    getDataFileName, getSysconfDir
  ) where


import qualified Control.Exception as Exception
import qualified Data.List as List
import Data.Version (Version(..))
import System.Environment (getEnv)
import Prelude


#if defined(VERSION_base)

#if MIN_VERSION_base(4,0,0)
catchIO :: IO a -> (Exception.IOException -> IO a) -> IO a
#else
catchIO :: IO a -> (Exception.Exception -> IO a) -> IO a
#endif

#else
catchIO :: IO a -> (Exception.IOException -> IO a) -> IO a
#endif
catchIO = Exception.catch

version :: Version
version = Version [0,1,0,0] []

getDataFileName :: FilePath -> IO FilePath
getDataFileName name = do
  dir <- getDataDir
  return (dir `joinFileName` name)

getBinDir, getLibDir, getDynLibDir, getDataDir, getLibexecDir, getSysconfDir :: IO FilePath




bindir, libdir, dynlibdir, datadir, libexecdir, sysconfdir :: FilePath
bindir     = "C:\\Users\\User\\AppData\\Roaming\\cabal\\bin"
libdir     = "C:\\Users\\User\\AppData\\Roaming\\cabal\\x86_64-windows-ghc-9.12.3\\httpTest-0.1.0.0-inplace-httpTest"
dynlibdir  = "C:\\Users\\User\\AppData\\Roaming\\cabal\\x86_64-windows-ghc-9.12.3"
datadir    = "C:\\Users\\User\\AppData\\Roaming\\cabal\\x86_64-windows-ghc-9.12.3\\httpTest-0.1.0.0"
libexecdir = "C:\\Users\\User\\AppData\\Roaming\\cabal\\httpTest-0.1.0.0-inplace-httpTest\\x86_64-windows-ghc-9.12.3\\httpTest-0.1.0.0"
sysconfdir = "C:\\Users\\User\\AppData\\Roaming\\cabal\\etc"

getBinDir     = catchIO (getEnv "httpTest_bindir")     (\_ -> return bindir)
getLibDir     = catchIO (getEnv "httpTest_libdir")     (\_ -> return libdir)
getDynLibDir  = catchIO (getEnv "httpTest_dynlibdir")  (\_ -> return dynlibdir)
getDataDir    = catchIO (getEnv "httpTest_datadir")    (\_ -> return datadir)
getLibexecDir = catchIO (getEnv "httpTest_libexecdir") (\_ -> return libexecdir)
getSysconfDir = catchIO (getEnv "httpTest_sysconfdir") (\_ -> return sysconfdir)



joinFileName :: String -> String -> FilePath
joinFileName ""  fname = fname
joinFileName "." fname = fname
joinFileName dir ""    = dir
joinFileName dir fname
  | isPathSeparator (List.last dir) = dir ++ fname
  | otherwise                       = dir ++ pathSeparator : fname

pathSeparator :: Char
pathSeparator = '\\'

isPathSeparator :: Char -> Bool
isPathSeparator c = c == '/' || c == '\\'
