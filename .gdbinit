target remote :3333
load
#break uart::__cortex_m_rt_main
# break mod.rs:196
# break wifi.rs:223
#break atat-test.rs:223
continue
