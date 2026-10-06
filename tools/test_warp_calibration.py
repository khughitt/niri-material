"""The warp calibration's estimator on a reduced ROI: known warps come back,
photometric-only changes do not read as displacement, and a window that
straddles a warp discontinuity is flagged rather than trusted."""
import importlib.util
from pathlib import Path
import tempfile
import unittest

WIN = 7


def load():
    path = (Path(__file__).resolve().parents[1] / 'docs/materials/scripts'
            / 'warp-calibration.py')
    spec = importlib.util.spec_from_file_location('warp_calibration', path)
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    # Small enough for the fast tooling route; the chamfer strip keeps its
    # 12 px, so the 7 px window the evidence adopts still fits inside it.
    module.ROI_W, module.ROI_H = 44, 24
    module.STRIP_X, module.STRIP_W = 16, 12
    module.SEARCH_X, module.SEARCH_Y = 6, 2
    module.WINDOWS = (WIN,)
    return module


class WarpCalibrationTest(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        cls.m = load()
        cls.field = cls.m.margin_field(lambda x, y: cls.m.aperiodic_code(cls.m.SEED, x, y))
        cls.grads = cls.field.gradients()

    def run_case(self, warp, photo):
        m = self.m
        out = m.render(self.field, warp, photo)
        est = m.estimate(self.field, out, WIN)
        return m.summarize(m.refine(self.field, self.grads, out, est, WIN), warp, WIN // 2)

    def test_rigid_subpixel_shift_is_recovered(self):
        s = self.run_case(self.m.warp_rigid(2.4, -0.6), self.m.photo_none)['all']
        # Bilinear resampling low-passes the shifted texture, which no
        # translation undoes: a few windows keep up to ~0.15 px.
        self.assertGreater(s['confident'], 0.9)
        self.assertLess(s['median'], 0.02)
        self.assertLess(s['max'], 0.2)
        self.assertEqual(s['wrong_confident'], 0)

    def test_photometric_change_is_not_displacement(self):
        s = self.run_case(self.m.warp_none, self.m.photo_ramp)['all']
        self.assertGreater(s['confident'], 0.9)
        self.assertLess(s['median'], 0.02)
        self.assertLess(s['max'], 0.2)

    def test_chamfer_strip_is_measured_and_its_boundary_flagged(self):
        s = self.run_case(self.m.warp_chamfer(3.0), self.m.photo_none)
        self.assertEqual(s['strip']['confident'], 1.0)
        self.assertLess(s['strip']['max'], 0.05)
        for region in s.values():
            self.assertEqual(region['wrong_confident'], 0)
        self.assertLess(s['boundary']['confident'], 0.5)

    def test_flat_field_removes_the_chamfer_tint_and_glint(self):
        m = self.m
        rows = m.render(self.field, m.warp_none, m.photo_chamfer)
        fixed = m.correct(rows, m.flat_field(m.photo_chamfer))
        x0, x1 = m.STRIP_X, m.STRIP_X + m.STRIP_W
        self.assertGreater(m.rmse_code(self.field, rows, x0, x1), 10)
        self.assertLess(m.rmse_code(self.field, fixed, x0, x1), 4)

    def test_field_is_a_function_of_absolute_position(self):
        m = self.m
        a = m.Field.of(lambda x, y: m.aperiodic_code(7, x, y), 100, 50, 8, 4)
        b = m.Field.of(lambda x, y: m.aperiodic_code(7, x, y), 102, 51, 8, 4)
        self.assertEqual(a.rows[1][2:], b.rows[0][:6])

    def test_backdrop_png_carries_its_provenance(self):
        with tempfile.TemporaryDirectory() as tmp:
            path = Path(tmp) / 'b.png'
            self.m.backdrop(path, 7, w=8, h=4)
            data = path.read_bytes()
        self.assertTrue(data.startswith(b'\x89PNG\r\n\x1a\n'))
        self.assertIn(b'backdrop aperiodic-s7', data)


if __name__ == '__main__':
    unittest.main()
