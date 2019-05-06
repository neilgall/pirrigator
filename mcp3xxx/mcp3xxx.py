import busio
import digitalio
import board
import time
from adafruit_mcp3xxx.mcp3008 import MCP3008
from adafruit_mcp3xxx.analog_in import AnalogIn

spi = busio.SPI(clock=board.SCK, MISO=board.MISO, MOSI=board.MOSI)
cs = digitalio.DigitalInOut(board.D22)
mcp = MCP3008(spi, cs)
chan = AnalogIn(mcp, 0)

while True:
	print(chan.value)
	time.sleep(0.5)