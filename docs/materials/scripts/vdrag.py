#!/usr/bin/env python3
"""Scripted interactive drag for nested niri (docs/specs/2026-10-01-drag-follow-lag-design.md
§7). Maps a striped wl_shm window (app-id vdrag), finds it with a zwlr_virtual_pointer_v1,
presses the left button on it and starts the move itself with xdg_toplevel.move, then
moves at a fixed cadence, holds and releases. Stdlib only; exits non-zero on any
protocol error or timeout."""

import argparse
import mmap
import os
import pathlib
import select
import signal
import socket
import struct
import sys
import time

BTN_LEFT = 0x110
ARGB8888 = 0


def u32(v):
    return struct.pack("<I", v & 0xFFFFFFFF)


def i32(v):
    return struct.pack("<i", v)


def fixed(v):
    return struct.pack("<i", int(round(v * 256)))


def string(s):
    b = s.encode() + b"\0"
    return u32(len(b)) + b + b"\0" * (-len(b) % 4)


def message(obj, opcode, *args):
    body = b"".join(args)
    return struct.pack("<II", obj, ((8 + len(body)) << 16) | opcode) + body


def split_messages(buf):
    """Complete (obj, opcode, body) messages from buf, and the incomplete tail."""
    out = []
    while len(buf) >= 8:
        obj, word = struct.unpack("<II", buf[:8])
        size = word >> 16
        if len(buf) < size:
            break
        out.append((obj, word & 0xFFFF, buf[8:size]))
        buf = buf[size:]
    return out, buf


def parse_global(body):
    name, n = struct.unpack("<II", body[:8])
    iface = body[8 : 8 + n - 1].decode()
    off = 8 + n + (-n % 4)
    (version,) = struct.unpack("<I", body[off : off + 4])
    return name, iface, version


