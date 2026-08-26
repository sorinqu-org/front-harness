import os
import pty
import struct
import fcntl
import termios
import sys
import time

def set_winsize(fd, rows, cols):
    winsize = struct.pack("HHHH", rows, cols, 0, 0)
    fcntl.ioctl(fd, termios.TIOCSWINSZ, winsize)

def test_run_from_home():
    master, slave = pty.openpty()
    set_winsize(master, 45, 130)
    os.set_blocking(master, False)
    
    env = os.environ.copy()
    env["TERM"] = "xterm-256color"
    
    # Run fh from /home/yuwye (outside front-harness repo directory)
    pid = os.fork()
    if pid == 0:
        os.close(master)
        os.setsid()
        os.dup2(slave, 0)
        os.dup2(slave, 1)
        os.dup2(slave, 2)
        os.close(slave)
        os.chdir("/home/yuwye")
        os.execvpe("/home/yuwye/.local/bin/fh", ["fh"], env)
        sys.exit(1)
        
    os.close(slave)
    print(f"[Test Home Dir] Process PID={pid} started in /home/yuwye", flush=True)
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

    screen1 = read_nonblocking()
    print(f"[Test Home Dir] Screen 1 captured ({len(screen1)} chars)", flush=True)
    
    # Press 'r' to open Design Studio modal
    print("[Test Home Dir] Sending 'r'...", flush=True)
    os.write(master, b"r")
    time.sleep(0.8)
    screen2 = read_nonblocking()
    print(f"[Test Home Dir] Modal opened ({len(screen2)} chars)", flush=True)
    
    # Move to launch button and press Enter
    print("[Test Home Dir] Navigating to Launch button...", flush=True)
    for _ in range(6):
        os.write(master, b"\t")
        time.sleep(0.1)
    
    print("[Test Home Dir] Pressing Enter to launch...", flush=True)
    os.write(master, b"\r")
    time.sleep(2.0)
    
    # Monitor for 10 seconds to verify Playwright crawler runs without Errno 2
    success_observed = False
    error_observed = False
    
    for i in range(8):
        time.sleep(1.2)
        chunk = read_nonblocking()
        if chunk:
            print(f"  [Live Check {i+1}] Received frame: {len(chunk)} chars", flush=True)
            if "can't open file" in chunk or "Errno 2" in chunk:
                print("  [ERROR DETECTED] Crawler script not found!", flush=True)
                error_observed = True
            if "AUDITING" in chunk or "RESEARCHING" in chunk or "DESIGNING" in chunk:
                print("  [OK] Status progressing normally!", flush=True)
                success_observed = True
                
    # Quit TUI
    os.write(master, b"q")
    time.sleep(0.5)
    try:
        os.kill(pid, 9)
    except Exception:
        pass
    os.close(master)
    
    if error_observed:
        print("[FAIL] Test observed 'Errno 2' or missing script error!", flush=True)
        sys.exit(1)
    elif success_observed:
        print("[PASS] Verified! TUI launched from /home/yuwye without any script path error!", flush=True)
    else:
        print("[WARN] No status change detected, checking frames.", flush=True)

if __name__ == "__main__":
    test_run_from_home()
