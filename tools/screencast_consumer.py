#!/usr/bin/env python3
"""A real screencast consumer for the optic-settling TTY lane (material-3acc86,
docs/specs/2026-10-02-real-tty-settling-lane-design.md §4).

Opens an org.gnome.Mutter.ScreenCast session on the session bus (the case's
private bus, where niri on DRM serves it), records one monitor with the
cursor hidden, and consumes the PipeWire node through GStreamer
(pipewiresrc ! glupload ! gldownload ! RGB appsink: niri sends DMA-BUFs only,
imported through EGL without a display). Every received frame is
journalled as one CLOCK_MONOTONIC line in FRAMES, the clock of niri's optic
edges. "ready <node>" goes to stdout once the pipeline plays: niri sends no
frame without damage, so readiness cannot wait for one.

SIGUSR1 arms a sample: the target path is read from REQUEST_FILE, the
request time is written to <target>.armed, and the first frame arriving
after it is saved as packed RGB to <target>, then its times and size to
<target>.json. SIGTERM or SIGINT stops the session and writes SUMMARY.

    screencast_consumer.py CONNECTOR FRAMES SUMMARY REQUEST_FILE
"""

import json
import os
import signal
import sys
import tempfile
import time
from pathlib import Path

BUS_NAME = 'org.gnome.Mutter.ScreenCast'
NODE_TIMEOUT_S = 10
CURSOR_HIDDEN = 0
PIPELINE = ('pipewiresrc path={node} ! video/x-raw(memory:DMABuf),format=DMA_DRM '
            '! glupload ! glcolorconvert ! gldownload ! videoconvert ! video/x-raw,format=RGB '
            '! appsink name=sink emit-signals=true sync=false')
# A TTY has no display server: GStreamer GL uses EGL with a pbuffer surface.
GL_ENV = {'GST_GL_PLATFORM': 'egl', 'GST_GL_WINDOW': 'surfaceless'}


def wait_for(subscribe, timeout_s):
    """Run a private main loop until subscribe's deliver(value) or the timeout.

    subscribe(deliver) starts the wait and returns an unsubscribe callable.
    The timeout source is removed before returning, so it can never stop a
    later loop: the consumer's frames run long after this deadline."""
    from gi.repository import GLib

    loop = GLib.MainLoop()
    found = []

    def deliver(value):
        if not found:
            found.append(value)
            loop.quit()

    def expire():
        loop.quit()
        return GLib.SOURCE_REMOVE

    unsubscribe = subscribe(deliver)
    timer = GLib.timeout_add(int(timeout_s * 1000), expire)
    loop.run()
    if found:
        GLib.source_remove(timer)
    unsubscribe()
    return found[0] if found else None


def packed_rgb(data, width, height, stride):
    """RGB rows without GStreamer's row padding."""
    row = width * 3
    needed = (height - 1) * stride + row
    if len(data) < needed:
        raise ValueError(f'frame buffer holds {len(data)} bytes, {width}x{height} at stride {stride} needs {needed}')
    if stride == row:
        return bytes(data[:row * height])
    return b''.join(bytes(data[y * stride:y * stride + row]) for y in range(height))


def write_atomically(path, data):
    """Write bytes so a reader sees the whole file or none: a temp file in the
    same directory, then os.replace."""
    path = Path(path)
    descriptor, temporary = tempfile.mkstemp(dir=path.parent, prefix=f'.{path.name}.')
    try:
        with os.fdopen(descriptor, 'wb') as handle:
            handle.write(data)
        os.replace(temporary, path)
    except BaseException:
        Path(temporary).unlink(missing_ok=True)
        raise


