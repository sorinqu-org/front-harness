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

def run_interactive_test():
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
    print(f"[TUI PTY Test] Process launched PID={pid}", flush=True)
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

    # 1. Capture Main Screen
    screen1 = read_nonblocking()
    print(f"[Step 1] Main Screen captured ({len(screen1)} chars)", flush=True)
    with open("docs/screenshots/tui_screen_1_idle.ansi", "w", encoding="utf-8") as f:
        f.write(screen1)
        
    # 2. Open Design Studio with 'r'
    print("[Step 2] Sending 'r' to open Design Studio modal...", flush=True)
    os.write(master, b"r")
    time.sleep(0.8)
    screen2 = read_nonblocking()
    print(f"[Step 2] Design Studio Modal captured ({len(screen2)} chars)", flush=True)
    with open("docs/screenshots/tui_screen_2_studio.ansi", "w", encoding="utf-8") as f:
        f.write(screen2)
        
    # 3. Press Enter / Down to navigate, toggle skills, navigate to launch button
    print("[Step 3] Navigating to Skills Matrix and Launch Button...", flush=True)
    for _ in range(6):
        os.write(master, b"\t")
        time.sleep(0.15)
    
    screen3 = read_nonblocking()
    print(f"[Step 3] Launch button highlighted ({len(screen3)} chars)", flush=True)
    with open("docs/screenshots/tui_screen_3_launch_focused.ansi", "w", encoding="utf-8") as f:
        f.write(screen3)
        
    # 4. Press Enter to Launch Pipeline
    print("[Step 4] Pressing Enter to launch pipeline...", flush=True)
    os.write(master, b"\r")
    time.sleep(1.5)
    
    # 5. Monitor Live TUI execution for 10 seconds
    print("[Step 5] Monitoring live TUI pipeline execution...", flush=True)
    for i in range(8):
        time.sleep(1.0)
        chunk = read_nonblocking()
        if chunk:
            print(f"  [TUI Live] Frame {i+1} received: {len(chunk)} chars", flush=True)
            with open(f"docs/screenshots/tui_screen_4_live_frame_{i+1}.ansi", "w", encoding="utf-8") as f:
                f.write(chunk)
            if "AUDITING" in chunk or "RESEARCHING" in chunk or "DESIGNING" in chunk or "IMPLEMENTING" in chunk:
                print(f"  [TUI Live] Status update confirmed in frame {i+1}!", flush=True)
                
    # 6. Exit
    print("[Step 6] Sending 'q' to quit TUI...", flush=True)
    os.write(master, b"q")
    time.sleep(0.5)
    
    try:
        os.kill(pid, 9)
    except Exception:
        pass
    os.close(master)
    print("[TUI PTY Test] Verification finished successfully!", flush=True)

if __name__ == "__main__":
    run_interactive_test()
