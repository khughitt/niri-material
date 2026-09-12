---
id: material-7f7aa3
title: Verify capture preflight busy-host refusal after NVML repair
status: done
priority: 2
size: xs
owner: material-bae9c9
created: 2026-09-12T04:56:56Z
updated: 2026-09-12T09:53:54Z
started: 2026-09-12T04:57:44Z
completed: 2026-09-12T09:53:54Z
depends: []
parent: material-bae9c9
tags: [performance, testing]
---

After the host NVIDIA driver/library mismatch is repaired, run one real headless capture-meta preflight while the desktop is naturally GPU-busy. Record exit 1, the failing quietness reason, and two or three graphics client names; release and remove only the disposable run. Do not manufacture load or perform a hardware capture.

## Notes

- 2026-09-12T04:57:03Z (material-bae9c9): parked (waiting on user, environment): After NVML works again, run the real headless preflight during naturally busy GPU use and record exit 1, threshold reason, and graphics clients.
- 2026-09-12T04:57:44Z (material-bae9c9): parked (waiting on user, environment): After NVML works again, run the real headless preflight during naturally busy GPU use and record exit 1, threshold reason, and graphics clients.
- 2026-09-12T04:58:41Z (material-bae9c9): Environment diagnosis: loaded NVIDIA kernel module is 610.57.04 while nvidia-smi loads NVML library 615.71; repair is outside scope.
- 2026-09-12T09:53:54Z (material-bae9c9): After reboot, real headless preflight on 2026-09-12T05:53:30-04:00 with NVIDIA 615.71.09 sampled 5 seconds and exited 1: load1 5.95 > 2.0; GPU utilization 8.0% > 5.0%; P0/P5 instead of P8; GPU power IQR 26.21 W > 1.0 W. Graphics clients included niri, noctalia, kitty (also Xwayland and qs); no compute clients. show and release exited 0; disposable run removed. No manufactured load or compositor capture.
- 2026-09-12T09:53:54Z (material-bae9c9): Verified real busy-host refusal after reboot; reasons and graphics clients recorded, lock released, disposable run removed
