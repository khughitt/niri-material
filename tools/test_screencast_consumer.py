# tools/test_screencast_consumer.py
"""Sampling predicates plus a private-bus GStreamer consumer check."""

import json
import os
import shutil
import signal
import subprocess
import sys
import tempfile
import time
import unittest
from unittest import mock
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

    def test_waits_for_changed_client_pixels_despite_other_damage(self):
        sampler = Sampler()
        sampler.request(self.target, 1_000)
        sampler.offer(b'aaabbbcccdddeeefff', 3, 2, 1_001)
        target = self.target.with_name('sample-2.raw')
        sampler.request(target, 2_000, different_from=self.target, region=[1, 0, 1, 1])
        # The other window changes, but the requested client crop is still old.
        self.assertFalse(sampler.offer(b'XXXbbbcccdddeeefff', 3, 2, 2_001))
        self.assertFalse(target.exists())
        self.assertFalse(Path(f'{target}.json').exists())
        self.assertTrue(sampler.offer(b'XXXNEWcccdddeeefff', 3, 2, 2_002))
        self.assertEqual(target.read_bytes(), b'XXXNEWcccdddeeefff')
        self.assertEqual(json.loads(Path(f'{target}.json').read_text())['frame_mono_ns'], 2_002)
        self.assertEqual(json.loads(Path(f'{target}.json').read_text())['rejected_frame_mono_ns'], [2_001])
        self.assertIsNone(sampler.pending)

    def test_invalid_reference_cannot_arm_a_request(self):
        sampler = Sampler()
        sampler.request(self.target, 1)
        sampler.offer(b'aaabbbcccdddeeefff', 3, 2, 2)
        target = self.target.with_name('sample-2.raw')
        for region in ([0, 0, 0, 1], [2, 1, 2, 1], [-1, 0, 1, 1], [0, 0, 1], [True, 0, 1, 1]):
            with self.subTest(region=region), self.assertRaises(ValueError):
                sampler.request(target, 3, different_from=self.target, region=region)
            self.assertFalse(Path(f'{target}.armed').exists())
            self.assertIsNone(sampler.pending)

    def test_changed_frame_dimensions_cannot_satisfy_a_crop_request(self):
        sampler = Sampler()
        sampler.request(self.target, 1)
        sampler.offer(b'aaabbb', 2, 1, 2)
        target = self.target.with_name('sample-2.raw')
        sampler.request(target, 3, different_from=self.target, region=[0, 0, 1, 1])
        with self.assertRaises(ValueError):
            sampler.offer(b'new', 1, 1, 4)
        self.assertFalse(Path(f'{target}.json').exists())

    def test_a_failed_armed_write_leaves_nothing_pending(self):
        sampler = Sampler()
        with self.assertRaises(OSError):
            sampler.request(self.target.parent / 'missing' / 'sample.raw', 1)
        self.assertIsNone(sampler.pending)

    def test_samples_are_written_atomically_raw_first_json_last(self):
        replaced = []
        real_replace = os.replace
        def record(source, destination):
            self.assertNotEqual(Path(source), Path(destination))   # via a temp file
            self.assertEqual(Path(source).parent, Path(destination).parent)
            replaced.append(Path(destination).name)
            real_replace(source, destination)
        sampler = Sampler()
        sampler.request(self.target, 1)
        with mock.patch('os.replace', record):
            sampler.offer(b'new', 1, 1, 2)
        self.assertEqual(replaced, ['sample-1.raw', 'sample-1.raw.json'])
        self.assertEqual(sorted(p.name for p in self.target.parent.iterdir()),
                         ['sample-1.raw', 'sample-1.raw.armed', 'sample-1.raw.json'])

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


TEST_PIPELINE = ('videotestsrc is-live=true ! video/x-raw,width=6,height=4 '
                 '! videoconvert ! video/x-raw,format=RGB ! appsink name=sink emit-signals=true sync=false')
CAN_RUN = (shutil.which('dbus-daemon') and shutil.which('gst-launch-1.0') and consumer_bindings())


