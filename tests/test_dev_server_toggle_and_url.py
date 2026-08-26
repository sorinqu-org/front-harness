import os
import pty
import struct
import fcntl
import termios
import sys
import time
import re
import urllib.request

def set_winsize(fd, rows, cols):
    winsize = struct.pack("HHHH", rows, cols, 0, 0)
    fcntl.ioctl(fd, termios.TIOCSWINSZ, winsize)

def run_test():
    master, slave = pty.openpty()
    set_winsize(master, 45, 130)
    os.set_blocking(master, False)
    
    env = os.environ.copy()
    env["TERM"] = "xterm-256color"
    
    pid = os.fork()
    if pid == 0:
        os.close(master)
        os.setsid()
        os.dup2(slave, 0)
        os.dup2(slave, 1)
        os.dup2(slave, 2)
        os.close(slave)
        os.execvpe("./target/release/frontharness", ["frontharness"], env)
        sys.exit(1)
        
    os.close(slave)
    print(f"[PTY DevServer Test] Started PID={pid}", flush=True)
    time.sleep(1.0)
    
    def read_nonblocking():
        buf = b""
        for _ in range(10):
            try:
                data = os.read(master, 16384)
                if data:
                    buf += data
                else:
                    break
            except BlockingIOError:
                time.sleep(0.05)
            except OSError:
                break
        return buf.decode("utf-8", errors="ignore")

    _ = read_nonblocking()

    # 1. Press 'p' to start DevServer
    print("[Step 1] Pressing 'p' to start DevServer...", flush=True)
    os.write(master, b"p")
    time.sleep(1.0)
    screen_after_p = read_nonblocking()
    clean_p = re.sub(r'\x1b\[[0-9;]*[a-zA-Z]', '', screen_after_p)
    print("[Step 1] Cleaned Screen after pressing 'p':\n", clean_p, flush=True)
    assert re.search(r'Running\s*\(3000\)', clean_p), "DevServer failed to show Running in statusline!"

    # 2. Test HTTP connection to localhost:3000
    print("[Step 2] Testing HTTP GET http://localhost:3000...", flush=True)
    req = urllib.request.Request("http://localhost:3000/")
    with urllib.request.urlopen(req, timeout=3) as resp:
        code = resp.getcode()
        content = resp.read()
        print(f"[Step 2] HTTP status: {code}, read {len(content)} bytes", flush=True)
        assert code == 200, f"Expected status 200 but got {code}"
        assert b"<!DOCTYPE html>" in content or b"<html" in content

    # 3. Press 'p' to stop DevServer
    print("[Step 3] Pressing 'p' again to stop DevServer...", flush=True)
    os.write(master, b"p")
    time.sleep(0.8)
    screen_after_stop = read_nonblocking()
    clean_stop = re.sub(r'\x1b\[[0-9;]*[a-zA-Z]', '', screen_after_stop)
    assert "Stopped" in clean_stop, "DevServer failed to show Stopped in statusline!"
    print("[Step 3] DevServer stopped successfully!", flush=True)

    # 4. Quit TUI
    os.write(master, b"q")
    time.sleep(0.5)
    try:
        os.kill(pid, 9)
    except Exception:
        pass
    os.close(master)
    print("[PTY DevServer Test] All tests passed with code 0!", flush=True)

if __name__ == "__main__":
    run_test()