class Sampler:
    """Saves the first frame that arrives after an armed request."""

    def __init__(self):
        self.pending = None

    def request(self, path, now_ns):
        if self.pending is not None:
            raise RuntimeError('a sample is already pending')
        Path(f'{path}.armed').write_text(f'{now_ns}\n')
        self.pending = (Path(path), now_ns)

    def offer(self, data, width, height, arrival_ns):
        if self.pending is None:
            return False
        path, requested = self.pending
        if arrival_ns <= requested:
            return False
        # Raw first, JSON last: the driver waits for the JSON.
        write_atomically(path, data)
        write_atomically(f'{path}.json', (json.dumps(dict(
            request_mono_ns=requested, frame_mono_ns=arrival_ns, width=width, height=height)) + '\n').encode())
        self.pending = None
        return True


def main():
    import gi
    gi.require_version('Gst', '1.0')
    gi.require_version('GstVideo', '1.0')
    gi.require_version('Gio', '2.0')
    from gi.repository import Gio, GLib, Gst, GstVideo

    connector, frames_path, summary_path, request_file = sys.argv[1:5]
    os.environ.update(GL_ENV)
    Gst.init(None)
    factory = Gst.ElementFactory.find('pipewiresrc')
    if factory is None:
        sys.exit('screencast-consumer: no pipewiresrc element (gst-plugin-pipewire)')

    bus = Gio.bus_get_sync(Gio.BusType.SESSION, None)

    def call(path, interface, method, args, reply):
        return bus.call_sync(BUS_NAME, path, interface, method, args, GLib.VariantType(reply),
                             Gio.DBusCallFlags.NONE, 5000, None).unpack()

    (session,) = call('/org/gnome/Mutter/ScreenCast', 'org.gnome.Mutter.ScreenCast',
                      'CreateSession', GLib.Variant('(a{sv})', ({},)), '(o)')

    # A stop before the GLib signal sources below exist would take the
    # default action and skip the finally: exit through it instead, nonzero,
    # since no summary is written. The GLib sources replace these handlers.
    def early_stop(number, _frame):
        raise SystemExit(f'screencast-consumer: stopped by {signal.Signals(number).name} '
                         'before the cast started; no summary written')

    signal.signal(signal.SIGTERM, early_stop)
    signal.signal(signal.SIGINT, early_stop)
    # The session exists from here on: every exit stops it, and a failing
    # Stop never replaces the error or exit status already on its way.
    try:
        properties = {'cursor-mode': GLib.Variant('u', CURSOR_HIDDEN)}
        (stream,) = call(session, 'org.gnome.Mutter.ScreenCast.Session', 'RecordMonitor',
                         GLib.Variant('(sa{sv})', (connector, properties)), '(o)')

        def subscribe(deliver):
            handle = bus.signal_subscribe(
                BUS_NAME, 'org.gnome.Mutter.ScreenCast.Stream', 'PipeWireStreamAdded', stream, None,
                Gio.DBusSignalFlags.NONE, lambda *args: deliver(args[-1].unpack()[0]))
            call(session, 'org.gnome.Mutter.ScreenCast.Session', 'Start', None, '()')
            return lambda: bus.signal_unsubscribe(handle)

        node = wait_for(subscribe, NODE_TIMEOUT_S)
        if node is None:
            sys.exit(f'screencast-consumer: no PipeWireStreamAdded within {NODE_TIMEOUT_S} s')

        # niri offers DMA-BUFs only (pw_utils: dataType DmaBuf) with the modifier
        # it fixates from the consumer's list. glupload imports any of them
        # through EGL; gldownload brings the frame to system memory.
        # SCREENCAST_CONSUMER_PIPELINE: offline tests substitute a test source.
        description = os.environ.get('SCREENCAST_CONSUMER_PIPELINE', PIPELINE)
        try:
            pipeline = Gst.parse_launch(description.format(node=node))
        except GLib.Error as error:
            sys.exit(f'screencast-consumer: cannot build the pipeline: {error.message}')
        sink = pipeline.get_by_name('sink')
        loop = GLib.MainLoop()
        frames = open(frames_path, 'w', buffering=1)
        times = []
        sampler = Sampler()

        failures = []

        def fail(error):
            sys.stderr.write(f'screencast-consumer: {error}\n')
            failures.append(str(error))
            loop.quit()

        def on_sample(appsink):
            try:
                return handle_sample(appsink)
            except Exception as error:
                fail(error)
                return Gst.FlowReturn.ERROR

        def handle_sample(appsink):
            sample = appsink.emit('pull-sample')
            now = time.monotonic_ns()
            times.append(now)
            frames.write(f'{now}\n')
            if sampler.pending is not None:
                info = GstVideo.VideoInfo.new_from_caps(sample.get_caps())
                buffer = sample.get_buffer()
                ok, mapped = buffer.map(Gst.MapFlags.READ)
                if not ok:
                    raise RuntimeError('cannot map a frame')
                try:
                    meta = GstVideo.buffer_get_video_meta(buffer)
                    stride = meta.stride[0] if meta is not None else info.stride[0]
                    data = packed_rgb(mapped.data, info.width, info.height, stride)
                finally:
                    buffer.unmap(mapped)
                sampler.offer(data, info.width, info.height, now)
            return Gst.FlowReturn.OK

        def on_error(_bus, message):
            if failures:
                return    # the failure is already reported; on_sample's ERROR return posts a second
            error, debug = message.parse_error()
            fail(f'{error.message} ({debug})')

        def on_request():
            try:
                sampler.request(Path(request_file).read_text().strip(), time.monotonic_ns())
            except Exception as error:
                fail(error)
                return GLib.SOURCE_REMOVE
            return GLib.SOURCE_CONTINUE

        stopping = []

        def stop():
            stopping.append(True)
            loop.quit()
            return GLib.SOURCE_REMOVE

        sink.connect('new-sample', on_sample)
        pipeline_bus = pipeline.get_bus()
        pipeline_bus.add_signal_watch()
        pipeline_bus.connect('message::error', on_error)
        GLib.unix_signal_add(GLib.PRIORITY_HIGH, signal.SIGUSR1, on_request)
        GLib.unix_signal_add(GLib.PRIORITY_HIGH, signal.SIGTERM, stop)
        GLib.unix_signal_add(GLib.PRIORITY_HIGH, signal.SIGINT, stop)
        if pipeline.set_state(Gst.State.PLAYING) == Gst.StateChangeReturn.FAILURE:
            message = pipeline_bus.timed_pop_filtered(Gst.SECOND, Gst.MessageType.ERROR)
            reason = ''
            if message is not None:
                error, debug = message.parse_error()
                reason = f': {error.message} ({debug})'
            pipeline.set_state(Gst.State.NULL)
            sys.exit(f'screencast-consumer: the pipeline did not start{reason}')
        print(f'ready {node}', flush=True)
        loop.run()

        pipeline.set_state(Gst.State.NULL)
        frames.close()
        summary = dict(
            consumer='gstreamer pipewiresrc ! glupload ! gldownload ! videoconvert ! appsink',
            gstreamer=Gst.version_string(),
            pipewiresrc=f'{factory.get_plugin_name()} {factory.get_plugin().get_version()}',
            connector=connector, node=node, frames=len(times),
            first_mono_ns=times[0] if times else None, last_mono_ns=times[-1] if times else None,
            stopped_by_signal=bool(stopping),
        )
        Path(summary_path).write_text(json.dumps(summary, indent=2) + '\n')
        if failures:
            sys.exit(1)
        if not stopping:
            sys.exit('screencast-consumer: the pipeline stopped before a stop signal')

    finally:
        try:
            call(session, 'org.gnome.Mutter.ScreenCast.Session', 'Stop', None, '()')
        except GLib.Error as error:
            if sys.exc_info()[0] is None:
                raise
            sys.stderr.write(f'screencast-consumer: Session.Stop failed: {error.message}\n')


if __name__ == '__main__':
    main()
