#Setup

* Install openocd with xpm:
```
xpm install --global @xpack-dev-tools/openocd@latest
```

* install multigdb

* setup rustup to nightly
```
rustup override set nightly
```
* add arm 
```
rustup target add thumbv7em-none-eabihf
```

# Use
* build
```
cargo build --example wifi_example2
```

* start openocd
```
openocd -f interface/cmsis-dap.cfg -f target/nrf52.cfg
```

* start gdb allow .gdbinit or do commands manually
```
arm-none-eabi-gdb -q target/thumbv7em-none-eabihf/debug/examples/wifi_example2
```

### gdbinit:
```
target remote :3333
load
```