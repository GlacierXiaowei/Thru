#!/usr/bin/env python3
"""
Thru Phone Receiver - Simple HTTP server that accepts file uploads
Usage: python3 thru_receiver.py [port]
Default port: 53317
"""

import http.server
import socketserver
import os
import sys
from urllib.parse import parse_qs
from datetime import datetime

PORT = int(sys.argv[1]) if len(sys.argv) > 1 else 53317
UPLOAD_DIR = os.path.expanduser("~/storage/downloads/Thru")

os.makedirs(UPLOAD_DIR, exist_ok=True)

class UploadHandler(http.server.SimpleHTTPRequestHandler):
    def do_POST(self):
        if self.path == '/upload':
            content_length = int(self.headers.get('Content-Length', 0))
            content_type = self.headers.get('Content-Type', '')
            
            boundary = None
            for part in content_type.split(';'):
                if 'boundary=' in part:
                    boundary = part.split('boundary=')[1].strip()
                    break
            
            if not boundary:
                self.send_error(400, "No boundary found")
                return
            
            body = self.rfile.read(content_length)
            
            boundary_bytes = boundary.encode()
            parts = body.split(b'--' + boundary_bytes)
            
            for part in parts:
                if b'Content-Disposition' in part and b'filename=' in part:
                    filename_start = part.find(b'filename="') + len(b'filename="')
                    filename_end = part.find(b'"', filename_start)
                    filename = part[filename_start:filename_end].decode()
                    
                    header_end = part.find(b'\r\n\r\n')
                    if header_end == -1:
                        header_end = part.find(b'\n\n')
                    file_data = part[header_end + 4:].rstrip(b'\r\n-')
                    
                    filepath = os.path.join(UPLOAD_DIR, filename)
                    with open(filepath, 'wb') as f:
                        f.write(file_data)
                    
                    print(f"[{datetime.now().strftime('%H:%M:%S')}] 📥 Received: {filename} ({len(file_data)} bytes)")
            
            self.send_response(200)
            self.send_header('Content-Type', 'application/json')
            self.end_headers()
            self.wfile.write(b'{"success": true}')
        else:
            self.send_error(404)
    
    def do_GET(self):
        if self.path == '/':
            self.send_response(200)
            self.send_header('Content-Type', 'application/json')
            self.end_headers()
            import json
            response = {
                "name": "Thru-Phone",
                "version": "1.0",
                "status": "running"
            }
            self.wfile.write(json.dumps(response).encode())
        elif self.path == '/device':
            self.send_response(200)
            self.send_header('Content-Type', 'application/json')
            self.end_headers()
            import json
            import uuid
            response = {
                "name": os.uname().nodename if hasattr(os, 'uname') else "Phone",
                "device_id": str(uuid.uuid4()),
                "port": PORT
            }
            self.wfile.write(json.dumps(response).encode())
        else:
            super().do_GET()

with socketserver.TCPServer(("", PORT), UploadHandler) as httpd:
    print(f"🌐 Thru Receiver started on port {PORT}")
    print(f"📁 Files saved to: {UPLOAD_DIR}")
    print("Press Ctrl+C to stop\n")
    try:
        httpd.serve_forever()
    except KeyboardInterrupt:
        print("\n👋 Stopped")