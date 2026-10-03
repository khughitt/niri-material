# tools/test_screencast_consumer.py
"""The consumer's sampling logic without GStreamer: a sample is the first
frame after an armed request, tightly packed (spec 2026-10-02 real-TTY §4)."""

import json
import os
import shutil
import signal
import subprocess
import sys
import tempfile
import time
import unittest
from pathlib import Path

from tools.screencast_consumer import PIPELINE, Sampler, packed_rgb, wait_for

ROOT = Path(__file__).resolve().parents[1]


class SamplerTests(unittest.TestCase):
    def setUp(self):
        temporary = tempfile.TemporaryDirectory()
        self.addCleanup(temporary.cleanup)
        self.target = Path(temporary.name) / 'sample-1.raw'

    def test_saves_the_first_frame_after_the_request(self):
        sampler = Sampler()
        sampler.request(self.target, 1_000)
        self.assertEqual(Path(f'{self.target}.armed').read_text(), '1000\n')
        self.assertFalse(sampler.offer(b'old', 1, 1, 1_000))       # not after the request
        self.assertFalse(self.target.exists())
        self.assertTrue(sampler.offer(b'new', 1, 1, 1_001))
        self.assertEqual(self.target.read_bytes(), b'new')
        self.assertEqual(json.loads(Path(f'{self.target}.json').read_text()),
                         {'request_mono_ns': 1_000, 'frame_mono_ns': 1_001, 'width': 1, 'height': 1})
        self.assertIsNone(sampler.pending)
        self.assertFalse(sampler.offer(b'later', 1, 1, 1_002))     # one frame per request

    def test_refuses_a_second_request_while_one_is_pending(self):
        sampler = Sampler()
        sampler.request(self.target, 1)
        with self.assertRaises(RuntimeError):
            sampler.request(self.target.with_name('sample-2.raw'), 2)

    def test_nothing_is_saved_without_a_request(self):
        self.assertFalse(Sampler().offer(b'frame', 1, 1, 5))


try:
    import gi
    gi.require_version('GLib', '2.0')
    from gi.repository import GLib
except (ImportError, ValueError):
    GLib = None


@unittest.skipIf(GLib is None, 'PyGObject is not installed')
class WaitForTests(unittest.TestCase):
    def test_the_discovery_deadline_cannot_stop_consumption(self):
        def subscribe(deliver):
            GLib.timeout_add(50, lambda: deliver(42) or GLib.SOURCE_REMOVE)
            return lambda: None

        self.assertEqual(wait_for(subscribe, 0.3), 42)
        # The consumption loop outlives the discovery deadline (0.3 s).
        loop = GLib.MainLoop()
        GLib.timeout_add(700, lambda: loop.quit() or GLib.SOURCE_REMOVE)
        started = time.monotonic()
        loop.run()
        self.assertGreater(time.monotonic() - started, 0.65)

    def test_times_out_without_a_node(self):
        started = time.monotonic()
        self.assertIsNone(wait_for(lambda deliver: (lambda: None), 0.2))
        self.assertLess(time.monotonic() - started, 1)


class PipelineTests(unittest.TestCase):
    def test_imports_dma_bufs_through_gl(self):
        description = PIPELINE.format(node=7)
        self.assertIn('pipewiresrc path=7 ! video/x-raw(memory:DMABuf),format=DMA_DRM ! glupload', description)
        self.assertIn('gldownload ! videoconvert ! video/x-raw,format=RGB ! appsink', description)


# Startup through summary on a private bus, with a fake ScreenCast service and
# a test source in place of PipeWire. 6x3-byte rows are padded to 20 bytes by
# GStreamer, so the sample's size also proves packing.
def consumer_bindings():
    """PyGObject with the introspection data the consumer and fake service load."""
    try:
        import gi
        for namespace, version in (('Gio', '2.0'), ('GLib', '2.0'), ('Gst', '1.0'), ('GstVideo', '1.0')):
            gi.require_version(namespace, version)
        from gi.repository import Gio, GLib, Gst, GstVideo  # noqa: F401
    except (ImportError, ValueError):
        return False
    return True


@unittest.skipUnless(shutil.which('dbus-daemon') and shutil.which('gst-launch-1.0') and consumer_bindings(),
                     'needs dbus-daemon, GStreamer and PyGObject with Gio, Gst and GstVideo')
