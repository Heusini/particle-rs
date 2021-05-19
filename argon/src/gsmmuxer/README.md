Tried to implement [gsm0710muxer](https://github.com/particle-iot/gsm0710muxer/tree/5335eede736acc48a8964566ef52501979ea8439)
but failed. Particle uses a custom firmware for their esp32 which does not support the default at-commands. They use a muxer to send data to the esp32 which is also encoded. 
This can be a starting point.