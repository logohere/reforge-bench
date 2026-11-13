from machine import I2C, Pin, PWM
from time import sleep

Pin(2, Pin.IN)
pulse = None
try:
    if input("Verify sensor supply, powered 3.3 V receivers and GND. Type RUN: ").strip() != "RUN":
        raise SystemExit("Not armed")
    bus = I2C(0, sda=Pin(4), scl=Pin(5), freq=100000)
    if 0x36 not in bus.scan():
        raise RuntimeError("AS5600 not responding at 0x36")
    pulse = PWM(Pin(2, Pin.OUT, value=0), freq=1000, duty_u16=0)
    while True:
        status = bus.readfrom_mem(0x36, 0x0B, 1)[0]
        if status & 0x38 != 0x20:
            pulse.duty_u16(0)
            print("Magnet missing, weak or strong; pulse stopped", status)
        else:
            data = bus.readfrom_mem(0x36, 0x0C, 2)
            angle = ((data[0] & 0x0F) << 8) | data[1]
            frequency = 500 + angle * 1500 // 4095
            pulse.freq(frequency)
            pulse.duty_u16(32768)
            print(angle, angle * 360 / 4096, frequency)
        sleep(0.1)
finally:
    try:
        if pulse is not None:
            pulse.duty_u16(0)
            pulse.deinit()
    finally:
        Pin(2, Pin.IN)
