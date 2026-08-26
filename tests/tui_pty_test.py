import os
import pty
import select
import struct
import fcntl
import termios
import sys
import time

def set_winsize(fd, rows, cols):
    winsize = struct.pack("HHHH", rows, cols, 0, 0)
    fcntl.ioctl(fd, termios.TIOCSWINSZ, winsize)

def run_tui_test():
    master, slave = pty.openpty()
    set_winsize(master, 45, 130)
    
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
        os.execvpe("./target/debug/frontharness", ["frontharness"], env)
        sys.exit(1)
        
    os.close(slave)
    
    print("[PTY Test] Forked TUI process, PID:", pid)
    time.sleep(1)
    
    # Read initial screen
    output = b""
    for _ in range(5):
        r, _, _ = select.select([master], [], [], 0.3)
        if r:
            output += os.read(master, 8192)
        
    clean_out = output.decode("utf-8", errors="ignore")
    print(f"[PTY Test] Initial render captured ({len(clean_out)} chars)")
    with open("tests/tui_screen_1_initial.txt", "w", encoding="utf-8") as f:
        f.write(clean_out)
    
    # Send 'r' key to open Design Studio modal
    print("[PTY Test] Sending 'r' key...")
    os.write(master, b"r")
    time.sleep(0.8)
    
    output = b""
    for _ in range(5):
        r, _, _ = select.select([master], [], [], 0.3)
        if r:
            output += os.read(master, 8192)
        
    clean_modal = output.decode("utf-8", errors="ignore")
    print(f"[PTY Test] Modal render captured ({len(clean_modal)} chars)")
    with open("tests/tui_screen_2_modal.txt", "w", encoding="utf-8") as f:
        f.write(clean_modal)
        
    if "Design Studio" in clean_modal:
        print("[PTY Test] SUCCESS: Design Studio modal rendered!")
    else:
        print("[PTY Test] WARNING: 'Design Studio' text not found in modal output")
        
    # Send 'q' or 'Esc' to exit
    print("[PTY Test] Sending 'Esc' and 'q' to quit...")
    os.write(master, b"\x1b")
    time.sleep(0.5)
    os.write(master, b"q")
    time.sleep(0.5)
    
    try:
        os.kill(pid, 9)
    except Exception:
        pass
    os.close(master)
    print("[PTY Test] Test finished cleanly.")

if __name__ == "__main__":
    run_tui_test()
