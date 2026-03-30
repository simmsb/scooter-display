glasgow-on:
    glasgow run probe-rs -V 'A=3.3,B=5.0' --swclk 'A3' --swdio 'A2'

glasgow-off:
    glasgow safe

glasgow-reset:
    just glasgow-off
    just glasgow-on

flash-stock:
    probe-rs download --chip AT32F415RCT7 --probe 20b7:9db1:C3-20251207T143255Z:1:0 --protocol swd --base-address "0x8000000" ~/Dropbox/egret-stuff/firmware/display_modified.bin --binary-format bin
