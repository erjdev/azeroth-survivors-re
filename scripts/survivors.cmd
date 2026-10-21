@echo off
cd /d "%~dp0.."
if defined WOW_DATA goto run
if exist "wow-data\Data\terrain.MPQ" set "WOW_DATA=%CD%\wow-data\Data"
:run
cargo survivors