@unittest.skipUnless(CAN_RUN, 'needs dbus-daemon, GStreamer and PyGObject with Gio, Gst and GstVideo')
class ConsumerHarness(unittest.TestCase):
    """A private bus, the fake ScreenCast service (ready once it owns its bus
    name) and a way to run the consumer against them. Every pipe is closed."""

    def setUp(self):
        self.tmp = Path(tempfile.mkdtemp())
        self.addCleanup(shutil.rmtree, self.tmp)
        bus = subprocess.Popen(['dbus-daemon', '--session', '--nofork', f'--address=unix:path={self.tmp}/bus'])
        self.addCleanup(lambda: (bus.kill(), bus.wait()))
        for _ in range(50):
            if (self.tmp / 'bus').exists(): break
            time.sleep(0.1)
        self.env = dict(os.environ, DBUS_SESSION_BUS_ADDRESS=f'unix:path={self.tmp}/bus',
                        SCREENCAST_CONSUMER_PIPELINE=TEST_PIPELINE)
        self.start_service()

    def start_service(self, *args):
        self.service = subprocess.Popen([sys.executable, str(ROOT / 'tools/fake_screencast.py'), '7', *args],
                                        env=self.env, stdout=subprocess.PIPE, text=True)
        self.addCleanup(self.stop_service)
        self.assertEqual(self.service.stdout.readline().strip(), 'owned')

    def stop_service(self):
        self.service.kill()
        self.service.wait()
        if not self.service.stdout.closed:
            self.service.stdout.close()

    def service_calls(self):
        """The methods the fake service saw, once it is stopped."""
        self.service.kill()
        calls, _ = self.service.communicate()
        return calls.split()

    def start_consumer(self, request=None, pipeline=None, launcher=None):
        env = dict(self.env, **({'SCREENCAST_CONSUMER_PIPELINE': pipeline} if pipeline else {}))
        consumer = subprocess.Popen([sys.executable, str(launcher or ROOT / 'tools/screencast_consumer.py'), 'DP-1',
                                     str(self.tmp / 'frames'), str(self.tmp / 'summary.json'),
                                     str(request or self.tmp / 'request')],
                                    env=env, stdout=subprocess.PIPE, stderr=subprocess.PIPE, text=True)
        def close():
            if consumer.poll() is None:
                consumer.kill()
            consumer.wait()
            consumer.stdout.close()
            consumer.stderr.close()
        self.addCleanup(close)
        return consumer


class ConsumerEndToEndTests(ConsumerHarness):
    def test_reference_loading_cannot_lose_journaled_arrivals(self):
        reference = self.tmp / 'sample-1.raw'
        reference.write_bytes(bytes([255]) * (6 * 4 * 3))
        Path(f'{reference}.json').write_text(json.dumps({'width': 6, 'height': 4}))
        target = self.tmp / 'sample-2.raw'
        (self.tmp / 'request').write_text(json.dumps(
            {'path': str(target), 'different_from': str(reference), 'region': [1, 1, 2, 2]}))
        # Run the real consumer/source, delaying only the reference's file I/O.
        launcher = self.tmp / 'delayed_consumer.py'
        launcher.write_text(
            'import runpy, time\nfrom pathlib import Path\n'
            'read = Path.read_bytes\n'
            'def delayed(path):\n'
            '    data = read(path)\n'
            '    time.sleep(0.3)\n'
            '    return data\n'
            'Path.read_bytes = delayed\n'
            f'runpy.run_path({str(ROOT / "tools/screencast_consumer.py")!r}, run_name="__main__")\n')
        consumer = self.start_consumer(launcher=launcher,
                                      pipeline=TEST_PIPELINE.replace('videotestsrc ', 'videotestsrc pattern=black '))
        self.assertEqual(consumer.stdout.readline().strip(), 'ready 7')
        consumer.send_signal(signal.SIGUSR1)
        for _ in range(50):
            if Path(f'{target}.json').exists():
                break
            time.sleep(0.1)
        consumer.send_signal(signal.SIGTERM)
        _, stderr = consumer.communicate(timeout=10)
        self.assertEqual(consumer.returncode, 0, stderr)
        sample = json.loads(Path(f'{target}.json').read_text())
        frames = [int(line) for line in (self.tmp / 'frames').read_text().splitlines()]
        between = [t for t in frames if sample['request_mono_ns'] < t < sample['frame_mono_ns']]
        self.assertEqual(sample['rejected_frame_mono_ns'], between)

    def test_unchanged_stream_cannot_complete_a_crop_request(self):
        target = self.tmp / 'sample-2.raw'
        reference = self.tmp / 'sample-1.raw'
        reference.write_bytes(bytes(6 * 4 * 3))
        Path(f'{reference}.json').write_text(json.dumps({'width': 6, 'height': 4}))
        request = self.tmp / 'request'
        request.write_text(json.dumps({'path': str(target), 'different_from': str(reference),
                                       'region': [1, 1, 2, 2]}))
        consumer = self.start_consumer(pipeline=TEST_PIPELINE.replace('videotestsrc ', 'videotestsrc pattern=black '))
        self.assertEqual(consumer.stdout.readline().strip(), 'ready 7')
        consumer.send_signal(signal.SIGUSR1)
        for _ in range(50):
            if Path(f'{target}.armed').exists() and len((self.tmp / 'frames').read_text().splitlines()) >= 3:
                break
            time.sleep(0.1)
        self.assertTrue(Path(f'{target}.armed').exists())
        self.assertGreaterEqual(len((self.tmp / 'frames').read_text().splitlines()), 3)
        self.assertFalse(Path(f'{target}.json').exists())
        self.assertFalse(target.exists())
        consumer.send_signal(signal.SIGTERM)
        _, stderr = consumer.communicate(timeout=10)
        self.assertEqual(consumer.returncode, 0, stderr)
        self.assertTrue(json.loads((self.tmp / 'summary.json').read_text())['stopped_by_signal'])

    def test_startup_sampling_and_summary(self):
        tmp = self.tmp
        request = tmp / 'request'; target = tmp / 'sample-1.raw'
        consumer = self.start_consumer()
        self.assertEqual(consumer.stdout.readline().strip(), 'ready 7')
        request.write_text(json.dumps({'path': str(target)}))
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
        self.assertEqual(self.service_calls()[-1], 'Stop')


