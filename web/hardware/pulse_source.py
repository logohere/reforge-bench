from machine import Pin, PWM
from time import sleep

Pin(2, Pin.IN)
pulse = None
try:
    if input("Verify powered 3.3 V receivers and common GND. Type RUN: ").strip() != "RUN":
        raise SystemExit("Not armed")
    pulse = PWM(Pin(2, Pin.OUT, value=0), freq=1000, duty_u16=0)
    pulse.freq(1000)
    pulse.duty_u16(32768)
    print("GP2: nominal 1000 Hz, 50% duty. Ctrl-C stops the pulse.")
    while True:
        sleep(1)
finally:
    try:
        if pulse is not None:
            pulse.duty_u16(0)
            pulse.deinit()
    finally:
        Pin(2, Pin.IN)