def stripes(width, height):
    """Premultiplied ARGB8888 rows: translucent 32 px stripes at alpha 0.6. Opaque
    pixels skip the material shader and would show glass only on the bevel."""
    a = struct.pack("<I", 0x998B8B8B)  # (0xE8, 0xE8, 0xE8) × 0.6
    b = struct.pack("<I", 0x9919386A)  # (0x2A, 0x5D, 0xB0) × 0.6
    row = b"".join((a if (x // 32) % 2 == 0 else b) for x in range(width))
    return row * height


class Conn:
    def __init__(self, path):
        self.sock = socket.socket(socket.AF_UNIX, socket.SOCK_STREAM)
        self.sock.connect(path)
        self.next_id = 2
        self.handlers = {1: self.on_display}
        self.buf = b""

    def new_id(self, handler=None):
        i = self.next_id
        self.next_id += 1
        if handler:
            self.handlers[i] = handler
        return i

    def send(self, obj, opcode, *args, fds=()):
        data = message(obj, opcode, *args)
        if fds:
            socket.send_fds(self.sock, [data], list(fds))
        else:
            self.sock.sendall(data)

    def on_display(self, op, body):
        if op == 0:
            oid, code = struct.unpack("<II", body[:8])
            sys.exit(f"vdrag: protocol error on object {oid}, code {code}")

    def dispatch(self, timeout):
        r, _, _ = select.select([self.sock], [], [], timeout)
        if not r:
            return
        chunk = self.sock.recv(65536)
        if not chunk:
            sys.exit("vdrag: compositor closed the connection")
        msgs, self.buf = split_messages(self.buf + chunk)
        for obj, op, body in msgs:
            h = self.handlers.get(obj)
            if h:
                h(op, body)

    def roundtrip(self, limit=5.0):
        done = []
        cb = self.new_id(lambda op, body: done.append(1))
        self.send(1, 0, u32(cb))
        end = time.monotonic() + limit
        while not done:
            if time.monotonic() > end:
                sys.exit("vdrag: roundtrip timed out")
            self.dispatch(0.1)


class Window:
    def __init__(self, c, globals_):
        self.c = c
        self.size = (640, 480)
        self.pending = None
        self.configured = False
        self.entered = False
        self.walk_y = 0
        self.button_serial = None

        def bind(iface, version, handler=None):
            name, _ = globals_[iface]
            new = c.new_id(handler)
            c.send(self.registry, 0, u32(name), string(iface), u32(version), u32(new))
            return new

        self.registry = globals_["__registry__"]
        self.compositor = bind("wl_compositor", 4)
        self.shm = bind("wl_shm", 1)
        self.wm = bind("xdg_wm_base", 1, self.on_wm)
        self.seat = bind("wl_seat", 5)
        self.vpm = bind("zwlr_virtual_pointer_manager_v1", 1)
        self.surface = c.new_id()
        c.send(self.compositor, 0, u32(self.surface))
        self.xdg_surface = c.new_id(self.on_xdg_surface)
        c.send(self.wm, 2, u32(self.xdg_surface), u32(self.surface))
        self.toplevel = c.new_id(self.on_toplevel)
        c.send(self.xdg_surface, 1, u32(self.toplevel))
        c.send(self.toplevel, 2, string("vdrag"))  # set_title
        c.send(self.toplevel, 3, string("vdrag"))  # set_app_id
        self.pointer = c.new_id(self.on_pointer)
        c.send(self.seat, 0, u32(self.pointer))  # get_pointer
        self.vp = c.new_id()
        c.send(self.vpm, 0, u32(self.seat), u32(self.vp))  # create_virtual_pointer
        c.send(self.surface, 6)  # initial commit, no buffer

    def on_wm(self, op, body):
        if op == 0:  # ping
            self.c.send(self.wm, 3, body[:4])

    def on_toplevel(self, op, body):
        if op == 0:  # configure(width, height, states)
            w, h = struct.unpack("<ii", body[:8])
            if w > 0 and h > 0:
                self.pending = (w, h)
        elif op == 1:
            sys.exit("vdrag: closed by the compositor")

    def on_xdg_surface(self, op, body):
        if op == 0:  # configure(serial)
            (serial,) = struct.unpack("<I", body[:4])
            if self.pending:
                self.size, self.pending = self.pending, None
            self.c.send(self.xdg_surface, 4, u32(serial))  # ack_configure
            self.draw()
            self.configured = True

    def on_pointer(self, op, body):
        if op == 0:  # enter(serial, surface, x, y)
            if struct.unpack("<I", body[4:8])[0] == self.surface:
                self.entered = True
        elif op == 3:  # button(serial, time, button, state)
            serial, _, button, state = struct.unpack("<IIII", body[:16])
            if button == BTN_LEFT and state == 1:
                self.button_serial = serial

    def draw(self):
        w, h = self.size
        stride = w * 4
        size = stride * h
        fd = os.memfd_create("vdrag-shm")
        os.ftruncate(fd, size)
        with mmap.mmap(fd, size) as m:
            m.write(stripes(w, h))
        pool = self.c.new_id()
        self.c.send(self.shm, 0, u32(pool), u32(size), fds=[fd])  # create_pool(id, fd, size)
        os.close(fd)
        buf = self.c.new_id()
        self.c.send(pool, 0, u32(buf), i32(0), i32(w), i32(h), i32(stride), u32(ARGB8888))
        self.c.send(pool, 1)  # destroy pool; the buffer keeps the memory
        self.c.send(self.surface, 1, u32(buf), i32(0), i32(0))  # attach
        self.c.send(self.surface, 2, i32(0), i32(0), i32(w), i32(h))  # damage
        self.c.send(self.surface, 6)  # commit


def win_entered_y(win):
    """The pointer's output y when the walk entered the window."""
    return win.walk_y


def wait_for(c, pred, secs, what):
    end = time.monotonic() + secs
    while not pred():
        if time.monotonic() > end:
            sys.exit(f"vdrag: timed out waiting for {what}")
        c.dispatch(0.02)


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--socket", required=True, help="absolute path of the nested niri socket")
    ap.add_argument("--extent", required=True, help="W,H of the output, logical px")
    ap.add_argument("--dx", type=float, required=True)
    ap.add_argument("--dy", type=float, default=0.0)
    ap.add_argument("--frames", type=int, required=True)
    ap.add_argument("--hz", type=float, default=125.0)
    ap.add_argument("--hold-ms", type=int, default=600)
    ap.add_argument("--lift-ms", type=int, default=500, help="pause after the vertical lift")
    ap.add_argument("--ready", required=True)
    ap.add_argument("--go", required=True)
    ap.add_argument("--done", required=True)
    args = ap.parse_args()
    signal.signal(signal.SIGTERM, lambda *_: sys.exit(0))
    w_ext, h_ext = (int(v) for v in args.extent.split(","))

    c = Conn(args.socket)
    found = {}
    registry = c.new_id(
        lambda op, body: op == 0 and found.__setitem__(parse_global(body)[1], parse_global(body)[0::2])
    )
    c.send(1, 1, u32(registry))  # get_registry
    c.roundtrip()
    for need in ("wl_compositor", "wl_shm", "xdg_wm_base", "wl_seat", "zwlr_virtual_pointer_manager_v1"):
        if need not in found:
            sys.exit(f"vdrag: compositor lacks {need}")
    found["__registry__"] = registry
    win = Window(c, found)
    wait_for(c, lambda: win.configured, 5, "first configure")
    c.roundtrip()
    time.sleep(0.5)  # open animation

    t0 = time.monotonic()

    def ms():
        return int((time.monotonic() - t0) * 1000)

    # Walk the pointer across three rows until it enters our window.
    for y in (h_ext // 2, h_ext // 3, 2 * h_ext // 3):
        for x in range(8, w_ext, 16):
            win.walk_y = y
            c.send(win.vp, 1, u32(ms()), u32(x), u32(y), u32(w_ext), u32(h_ext))  # motion_absolute
            c.send(win.vp, 4)  # frame
            c.roundtrip()
            if win.entered:
                break
        if win.entered:
            break
    if not win.entered:
        sys.exit("vdrag: pointer never entered the window")
    pathlib.Path(args.ready).touch()
    wait_for(c, lambda: os.path.exists(args.go), 60, "go file")

    c.send(win.vp, 2, u32(ms()), u32(BTN_LEFT), u32(1))  # button press
    c.send(win.vp, 4)
    wait_for(c, lambda: win.button_serial is not None, 2, "button event")
    c.send(win.toplevel, 5, u32(win.seat), u32(win.button_serial))  # xdg_toplevel.move
    c.roundtrip()
    period = 1.0 / args.hz
    # Lift vertically first. The grab decides between a window move and a view pan
    # after 8 px: mostly horizontal motion on a tiled window pans the view, which never
    # reaches the follower. Vertical motion selects the move, and 260 px crosses the
    # 256 px lift threshold. Then a pause lets the lift's own flex settle before the
    # timed segment.
    lift_dy = -10.0 if win_entered_y(win) > h_ext // 3 else 10.0
    for _ in range(26):
        c.send(win.vp, 0, u32(ms()), fixed(0.0), fixed(lift_dy))
        c.send(win.vp, 4)
        end = time.monotonic() + period
        while time.monotonic() < end:
            c.dispatch(max(0.0, end - time.monotonic()))
    end = time.monotonic() + args.lift_ms / 1000
    while time.monotonic() < end:
        c.dispatch(max(0.0, end - time.monotonic()))
    for _ in range(args.frames):
        c.send(win.vp, 0, u32(ms()), fixed(args.dx), fixed(args.dy))  # motion
        c.send(win.vp, 4)
        end = time.monotonic() + period
        while time.monotonic() < end:
            c.dispatch(max(0.0, end - time.monotonic()))
    end = time.monotonic() + args.hold_ms / 1000
    while time.monotonic() < end:
        c.dispatch(max(0.0, end - time.monotonic()))
    c.send(win.vp, 2, u32(ms()), u32(BTN_LEFT), u32(0))  # release
    c.send(win.vp, 4)
    c.roundtrip()
    pathlib.Path(args.done).touch()
    while True:
        c.dispatch(1.0)


if __name__ == "__main__":
    main()
