#!/usr/bin/env python3
"""A real screencast consumer for the optic-settling capture (material-3acc86).

Opens an org.gnome.Mutter.ScreenCast session on the session bus (the
capture's private bus, where the nested niri serves it), records one
monitor, and consumes the PipeWire node with GStreamer (pipewiresrc !
appsink). Every received frame is journalled as one CLOCK_MONOTONIC line in
FRAMES, the clock of niri's optic edges. "ready <node>" goes to stdout once
the first frame arrives. SIGTERM or SIGINT stops the session and writes
SUMMARY: the consumer's identity, its node, frame count and capture interval.

    screencast-consumer.py CONNECTOR FRAMES SUMMARY
"""

import json
import signal
import sys
import time

import gi

gi.require_version('Gst', '1.0')
gi.require_version('Gio', '2.0')
from gi.repository import Gio, GLib, Gst  # noqa: E402

BUS_NAME = 'org.gnome.Mutter.ScreenCast'
NODE_TIMEOUT_S = 10


def main():
    connector, frames_path, summary_path = sys.argv[1:4]
    Gst.init(None)
    factory = Gst.ElementFactory.find('pipewiresrc')
    if factory is None:
        sys.exit('screencast-consumer: no pipewiresrc element (gst-plugin-pipewire)')

    bus = Gio.bus_get_sync(Gio.BusType.SESSION, None)
    loop = GLib.MainLoop()

    def call(path, interface, method, args, reply):
        return bus.call_sync(BUS_NAME, path, interface, method, args, GLib.VariantType(reply),
                             Gio.DBusCallFlags.NONE, 5000, None).unpack()

    (session,) = call('/org/gnome/Mutter/ScreenCast', 'org.gnome.Mutter.ScreenCast',
                      'CreateSession', GLib.Variant('(a{sv})', ({},)), '(o)')
    (stream,) = call(session, 'org.gnome.Mutter.ScreenCast.Session', 'RecordMonitor',
                     GLib.Variant('(sa{sv})', (connector, {})), '(o)')
    node = []

    def on_stream_added(_bus, _sender, _path, _interface, _signal, params):
        node.append(params.unpack()[0])
        loop.quit()

    bus.signal_subscribe(BUS_NAME, 'org.gnome.Mutter.ScreenCast.Stream', 'PipeWireStreamAdded',
                         stream, None, Gio.DBusSignalFlags.NONE, on_stream_added)
    call(session, 'org.gnome.Mutter.ScreenCast.Session', 'Start', None, '()')
    GLib.timeout_add_seconds(NODE_TIMEOUT_S, loop.quit)
    loop.run()
    if not node:
        sys.exit(f'screencast-consumer: no PipeWireStreamAdded within {NODE_TIMEOUT_S} s')

    pipeline = Gst.Pipeline.new('cast')
    source = Gst.ElementFactory.make('pipewiresrc', 'source')
    source.set_property('path', str(node[0]))
    sink = Gst.ElementFactory.make('appsink', 'sink')
    sink.set_property('emit-signals', True)
    sink.set_property('sync', False)
    pipeline.add(source)
    pipeline.add(sink)
    if not source.link(sink):
        sys.exit('screencast-consumer: cannot link pipewiresrc to appsink')

    frames = open(frames_path, 'w', buffering=1)
    times = []

    def on_sample(appsink):
        appsink.emit('pull-sample')
        now = time.monotonic_ns()
        times.append(now)
        frames.write(f'{now}\n')
        if len(times) == 1:
            print(f'ready {node[0]}', flush=True)
        return Gst.FlowReturn.OK

    def on_error(_bus, message):
        error, debug = message.parse_error()
        sys.stderr.write(f'screencast-consumer: {error.message} ({debug})\n')
        loop.quit()

    sink.connect('new-sample', on_sample)
    pipeline_bus = pipeline.get_bus()
    pipeline_bus.add_signal_watch()
    pipeline_bus.connect('message::error', on_error)
    stopping = []

    def stop():
        stopping.append(True)
        loop.quit()
        return GLib.SOURCE_REMOVE

    GLib.unix_signal_add(GLib.PRIORITY_HIGH, signal.SIGTERM, stop)
    GLib.unix_signal_add(GLib.PRIORITY_HIGH, signal.SIGINT, stop)
    pipeline.set_state(Gst.State.PLAYING)
    loop.run()

    pipeline.set_state(Gst.State.NULL)
    call(session, 'org.gnome.Mutter.ScreenCast.Session', 'Stop', None, '()')
    frames.close()
    summary = dict(
        consumer='gstreamer pipewiresrc ! appsink',
        gstreamer=Gst.version_string(),
        pipewiresrc=f'{factory.get_plugin_name()} {factory.get_plugin().get_version()}',
        connector=connector, node=node[0], frames=len(times),
        first_mono_ns=times[0] if times else None, last_mono_ns=times[-1] if times else None,
        stopped_by_signal=bool(stopping),
    )
    with open(summary_path, 'w') as stream_out:
        json.dump(summary, stream_out, indent=2)
        stream_out.write('\n')
    if not stopping:
        sys.exit('screencast-consumer: the pipeline stopped before a stop signal')
    if not times:
        sys.exit('screencast-consumer: no frame received')


if __name__ == '__main__':
    main()
