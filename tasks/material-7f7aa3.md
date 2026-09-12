---
id: material-7f7aa3
title: Verify capture preflight busy-host refusal after NVML repair
status: doing
priority: 2
size: xs
owner: material-bae9c9
created: 2026-09-12T04:56:56Z
updated: 2026-09-12T04:58:41Z
started: 2026-09-12T04:57:44Z
depends: []
parent: material-bae9c9
tags: [performance, testing]
---

After the host NVIDIA driver/library mismatch is repaired, run one real headless capture-meta preflight while the desktop is naturally GPU-busy. Record exit 1, the failing quietness reason, and two or three graphics client names; release and remove only the disposable run. Do not manufacture load or perform a hardware capture.

## Notes

- 2026-09-12T04:57:03Z (material-bae9c9): parked (waiting on user, environment): After NVML works again, run the real headless preflight during naturally busy GPU use and record exit 1, threshold reason, and graphics clients.
- 2026-09-12T04:57:44Z (material-bae9c9): parked (waiting on user, environment): After NVML works again, run the real headless preflight during naturally busy GPU use and record exit 1, threshold reason, and graphics clients.
- 2026-09-12T04:58:41Z (material-bae9c9): Environment diagnosis: loaded NVIDIA kernel module is 610.57.04 while nvidia-smi loads NVML library 615.71; repair is outside scope.
