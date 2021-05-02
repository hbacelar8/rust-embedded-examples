import serial
from serial.serialutil import SerialException
from serial.tools.list_ports_linux import comports


class UART:
    """UART serial communication handling class.
    """

    @staticmethod
    def list_serial_ports() -> list:
        """Lists connected serial ports on UNIX systems.

        Returns:
            list: List of connected ports
        """

        return comports()

    def __init__(self, port: str, baudrate: int = 9600) -> None:
        """UART class constructor. Opens a serial port.

        Args:
            port (str): Port name
            baudrate (int, optional): Port baudrate. Defaults to 9600.
        """

        self.uart = None

        try:
            self.uart = serial.Serial(port=port, baudrate=baudrate)
        except ValueError:
            print("Parameter value out of range")
        except SerialException:
            print("Could not connect to device {}".format(port))

    def write(self, packet: bytearray) -> None:
        """Writes a packet of bytes to serial port if connected.

        Args:
            packet (bytearray): Packet of bytes
        """

        print("{} - {} bytes => {}".format(self.uart.port,
              len(packet), packet.hex().upper()))
        self.uart.write(packet)

    def close_port(self) -> None:
        """Close connected port.
        """

        self.uart.close()
        self.uart = None


if __name__ == "__main__":
    ports = UART.list_serial_ports()
    uart = UART(ports[0].device)
