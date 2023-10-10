import serial
import sys


def send_file_over_uart(port, filename):
    with serial.Serial(port, 115200, timeout=1) as ser:
        # 等待硬件准备好
        input("请确保RP2040已经在等待数据，然后按回车继续...")

        # 读取文件内容
        with open(filename, 'rb') as file:
            content = file.read()

        # 发送文件大小
        size_bytes = len(content).to_bytes(4, byteorder='little')
        ser.write(size_bytes)

        # 发送文件内容
        ser.write(content)

        print(f"文件 {filename} 已成功发送！")


if __name__ == "__main__":
    if len(sys.argv) < 3:
        print(f"用法: {sys.argv[0]} <串口路径> <文件路径>")
        sys.exit(1)

    port = sys.argv[1]
    filename = sys.argv[2]

    send_file_over_uart(port, filename)
