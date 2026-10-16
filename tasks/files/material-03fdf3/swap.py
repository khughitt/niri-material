import sys, re
src, noise, out = sys.argv[1:4]
s = open(src).read()
start = s.index("// ---- optic: noise\n") + len("// ---- optic: noise\n")
end = s.index("\n// ---- ", start)
open(out, "w").write(s[:start] + open(noise).read() + s[end:])