class ConsumerEndToEndTests(unittest.TestCase):
    def test_startup_sampling_and_summary(self):
        tmp = Path(tempfile.mkdtemp()); self.addCleanup(shutil.rmtree, tmp)
        bus = subprocess.Popen(['dbus-daemon', '--session', '--nofork', f'--address=unix:path={tmp}/bus'])
        self.addCleanup(lambda: (bus.kill(), bus.wait()))
        for _ in range(50):
            if (tmp / 'bus').exists(): break
            time.sleep(0.1)
        env = dict(os.environ, DBUS_SESSION_BUS_ADDRESS=f'unix:path={tmp}/bus',
                   SCREENCAST_CONSUMER_PIPELINE='videotestsrc is-live=true ! video/x-raw,width=6,height=4 '
                   '! videoconvert ! video/x-raw,format=RGB ! appsink name=sink emit-signals=true sync=false')
        service = subprocess.Popen([sys.executable, str(ROOT / 'tools/fake_screencast.py'), '7'], env=env,
                                   stdout=subprocess.PIPE, text=True)
        self.addCleanup(lambda: (service.kill(), service.wait()))
        time.sleep(0.5)
        request = tmp / 'request'; target = tmp / 'sample-1.raw'
        consumer = subprocess.Popen([sys.executable, str(ROOT / 'tools/screencast_consumer.py'), 'DP-1',
                                     str(tmp / 'frames'), str(tmp / 'summary.json'), str(request)],
                                    env=env, stdout=subprocess.PIPE, stderr=subprocess.PIPE, text=True)
        self.addCleanup(lambda: consumer.poll() is None and consumer.kill())
        self.assertEqual(consumer.stdout.readline().strip(), 'ready 7')
        request.write_text(f'{target}\n')
        consumer.send_signal(signal.SIGUSR1)
        for _ in range(50):
            if Path(f'{target}.json').exists(): break
            time.sleep(0.1)
        consumer.send_signal(signal.SIGTERM)
        _, stderr = consumer.communicate(timeout=10)
        self.assertEqual(consumer.returncode, 0, stderr)
        summary = json.loads((tmp / 'summary.json').read_text())
        self.assertEqual((summary['node'], summary['connector'], summary['stopped_by_signal']), (7, 'DP-1', True))
        self.assertGreater(summary['frames'], 0)
        sample = json.loads(Path(f'{target}.json').read_text())
        self.assertEqual((sample['width'], sample['height']), (6, 4))
        self.assertEqual(len(target.read_bytes()), 6 * 4 * 3)
        self.assertLess(sample['request_mono_ns'], sample['frame_mono_ns'])


@unittest.skipUnless(shutil.which('dbus-daemon') and shutil.which('gst-launch-1.0') and consumer_bindings(),
                     'needs dbus-daemon, GStreamer and PyGObject with Gio, Gst and GstVideo')
class ConsumerFailureTests(unittest.TestCase):
    def test_unreadable_request_file_exits_nonzero(self):
        tmp = Path(tempfile.mkdtemp()); self.addCleanup(shutil.rmtree, tmp)
        bus = subprocess.Popen(['dbus-daemon', '--session', '--nofork', f'--address=unix:path={tmp}/bus'])
        self.addCleanup(lambda: (bus.kill(), bus.wait()))
        for _ in range(50):
            if (tmp / 'bus').exists(): break
            time.sleep(0.1)
        env = dict(os.environ, DBUS_SESSION_BUS_ADDRESS=f'unix:path={tmp}/bus',
                   SCREENCAST_CONSUMER_PIPELINE='videotestsrc is-live=true ! video/x-raw,width=6,height=4 '
                   '! videoconvert ! video/x-raw,format=RGB ! appsink name=sink emit-signals=true sync=false')
        service = subprocess.Popen([sys.executable, str(ROOT / 'tools/fake_screencast.py'), '7'], env=env,
                                   stdout=subprocess.PIPE, text=True)
        self.addCleanup(lambda: (service.kill(), service.wait()))
        time.sleep(0.5)
        consumer = subprocess.Popen([sys.executable, str(ROOT / 'tools/screencast_consumer.py'), 'DP-1',
                                     str(tmp / 'frames'), str(tmp / 'summary.json'), str(tmp / 'missing')],
                                    env=env, stdout=subprocess.PIPE, stderr=subprocess.PIPE, text=True)
        self.addCleanup(lambda: consumer.poll() is None and consumer.kill())
        self.assertEqual(consumer.stdout.readline().strip(), 'ready 7')
        consumer.send_signal(signal.SIGUSR1)
        _, stderr = consumer.communicate(timeout=10)
        self.assertNotEqual(consumer.returncode, 0)
        self.assertIn('screencast-consumer:', stderr)
        self.assertIn('missing', stderr)


class PackedRgbTests(unittest.TestCase):
    def test_strips_row_padding(self):
        # 2x2 RGB, stride 8: two pad bytes per row.
        data = bytes([1, 2, 3, 4, 5, 6, 0, 0, 7, 8, 9, 10, 11, 12, 0, 0])
        self.assertEqual(packed_rgb(data, 2, 2, 8), bytes(range(1, 13)))

    def test_short_buffer_raises(self):
        # Last row needs only its 6 pixel bytes, so 14 is short and 14+... below is too.
        with self.assertRaises(ValueError):
            packed_rgb(bytes(13), 2, 2, 8)
        with self.assertRaises(ValueError):
            packed_rgb(bytes(11), 2, 2, 6)
        self.assertEqual(len(packed_rgb(bytes(14), 2, 2, 8)), 12)

    def test_packed_input_is_unchanged(self):
        data = bytes(range(12))
        self.assertEqual(packed_rgb(data, 2, 2, 6), data)


if __name__ == '__main__':
    unittest.main()
