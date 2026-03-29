#!/usr/bin/env python3
"""
手机发送文件到电脑 - Thru 测试脚本
使用方法：
1. 修改 PC_IP 为电脑的 IP 地址
2. 修改 FILE_PATH 为要发送的文件路径
3. 运行：python send_to_pc.py
"""

import requests
import os
import sys

# ========== 配置区域 ==========
# 电脑 IP (二选一)
PC_IP = "100.92.89.83"  # Tailscale IP (异地传输)
# PC_IP = "172.18.140.188"  # 局域网 IP (同一 WiFi 下)

PORT = 53317

# 要发送的文件路径 (修改这里)
# Android 示例:
FILE_PATH = "/sdcard/Download/test.txt"
# iOS (需要短指令或 Pythonista):
# FILE_PATH = "/var/mobile/Media/DCIM/100APPLE/IMG_0001.JPG"
# ===============================


def send_file(file_path):
    if not os.path.exists(file_path):
        print(f"❌ 文件不存在：{file_path}")
        return False

    file_size = os.path.getsize(file_path)
    file_name = os.path.basename(file_path)

    url = f"http://{PC_IP}:{PORT}/upload"

    print(f"📤 正在发送：{file_name} ({file_size / 1024 / 1024:.1f}MB)")
    print(f"🎯 目标：{url}")
    print()

    try:
        with open(file_path, "rb") as f:
            files = {"file": (file_name, f)}
            response = requests.post(url, files=files, timeout=60)

        print()
        if response.status_code == 200:
            print(f"✅ 发送成功!")
            print(f"   状态码：{response.status_code}")
            print(f"   响应：{response.text}")
            return True
        else:
            print(f"❌ 发送失败!")
            print(f"   状态码：{response.status_code}")
            print(f"   响应：{response.text}")
            return False

    except requests.exceptions.ConnectionError as e:
        print(f"❌ 连接错误：无法连接到电脑")
        print(f"   请检查:")
        print(f"   1. 电脑是否运行 'thru serve'")
        print(f"   2. IP 地址是否正确 ({PC_IP})")
        print(f"   3. 防火墙是否允许 53317 端口")
        return False

    except requests.exceptions.Timeout as e:
        print(f"❌ 请求超时")
        return False

    except Exception as e:
        print(f"❌ 未知错误：{e}")
        return False


if __name__ == "__main__":
    # 如果命令行提供了文件路径，使用命令行参数
    if len(sys.argv) > 1:
        FILE_PATH = sys.argv[1]

    send_file(FILE_PATH)
