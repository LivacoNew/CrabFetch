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

**Relevant Information**
Such as but not limited to Distribution, what you would expect a module to output, etc.

**Config File**
<details>
<summary>Config File</summary>

```toml
Configuration file here please.
```

</details>

**Version**
```
crabfetch -v
```

**Additional Comments**
Any additional comments you may have.
