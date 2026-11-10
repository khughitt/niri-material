"""Tracy CSV zone reader shared by view-tilt-cost.sh's report heredocs."""
import csv


def zones(path, t, d):
    with path.open() as f:
        return [(next(iter(r.values())), float(r[t]), float(r[d])) for r in csv.DictReader(f)]
