@echo off
for /f "delims=" %%i in ('powershell get-date -format "{yyyy-MM-dd--HH-mm-ss}"') do set datetime=%%i
if not exist ".\results\raw\smart_meter" mkdir ".\results\raw\smart_meter"

echo Running base (1/3)
cargo run --release --example smart_meter_base > .\results\raw\smart_meter\%datetime%_smart_meter_base.txt 2>&1

echo Running expand (2/3)
cargo run --release --example smart_meter_expand > .\results\raw\smart_meter\%datetime%_smart_meter_expand.txt 2>&1

echo Running shuffle (3/3)
cargo run --release --example smart_meter_shuffle > .\results\raw\smart_meter\%datetime%_smart_meter_shuffle.txt 2>&1

echo Parsing results
python .\scripts\parse_smart_meter_results.py %datetime%
