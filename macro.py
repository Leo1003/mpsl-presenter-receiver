from hid_keycode import Keycode
import serial
import time

def presskey(ser, keycode, modifier=None):
    if modifier != None:
        ser.write('kd {}\r'.format(modifier).encode('ascii'))
        ser.flush()
        time.sleep(0.2)
    ser.write('kd {}\r'.format(keycode).encode('ascii'))
    ser.flush()
    time.sleep(0.1)
    ser.write('ku {}\r'.format(keycode).encode('ascii'))
    ser.flush()
    time.sleep(0.1)
    if modifier != None:
        ser.write('ku {}\r'.format(modifier).encode('ascii'))
        ser.flush()
        time.sleep(0.1)

with serial.Serial('/dev/ttyACM0', 115200) as ser:
    presskey(ser, Keycode.SPACE, Keycode.ALT)
    presskey(ser, Keycode.K)
    presskey(ser, Keycode.W)
    presskey(ser, Keycode.R)
    presskey(ser, Keycode.I)
    presskey(ser, Keycode.T)
    presskey(ser, Keycode.E)
    presskey(ser, Keycode.ENTER)
    time.sleep(5)
    presskey(ser, Keycode.H, Keycode.SHIFT)
    presskey(ser, Keycode.E)
    presskey(ser, Keycode.L)
    presskey(ser, Keycode.L)
    presskey(ser, Keycode.O)
    presskey(ser, Keycode.COMMA)
    presskey(ser, Keycode.SPACE)
    presskey(ser, Keycode.W)
    presskey(ser, Keycode.O)
    presskey(ser, Keycode.R)
    presskey(ser, Keycode.L)
    presskey(ser, Keycode.D)
    presskey(ser, Keycode.PERIOD)

