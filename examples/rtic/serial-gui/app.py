import tkinter as tk
from tkinter.constants import CENTER, E, N, NE, NW, RIGHT, SE, SW, W
from uart import UART
from defines import *


class App:
    """tKinter application.
    """

    def __init__(self, parent: tk.Tk) -> None:
        """App class constructor.

        Args:
            parent (tk.Tk): Main window
        """

        self.parent = parent
        self.uart = None
        self.led_freq = 1

        self.create_menu()
        self.create_led_box()
        self.create_rgb_box()
        self.create_lcd_box()

    def send(self, app: int, cmd: int, payload=0) -> None:
        """Sends command through UART instance.

        Args:
            app (int): Application code
            cmd (int): Command code
            payload ([type]): Data packet
        """

        packet = bytearray()
        packet.append(app)
        packet.append(cmd)

        if type(payload) == list:
            packet.append(len(payload))
            packet.extend(payload)
        elif type(payload) == str:
            packet.append(len(payload))

            for l in payload:
                packet.append(int(l.encode('utf-8').hex(), 16))
        elif type(payload) == int:
            packet.append(0x01)
            packet.append(payload)

        try:
            self.uart.write(packet)
        except AttributeError:
            print("No device connected")

    def create_menu(self):
        """Creates menu bar.
        """

        self.menu_bar = tk.Menu(self.parent)
        self.connect_menu = tk.Menu(self.menu_bar, tearoff=0)
        ports = UART.list_serial_ports()

        for p in ports:
            self.connect_menu.add_command(
                label=p.device, command=lambda: self.connect_to_port(p.device))

        self.menu_bar.add_cascade(label="Connect", menu=self.connect_menu)
        self.parent.config(menu=self.menu_bar)

    def change_port_menu_item(self, port_name: str) -> None:
        index = self.connect_menu.index(port_name)
        self.connect_menu.entryconfig(index, state="disabled")

    def connect_to_port(self, port: str) -> None:
        """Connects to a specific serial port.

        Args:
            port (str): Port name
        """

        self.uart = UART(port)
        self.change_port_menu_item(port)
        print("Connected to {}".format(port))

    def turn_led_on(self) -> None:
        """Button callback function to turn LED off.
        """

        self.send(APP.LED, LED_CMD.LED_ON)

    def turn_led_off(self) -> None:
        """Button callback function to turn LED off.
        """

        self.send(APP.LED, LED_CMD.LED_OFF)

    def set_led_freq(self) -> None:
        """Button callback function to set LED frequency.
        """

        freq = int(self.freq_spinbox.get())
        self.send(APP.LED, LED_CMD.SET_FREQ, freq)

    def create_led_box(self):
        """Creates LED section.
        """

        # LED frame
        self.led_frame = tk.Frame(
            self.parent, highlightbackground='black', highlightthickness=2)
        self.led_frame.place(relwidth=0.95, relheight=0.223,
                             relx=0.5, rely=0.12, anchor=CENTER)

        # LED labels
        self.led_title_label = tk.Label(
            self.led_frame, text="LED", font="Verdana 12 bold")
        self.led_title_label.place(relx=0.5, rely=0.15, anchor=CENTER)

        # LED Buttons
        self.turn_on_button = tk.Button(
            self.led_frame, text="ON", command=self.turn_led_on)
        self.turn_on_button.place(
            relwidth=0.25, relheight=0.3, relx=0.1, rely=0.45, anchor=W)

        self.turn_off_button = tk.Button(
            self.led_frame, text="OFF", command=self.turn_led_off)
        self.turn_off_button.place(
            relwidth=0.25, relheight=0.3, relx=0.1, rely=0.8, anchor=W)

        self.freq_spinbox = tk.Spinbox(
            self.led_frame, justify=CENTER, from_=1, to=99, state="readonly")
        self.freq_spinbox.place(
            relwidth=0.25, relheight=0.3, relx=0.6, rely=0.45, anchor=W)

        self.freq_spinbox_label = tk.Label(
            self.freq_spinbox, text="Hz", font="Verdana 10")
        self.freq_spinbox_label.place(
            relheight=0.55, relx=0.52, rely=0.45, anchor=W)

        self.set_freq_button = tk.Button(
            self.led_frame, text="Set Freq", command=self.set_led_freq)
        self.set_freq_button.place(
            relwidth=0.25, relheight=0.3, relx=0.6, rely=0.8, anchor=W)

    def set_rgb_blue_val(self):
        """Button callback function to set RGB blue value.
        """

        val = int(self.blue_spinbox.get())
        self.send(APP.RGB, RGB_CMD.SET_BLUE, val)

    def set_rgb_green_val(self):
        """Button callback function to set RGB green value.
        """

        val = int(self.green_spinbox.get())
        self.send(APP.RGB, RGB_CMD.SET_GREEN, val)

    def set_rgb_red_val(self):
        """Button callback function to set RGB red value.
        """

        val = int(self.red_spinbox.get())
        self.send(APP.RGB, RGB_CMD.SET_RED, val)

    def set_rgb_all_colors(self):
        """Button callback function to set all RGB colors
        """

        red_val = int(self.red_spinbox.get())
        green_val = int(self.green_spinbox.get())
        blue_val = int(self.blue_spinbox.get())

        self.send(APP.RGB, RGB_CMD.SET_COLORS, [red_val, green_val, blue_val])

    def create_rgb_box(self):
        """Creates RGB section.
        """

        # RGB frame
        self.rgb_frame = tk.Frame(
            self.parent, highlightbackground='black', highlightthickness=2)
        self.rgb_frame.place(relwidth=0.95, relheight=0.3,
                             relx=0.5, rely=0.4, anchor=CENTER)

        # RGB labels
        self.rgb_title_label = tk.Label(
            self.rgb_frame, text="RGB", font="Verdana 12 bold")
        self.rgb_title_label.place(relx=0.5, rely=0.1, anchor=CENTER)

        # RGB Buttons
        self.set_red_value = tk.Button(
            self.rgb_frame, text="Set Red", command=self.set_rgb_red_val)
        self.set_red_value.place(
            relwidth=0.25, relheight=0.2, relx=0.10, rely=0.25, anchor=NW)

        self.set_green_value = tk.Button(
            self.rgb_frame, text="Set Green", command=self.set_rgb_green_val)
        self.set_green_value.place(
            relwidth=0.25, relheight=0.2, relx=0.10, rely=0.6, anchor=W)

        self.set_blue_value = tk.Button(
            self.rgb_frame, text="Set Blue", command=self.set_rgb_blue_val)
        self.set_blue_value.place(
            relwidth=0.25, relheight=0.2, relx=0.10, rely=0.95, anchor=SW)

        self.set_blue_value = tk.Button(
            self.rgb_frame, text="Set All Colors", command=self.set_rgb_all_colors)
        self.set_blue_value.place(
            relwidth=0.3, relheight=0.2, relx=0.95, rely=0.6, anchor=E)

        self.red_spinbox = tk.Spinbox(
            self.rgb_frame, justify=CENTER, from_=0, to=254)
        self.red_spinbox.place(
            relwidth=0.2, relheight=0.2, relx=0.5, rely=0.25, anchor=N)

        self.green_spinbox = tk.Spinbox(
            self.rgb_frame, justify=CENTER, from_=0, to=254)
        self.green_spinbox.place(
            relwidth=0.2, relheight=0.2, relx=0.605, rely=0.6, anchor=E)

        self.blue_spinbox = tk.Spinbox(
            self.rgb_frame, justify=CENTER, from_=0, to=254)
        self.blue_spinbox.place(
            relwidth=0.2, relheight=0.2, relx=0.605, rely=0.95, anchor=SE)

    def send_lcd_cmd(self) -> None:
        """Button callback function to send command to LCD.
        """

        try:
            cmd = int(self.lcd_cmd_entry.get(), 16)
        except ValueError:
            print("Invalid value. Format should be 0x00")
        else:
            self.send(APP.LCD, LCD_CMD.SEND_CMD, cmd)

    def send_lcd_data(self) -> None:
        """Button callback function to send data to LCD.
        """

        data = self.lcd_data_entry.get()

        if len(data) > 32:
            data = data[:32]

        if len(data) == 0:
            data = ' '

        self.send(APP.LCD, LCD_CMD.SEND_DATA, data)

    def create_lcd_box(self):
        """Creates LCD section.
        """

        # LCD frame
        self.lcd_frame = tk.Frame(
            self.parent, highlightbackground='black', highlightthickness=2)
        self.lcd_frame.place(relwidth=0.95, relheight=0.4,
                             relx=0.5, rely=0.77, anchor=CENTER)

        # LCD labels
        self.lcd_title_label = tk.Label(
            self.lcd_frame, text="LCD", font="Verdana 12 bold")
        self.lcd_title_label.place(relx=0.5, rely=0.1, anchor=CENTER)

        self.lcd_cmd_label = tk.Label(
            self.lcd_frame, text="Command", font="Verdana 10 bold")
        self.lcd_cmd_label.place(relx=0.05, rely=0.25, anchor=W)

        self.lcd_data_label = tk.Label(
            self.lcd_frame, text="Data", font="Verdana 10 bold")
        self.lcd_data_label.place(relx=0.6, rely=0.25, anchor=W)

        # LCD entries
        self.lcd_cmd_entry = tk.Entry(self.lcd_frame, justify=CENTER)
        self.lcd_cmd_entry.place(
            relwidth=0.24, relheight=0.15, relx=0.05, rely=0.4, anchor=W)

        self.lcd_data_entry = tk.Entry(self.lcd_frame, justify=RIGHT)
        self.lcd_data_entry.place(
            relwidth=0.6, relheight=0.15, relx=0.95, rely=0.4, anchor=E)

        # LCD Buttons
        self.lcd_send_cmd_button = tk.Button(
            self.lcd_frame, text="Send Cmd", command=self.send_lcd_cmd)
        self.lcd_send_cmd_button.place(
            relwidth=0.24, relheight=0.2, relx=0.05, rely=0.6, anchor=W)

        self.lcd_send_data_button = tk.Button(
            self.lcd_frame, text="Send Data", command=self.send_lcd_data)
        self.lcd_send_data_button.place(
            relwidth=0.24, relheight=0.2, relx=0.53, rely=0.6, anchor=W)


if __name__ == "__main__":
    root = tk.Tk()

    root.title("Rust Serial")
    root.geometry("350x400")
    root.resizable(False, False)

    app = App(root)
    root.mainloop()
