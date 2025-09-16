---

name: Bug report
about: File a bug to help us improve ResoBoost
title: "\[BUG] "
labels: bug, needs-triage
assignees: ""
-------------

<!--
Thanks for taking the time to report a bug! A clear report helps us fix things faster.
Please search existing issues/discussions before filing a new one.
Do NOT include secrets, tokens, or personal data in your report.
For security vulnerabilities, DO NOT open an issue — use the Security tab's private report
or email ednoct@proton.me (optionally with your PGP).
-->

### Checklist

* [ ] I have searched **existing issues** and **discussions** for duplicates.
* [ ] I have read and agree to follow the **Code of Conduct**.
* [ ] This report is **not** a security vulnerability (if it is, see “Security” above).

---

### Summary

A clear and concise description of the problem.

### Environment

* **ResoBoost version/commit**: (e.g., v1.2.3 or `abcdef1`)
* **Install method**: (source | pip | container | other)
* **OS**: (Linux/macOS/Windows + version)
* **CPU/GPU**: (e.g., AMD Ryzen 7 / NVIDIA RTX 4070)
* **Python**: (e.g., 3.11.x)
* **Key deps**: (e.g., torch/cuda versions, driver/runtime details if applicable)

### Steps to Reproduce

1.
2.
3.

### Minimal Reproduction (code/commands)

```bash
# the smallest set of commands that reproduces the bug
```

```python
# if Python code is involved, include a minimal runnable snippet
```

### Expected Behavior

What you expected to happen.

### Actual Behavior

What actually happened (include full error text where possible).

<details>
<summary>Logs / Stack Traces</summary>

```
# paste logs here (trim to the relevant section if large)
```

</details>

### Frequency & Scope

* **Frequency**: (always | often | intermittent)
* **Scope**: (single user | many users | CI only | specific platform)

### Regression?

* Did this work in a previous version? If so, which:

### Related Issues/PRs

* (link to similar reports or PRs)

### Workarounds Tried

* (list any mitigations you found)

### Screenshots

* (if helpful)

### Additional Context

* Any other context that might be helpful for triage.

---

### Contributor Details (optional)

* **Contact email (optional)**: (we may reach out if we need more info)
* **Willing to submit a PR?** (Yes/No/Maybe with guidance)
* **Area**: \[ ] Core  \[ ] CLI  \[ ] API  \[ ] Docs  \[ ] Build/CI  \[ ] Packaging  \[ ] Performance

<!--
Triage notes (maintainers):
- Confirm repro; label with area + severity.
- Link advisory if the issue has security implications.
- If repro requires private data, request a sanitised example.
-->
