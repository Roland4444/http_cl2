{-# LANGUAGE DeriveGeneric #-}
{-# LANGUAGE OverloadedStrings #-}
{-# LANGUAGE RecordWildCards #-}

module Main where

import qualified Data.ByteString.Lazy as B
import Data.Binary (Binary, encode, decode)
import Data.Binary.Put (putWord32le, runPut)
import Data.Binary.Get (getWord32le, runGet, isEmpty)
import GHC.Generics (Generic)
import System.Directory (doesFileExist, removeFile)
import Control.Exception (catch, SomeException)
import System.IO (IOMode(ReadMode, WriteMode), withFile)
import Control.Monad (when, void)
import Data.List (find, findIndex)
import Data.Maybe (fromMaybe, isJust, catMaybes)


data Employee = Employee {
    empId          :: Int,
    empName        :: String,
    empLastName    :: String,
    empMiddleName  :: String,
    empWorkPosition :: String
} deriving (Show, Eq, Generic)


instance Binary Employee

newtype Pack = Pack { packEmployees :: [Employee] }
    deriving (Show, Eq, Generic)

instance Binary Pack

deserializeFromFile :: Binary a => FilePath -> IO (Either String a)
deserializeFromFile filename = do
    exists <- doesFileExist filename
    if not exists
        then return $ Left $ "File not found: " ++ filename
        else do
            content <- B.readFile filename
            return $ Right $ decode content

splitToFi :: String -> [String]
splitToFi = words  

employeeNew :: Int -> String -> String -> String -> String -> Employee
employeeNew = Employee

employeeSerializeToFile :: Employee -> FilePath -> IO (Either String ())
employeeSerializeToFile emp filename = do
    let encoded = encode emp
    catch (do
        B.writeFile filename encoded
        return $ Right ())
        (\e -> return $ Left $ "Write error: " ++ show (e :: SomeException))

employeeDeserializeFromFile :: FilePath -> IO (Either String Employee)
employeeDeserializeFromFile = deserializeFromFile

employeeToString :: Employee -> String
employeeToString Employee{..} = 
    unwords [show empId, empName, empMiddleName, empLastName, empWorkPosition]


packNew :: [Employee] -> Pack
packNew = Pack

packSerializeToFile :: Pack -> FilePath -> IO (Either String ())
packSerializeToFile pack filename = do
    let encoded = encode pack
    catch (do
        B.writeFile filename encoded
        return $ Right ())
        (\e -> return $ Left $ "Write error: " ++ show (e :: SomeException))

packDeserializeFromFile :: FilePath -> IO (Either String Pack)
packDeserializeFromFile = deserializeFromFile

pushAndUpdate :: Pack -> Employee -> Pack
pushAndUpdate (Pack employees) entry
    | isContains (Pack employees) entry = 
        let updated = removeEmployee (Pack employees) entry
        in Pack (packEmployees updated ++ [entry])
    | otherwise = Pack (employees ++ [entry])

isContains :: Pack -> Employee -> Bool
isContains (Pack employees) entry =
    any (\emp -> 
        empId emp == empId entry || 
        (empName emp == empName entry && empLastName emp == empLastName entry))
    employees

removeEmployee :: Pack -> Employee -> Pack
removeEmployee (Pack employees) entry =
    Pack $ filter (\emp -> 
        not (empId emp == empId entry || 
            (empName emp == empName entry && empLastName emp == empLastName entry)))
        employees

packToString :: Pack -> String -> String
packToString (Pack employees) ender =
    concatMap (\emp -> employeeToString emp ++ ender) employees

getIdByFi :: Pack -> String -> Maybe Int
getIdByFi (Pack employees) fi =
    case splitToFi fi of
        [lastName, firstName, _] -> 
            find (\emp -> empLastName emp == lastName && empName emp == firstName) employees
            >>= Just . empId
        [lastName, firstName] -> 
            find (\emp -> empLastName emp == lastName && empName emp == firstName) employees
            >>= Just . empId
        _ -> Nothing


testEmployees :: [Employee]
testEmployees = [
    employeeNew 1 "Иван" "Иванов" "Иванович" "Разработчик",
    employeeNew 2 "Петр" "Петров" "Петрович" "Тестировщик",
    employeeNew 3 "Анна" "Сидорова" "Александровна" "Менеджер"]

demo :: IO ()
demo = do
    putStrLn "=== Демонстрация работы на Haskell ==="
    putStrLn ""
    
    let pack = packNew testEmployees
    putStrLn "Создан пакет с сотрудниками:"
    putStrLn $ packToString pack "\n"
    
    putStrLn "Поиск ID по ФИ 'Иванов Иван':"
    case getIdByFi pack "Иванов Иван" of
        Just id -> putStrLn $ "Найден ID: " ++ show id
        Nothing -> putStrLn "Не найден"
    
    putStrLn ""
    
    let newEmp = employeeNew 4 "Мария" "Васильева" "Сергеевна" "Дизайнер"
    putStrLn "Добавляем нового сотрудника:"
    print newEmp
    
    let updatedPack = pushAndUpdate pack newEmp
    putStrLn "\nПакет после добавления:"
    putStrLn $ packToString updatedPack "\n"
    
    let duplicateEmp = employeeNew 1 "Иван" "Иванов" "Иванович" "Старший разработчик"
    putStrLn "Добавляем дубликат (должен заменить):"
    
    let finalPack = pushAndUpdate updatedPack duplicateEmp
    putStrLn "\nФинальный пакет:"
    putStrLn $ packToString finalPack "\n"
    
    putStrLn "Сериализация в файл 'employees.bin'..."
    result <- packSerializeToFile finalPack "employees.bin"
    
    case result of
        Left err -> putStrLn $ "Ошибка записи: " ++ err
        Right _ -> do
            putStrLn "Запись успешна"
            
            putStrLn "\nДесериализация из файла..."
            readResult <- packDeserializeFromFile "employees.bin"
            
            case readResult of
                Left err -> putStrLn $ "Ошибка чтения: " ++ err
                Right restoredPack -> do
                    putStrLn "Чтение успешно"
                    
                    if finalPack == restoredPack
                        then putStrLn "✓ Пакеты идентичны после сериализации/десериализации"
                        else putStrLn "✗ Пакеты различаются"
                    
                    putStrLn "\nВосстановленные данные:"
                    putStrLn $ packToString restoredPack "\n"

testPerformance :: IO ()
testPerformance = do
    putStrLn "\n=== Тест производительности ==="
    
    let bigList = [employeeNew i ("Name_" ++ show i) ("LastName_" ++ show i) 
                   ("Middle_" ++ show i) ("Position_" ++ show i) | i <- [1..10000]]
    
    let bigPack = packNew bigList
    putStrLn $ "Создан пакет с " ++ show (length bigList) ++ " сотрудниками"
    
    putStrLn "Сериализация..."
    _ <- packSerializeToFile bigPack "big_pack.bin"
    
    putStrLn "Десериализация..."
    result <- packDeserializeFromFile "big_pack.bin"
    
    case result of
        Right restored -> do
            putStrLn $ "Восстановлено сотрудников: " ++ show (length (packEmployees restored))
            putStrLn "✓ Большой пакет успешно сериализован и десериализован"
        Left err -> putStrLn $ "Ошибка: " ++ err
    
    return ()

safeDeserializeFromFile :: Binary a => FilePath -> IO (Either String a)
safeDeserializeFromFile filename = do
    exists <- doesFileExist filename
    if not exists
        then return $ Left $ "File does not exist: " ++ filename
        else do
            content <- B.readFile filename
            catch (do
                let result = decode content
                return $ Right result)
                (\e -> return $ Left $ "Decode error: " ++ show (e :: SomeException))

serializeWithHandle :: Binary a => a -> FilePath -> IO (Either String ())
serializeWithHandle value filename = do
    catch (do
        B.writeFile filename (encode value)
        return $ Right ())
        (\e -> return $ Left $ show (e :: SomeException))

deserializeWithHandle :: Binary a => FilePath -> IO (Either String a)
deserializeWithHandle filename = do
    exists <- doesFileExist filename
    if not exists
        then return $ Left "File not found"
        else do
            content <- B.readFile filename
            return $ Right $ decode content

main :: IO ()
main = do
    demo
    testPerformance
       
    putStrLn "\n=== Программа завершена ==="
