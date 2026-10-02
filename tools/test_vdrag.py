import importlib.util
import pathlib
import struct
import unittest

SPEC = importlib.util.spec_from_file_location(
    "vdrag", pathlib.Path(__file__).resolve().parent.parent / "docs/materials/scripts/vdrag.py"
)
vdrag = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(vdrag)


class WireTest(unittest.TestCase):
    def test_header_packs_object_size_and_opcode(self):
        msg = vdrag.message(7, 2, vdrag.u32(1), vdrag.u32(0x110), vdrag.u32(1))
        obj, word = struct.unpack("<II", msg[:8])
        self.assertEqual(obj, 7)
        self.assertEqual(word >> 16, len(msg))
        self.assertEqual(word & 0xFFFF, 2)

    def test_fixed_is_24_8(self):
        self.assertEqual(vdrag.fixed(1.5), struct.pack("<i", 384))
        self.assertEqual(vdrag.fixed(-2.0), struct.pack("<i", -512))

    def test_string_is_padded_and_nul_terminated(self):
        s = vdrag.string("wl_seat")
        (n,) = struct.unpack("<I", s[:4])
        self.assertEqual(n, 8)
        self.assertEqual(len(s) % 4, 0)
        self.assertEqual(s[4:12], b"wl_seat\0")

    def test_split_messages_handles_partial_tail(self):
        a = vdrag.message(3, 0, vdrag.u32(5))
        b = vdrag.message(4, 1, vdrag.string("x"))
        msgs, rest = vdrag.split_messages(a + b[:6])
        self.assertEqual([(m[0], m[1]) for m in msgs], [(3, 0)])
        self.assertEqual(rest, b[:6])

    def test_parse_global(self):
        body = vdrag.u32(9) + vdrag.string("wl_seat") + vdrag.u32(7)
        self.assertEqual(vdrag.parse_global(body), (9, "wl_seat", 7))

    def test_stripes_fill_the_buffer_translucent_and_premultiplied(self):
        data = vdrag.stripes(64, 4)
        self.assertEqual(len(data), 64 * 4 * 4)
        for (px,) in struct.iter_unpack("<I", data):
            a = px >> 24
            self.assertLess(a, 255)
            for shift in (0, 8, 16):
                self.assertLessEqual((px >> shift) & 0xFF, a)


if __name__ == "__main__":
    unittest.main()
