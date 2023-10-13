# Minimal Raspberry Pi Pico Example

Roughly based on [rp-rs/rp2040-project-template](https://github.com/rp-rs/rp2040-project-template/) but with a testing example.

### Picoprobe connected to USB

```shell
❯ sudo lsusb | rg -i Pi
Bus 003 Device 007: ID 2e8a:000c Raspberry Pi Picoprobe (CMSIS-DAP)
```

#### Picoprobe connected to target Pico:

```

                            (1) (40)
                            (2) (39) VSYS  | +V bus->
                            (3) (38) GND   | -V bus->
<-debug swclk |   SPIO SCK  (4) (37)
<-debug swdio |   SPIO TX   (5) (36)
<-GPIO1     (2) | SPIO RX   (6) (35)
<-GPIO0     (1) | SPIO CSn  (7) (34)
                            (8) (33)
                            (9) (32)
                           (10) (31)
                           (11) (30)
                           (12) (29)
                           (13) (28)
                           (14) (27)
                           (15) (26)
                           (16) (25)
                           (17) (24)
                           (18) (23)
                           (19) (22)
                           (20) (21)
```

#### Target Pico

Or Pico W (pico w on-board LED is controlled by a different chip, so this doesn't work).

```

<-GPIO5 (7) |     SPIO RX   (1) (40)
<-GPIO4 (6) |     SPIO CSn  (2) (39) VSYS  | +V bus->
                            (3) (38) GND   | -V bus->
                            (4) (37)
                            (5) (36)
                            (6) (35)
                            (7) (34)
                            (8) (33)
                            (9) (32)
                           (10) (31)
                           (11) (30)
                           (12) (29)
                           (13) (28)
                           (14) (27)
                           (15) (26)
                           (16) (25)
                           (17) (24)
                           (18) (23)
                           (19) (22)
                           (20) (21)
DEBUG SWCLK -> GPIO2 (4)
DEBUG GND   -> -V bus
DEBUG SWDIO -> GPIO3 (5)
```

### test

```shell
❯ cargo test
   Compiling rprsblink v0.1.0 (/home/bob/src/rprsblink)
    Finished test [optimized + debuginfo] target(s) in 0.13s
     Running tests/main.rs (target/thumbv6m-none-eabi/debug/deps/main-1150da813bef6a85)
(HOST) INFO  flashing program (2 pages / 8.00 KiB)
(HOST) INFO  success!
────────────────────────────────────────────────────────────────────────────────
(1/1) running `it_works`...
all tests passed!
────────────────────────────────────────────────────────────────────────────────
(HOST) INFO  program has used at least 0.17/254.93 KiB (0.1%) of stack space
(HOST) INFO  device halted without error
```

### run

```shell
❯ cargo run
   Compiling rprsblink v0.1.0 (/home/bob/src/rprsblink)
    Finished dev [optimized + debuginfo] target(s) in 0.13s
     Running `probe-run --chip RP2040 target/thumbv6m-none-eabi/debug/rprsblink`
(HOST) INFO  flashing program (3 pages / 12.00 KiB)
(HOST) INFO  success!
────────────────────────────────────────────────────────────────────────────────
INFO  Program start
└─ rprsblink::__cortex_m_rt_main @ src/main.rs:23
INFO  on!
└─ rprsblink::__cortex_m_rt_main @ src/main.rs:60
INFO  off!
└─ rprsblink::__cortex_m_rt_main @ src/main.rs:63
INFO  on!
└─ rprsblink::__cortex_m_rt_main @ src/main.rs:60
```