class ConsumerFailureTests(ConsumerHarness):
    def test_unreadable_request_file_exits_nonzero(self):
        consumer = self.start_consumer(request=self.tmp / 'missing')
        self.assertEqual(consumer.stdout.readline().strip(), 'ready 7')
        consumer.send_signal(signal.SIGUSR1)
        _, stderr = consumer.communicate(timeout=10)
        self.assertNotEqual(consumer.returncode, 0)
        self.assertIn('screencast-consumer:', stderr)
        self.assertIn('missing', stderr)
        self.assertEqual(self.service_calls()[-1], 'Stop')

    def test_a_pipeline_that_fails_after_playing_exits_nonzero_with_gstreamers_error(self):
        # identity errors out of the running pipeline after three buffers.
        consumer = self.start_consumer(pipeline=(
            'videotestsrc is-live=true ! identity error-after=3 ! appsink name=sink emit-signals=true sync=false'))
        self.assertEqual(consumer.stdout.readline().strip(), 'ready 7')
        _, stderr = consumer.communicate(timeout=10)
        self.assertNotEqual(consumer.returncode, 0)
        self.assertEqual(stderr.count('screencast-consumer:'), 1, stderr)
        self.assertIn('identity', stderr.lower())
        self.assertEqual(self.service_calls()[-1], 'Stop')

    def test_a_pipeline_that_cannot_start_exits_nonzero_with_gstreamers_error(self):
        consumer = self.start_consumer(pipeline='filesrc location=/nonexistent/frames ! appsink name=sink')
        _, stderr = consumer.communicate(timeout=10)
        self.assertNotEqual(consumer.returncode, 0)
        self.assertIn('screencast-consumer:', stderr)
        self.assertIn('/nonexistent/frames', stderr)
        self.assertEqual(self.service_calls()[-1], 'Stop')

    def test_a_pipeline_that_cannot_be_built_stops_the_session_and_exits_nonzero(self):
        consumer = self.start_consumer(pipeline='noSuchElement ! appsink name=sink')
        _, stderr = consumer.communicate(timeout=10)
        self.assertNotEqual(consumer.returncode, 0)
        self.assertIn('screencast-consumer:', stderr)
        self.assertNotIn('Traceback', stderr)
        self.assertEqual(self.service_calls()[-1], 'Stop')


    def test_term_while_waiting_for_the_node_still_stops_the_session(self):
        # The service never announces a stream: TERM lands in the node wait,
        # before the GLib signal sources exist.
        self.stop_service()
        self.start_service('silent')
        consumer = self.start_consumer()
        for line in self.service.stdout:
            if line.strip() == 'Start':
                break
        consumer.send_signal(signal.SIGTERM)
        _, stderr = consumer.communicate(timeout=10)
        self.assertNotEqual(consumer.returncode, 0)
        self.assertNotEqual(consumer.returncode, -signal.SIGTERM, stderr)   # not the default action
        self.assertIn('screencast-consumer: stopped by SIGTERM before', stderr)
        self.assertFalse((self.tmp / 'summary.json').exists())
        self.assertEqual(self.service_calls()[-1], 'Stop')


class PackedRgbTests(unittest.TestCase):
    def test_strips_row_padding(self):
        # 2x2 RGB, stride 8: two pad bytes per row.
        data = bytes([1, 2, 3, 4, 5, 6, 0, 0, 7, 8, 9, 10, 11, 12, 0, 0])
        self.assertEqual(packed_rgb(data, 2, 2, 8), bytes(range(1, 13)))

    def test_short_buffer_raises(self):
        # The last row needs only its 6 pixel bytes: 8 + 6 = 14 bytes suffice, 13 do not.
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
