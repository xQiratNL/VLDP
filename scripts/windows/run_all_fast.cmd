@echo off

echo ---Starting benches (fast)---
call .\scripts\windows\run_benches_fast.cmd
echo ---Finished benches (fast)---

echo ---Starting geo data---
call .\scripts\windows\run_geo_data_examples.cmd
echo ---Finished geo data---

echo ---Starting smart meter---
call .\scripts\windows\run_smart_meter_examples.cmd
echo --Finished smart meter---

echo ---Starting additional benches (fast)---
call .\scripts\windows\run_additional_benches_fast.cmd
echo ---Finished additional benches (fast)---
