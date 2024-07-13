---
name: Performance Issue
about: CrabFetch's performance is not as you expected.
title: ''
labels: performance
assignees: ''

---

**Screenshot of your CrabFetch being slow**
Use the "hyperfine" utility to time it;
```bash
hyperfine "crabfetch" -N --warmup=50
```

**Output of Benchmark**
```bash
crabfetch --benchmark
```

**Expected Performance**
What were you expecting, screenshot another output if needed.


**System Info**
 - Distro / Kernel Version: 
 - Terminal:
 - Desktop Environment:

**Additional Comments**
Add any other context about the problem here.
