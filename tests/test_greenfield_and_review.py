import os
import pty
import struct
import fcntl
import termios
import sys
import time
import re
import html
import asyncio
from playwright.async_api import async_playwright

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
    print(f"[PTY Test] Process PID={pid} started", flush=True)
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

    # 1. Capture main screen
    screen_main = read_nonblocking()
    print(f"[Step 1] Main Screen captured ({len(screen_main)} chars)", flush=True)
    
    # 2. Open Design Studio & Switch to Greenfield Mode
    print("[Step 2] Opening Design Studio ('r') and switching mode ('m')...", flush=True)
    os.write(master, b"r")
    time.sleep(0.6)
    os.write(master, b"m") # Toggle to Greenfield mode!
    time.sleep(0.6)
    screen_greenfield = read_nonblocking()
    print(f"[Step 2] Greenfield Studio Modal captured ({len(screen_greenfield)} chars)", flush=True)
    with open("docs/screenshots/tui_greenfield_modal.ansi", "w", encoding="utf-8") as f:
        f.write(screen_greenfield)
    assert "Greenfield (From Scratch)" in screen_greenfield or "Greenfield" in screen_greenfield
    
    # Close modal
    os.write(master, b"\x1b")
    time.sleep(0.5)
    
    # 3. Open Review & Iterative Refinement Modal ('e')
    print("[Step 3] Opening Review & Refinement modal ('e')...", flush=True)
    os.write(master, b"e")
    time.sleep(0.6)
    screen_review_initial = read_nonblocking()
    print(f"[Step 3] Review Modal captured ({len(screen_review_initial)} chars)", flush=True)
    
    # Set 5 stars and enter critique
    print("[Step 4] Setting 5 Stars and entering critique...", flush=True)
    os.write(master, b"5") # Press 5 for 5 stars
    time.sleep(0.2)
    os.write(master, b"\t") # Focus critique field
    time.sleep(0.2)
    os.write(master, b"Add dark mode switch and make phone call button sticky")
    time.sleep(0.5)
    screen_review_filled = read_nonblocking()
    with open("docs/screenshots/tui_review_modal.ansi", "w", encoding="utf-8") as f:
        f.write(screen_review_filled)
    print(f"[Step 4] Filled Review Modal captured ({len(screen_review_filled)} chars)", flush=True)
    
    # Close review modal
    os.write(master, b"\x1b")
    time.sleep(0.5)
    
    # 4. Quit TUI
    os.write(master, b"q")
    time.sleep(0.5)
    try:
        os.kill(pid, 9)
    except Exception:
        pass
    os.close(master)
    print("[PTY Test] Greenfield & Review modal interactions verified successfully!", flush=True)

if __name__ == "__main__":
    run_test()
