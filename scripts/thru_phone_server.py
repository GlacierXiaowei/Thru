#!/usr/bin/env python3
"""
Thru 手机端 HTTP 服务
功能：
1. UDP Discovery 响应 - 让电脑能发现手机
2. POST /upload 接收 - 让电脑能发送文件到手机

用法：
  python thru_phone_server.py              # 默认端口 53317
  python thru_phone_server.py --port 8080  # 指定端口

保存目录: ~/storage/downloads/Thru/
"""

import argparse
import json
import os
import socket
import struct
import threading
import uuid
from http.server import HTTPServer, BaseHTTPRequestHandler
from pathlib import Path

MULTICAST_ADDR = "239.12.34.56"
MULTICAST_PORT = 53317
DEFAULT_HTTP_PORT = 53317
SAVE_DIR = Path("/storage/emulated/0/Download/Thru")


class ThruRequestHandler(BaseHTTPRequestHandler):
    def log_message(self, format, *args):
        pass

    def do_GET(self):
        if self.path == "/":
            self.send_json_response(
                {"name": "Thru Phone Server", "version": "1.0", "status": "running"}
            )
        elif self.path == "/device":
            self.send_json_response(
                {
                    "type": "THRU_RESPONSE",
                    "name": socket.gethostname(),
                    "ip": get_local_ip(),
                    "port": self.server.server_port,
                    "device_id": self.server.device_id,
                    "network": "lan",
                }
            )
        else:
            self.send_error(404)

    def do_POST(self):
        if self.path == "/upload":
            self.handle_upload()
        else:
            self.send_error(404)

    def send_json_response(self, data):
        self.send_response(200)
        self.send_header("Content-Type", "application/json")
        self.send_header("Access-Control-Allow-Origin", "*")
        self.end_headers()
        self.wfile.write(json.dumps(data).encode())

    def handle_upload(self):
        content_type = self.headers.get("Content-Type", "")
        if "multipart/form-data" not in content_type:
            self.send_error(400, "Expected multipart/form-data")
            return

        boundary = None
        for part in content_type.split(";"):
            part = part.strip()
            if part.startswith("boundary="):
                boundary = part[9:]
                if boundary.startswith('"') and boundary.endswith('"'):
                    boundary = boundary[1:-1]
                break

        if not boundary:
            self.send_error(400, "No boundary found")
            return

        content_length = int(self.headers.get("Content-Length", 0))
        body = self.rfile.read(content_length)

        boundary_bytes = f"--{boundary}".encode()
        parts = body.split(boundary_bytes)

        saved_files = []
        for part in parts:
            if not part or part == b"--" or part == b"--\r\n":
                continue

            if b"Content-Disposition" not in part:
                continue

            headers_end = part.find(b"\r\n\r\n")
            if headers_end == -1:
                continue

            headers = part[:headers_end].decode("utf-8", errors="ignore")
            file_data = part[headers_end + 4 :]

            if file_data.endswith(b"\r\n"):
                file_data = file_data[:-2]

            filename = None
            for line in headers.split("\r\n"):
                if 'filename="' in line:
                    start = line.find('filename="') + 10
                    end = line.find('"', start)
                    filename = line[start:end]
                    break

            if filename and file_data:
                SAVE_DIR.mkdir(parents=True, exist_ok=True)
                file_path = SAVE_DIR / filename
                file_path.write_bytes(file_data)
                saved_files.append(filename)
                print(f"📥 收到文件: {filename} ({len(file_data)} bytes)")
                print(f"✓ 已保存到: {file_path}")

        if saved_files:
            self.send_json_response({"success": True, "files": saved_files})
        else:
            self.send_error(400, "No files in upload")


def get_local_ip():
    try:
        hostname = socket.gethostname()
        for info in socket.getaddrinfo(hostname, None):
            ip = info[4][0]
            if ip.startswith("127.") or ip.startswith("172."):
                continue
            if ":" not in ip:
                return ip
    except:
        pass

    try:
        s = socket.socket(socket.AF_INET, socket.SOCK_DGRAM)
        s.connect(("8.8.8.8", 80))
        ip = s.getsockname()[0]
        s.close()
        if not ip.startswith("172."):
            return ip
    except:
        pass

    return "0.0.0.0"


def discovery_listener(http_port, device_id):
    sock = socket.socket(socket.AF_INET, socket.SOCK_DGRAM)
    sock.setsockopt(socket.SOL_SOCKET, socket.SO_REUSEADDR, 1)

    mreq = struct.pack("4sl", socket.inet_aton(MULTICAST_ADDR), socket.INADDR_ANY)
    sock.setsockopt(socket.IPPROTO_IP, socket.IP_ADD_MEMBERSHIP, mreq)
    sock.bind(("0.0.0.0", MULTICAST_PORT))

    hostname = socket.gethostname()
    local_ip = get_local_ip()

    print(f"🔍 Discovery 监听已启动 ({MULTICAST_ADDR}:{MULTICAST_PORT})")

    while True:
        try:
            data, addr = sock.recvfrom(4096)
            msg = json.loads(data.decode())

            if msg.get("type") == "THRU_DISCOVER":
                response = {
                    "type": "THRU_RESPONSE",
                    "name": hostname,
                    "ip": local_ip,
                    "port": http_port,
                    "device_id": device_id,
                    "network": "lan",
                }
                sock.sendto(json.dumps(response).encode(), addr)
        except Exception as e:
            pass


def run_server(port):
    SAVE_DIR.mkdir(parents=True, exist_ok=True)

    device_id = str(uuid.uuid4())

    discovery_thread = threading.Thread(
        target=discovery_listener, args=(port, device_id), daemon=True
    )
    discovery_thread.start()

    server = HTTPServer(("0.0.0.0", port), ThruRequestHandler)
    server.device_id = device_id
    server.server_port = port

    local_ip = get_local_ip()

    print("╔════════════════════════════════════════╗")
    print("║       Thru 手机端 HTTP 服务            ║")
    print("╠════════════════════════════════════════╣")
    print(f"║  HTTP:  http://{local_ip}:{port:<19}║")
    print(f"║  保存:  {str(SAVE_DIR):<26}║")
    print("║  按 Ctrl+C 停止                        ║")
    print("╚════════════════════════════════════════╝")
    print()

    try:
        server.serve_forever()
    except KeyboardInterrupt:
        print("\n服务已停止")


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description="Thru 手机端 HTTP 服务")
    parser.add_argument(
        "--port", "-p", type=int, default=DEFAULT_HTTP_PORT, help="HTTP 端口"
    )
    args = parser.parse_args()

    run_server(args.port)
