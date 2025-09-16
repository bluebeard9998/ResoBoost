# Security Policy

> We build carefully, we disclose responsibly, and we fix with purpose.

This document explains how to report security issues in **ResoBoost**, how we triage and remediate, and how we publish advisories for the community.

---

## Supported Versions

We currently maintain the latest stable release line and the `main` branch. Update this table as you tag releases.

| Version            | Supported |
| :----------------- | :-------: |
| `main`             |     ✅     |
| `v1.x`             |     ✅     |
| `<older releases>` |     ❌     |

> **Note:** If you publish new release lines (e.g., `v2.x`), add them here. Older, end‑of‑life lines are not supported except for critical fixes at the maintainers’ discretion.

---

## Reporting a Vulnerability

Please **do not file public issues** for security reports. Use one of the private channels below:

1. **GitHub – Private Vulnerability Report (preferred)**
   If enabled for this repo, open a private report from the repository’s **Security** tab → **“Report a vulnerability to the maintainers.”**

2. **Email**
   Send details to ***\[REPLACE WITH SECURITY EMAIL]*** (optionally encrypt with our PGP key: ***\[REPLACE WITH PGP FINGERPRINT / LINK]***).

3. **If the issue affects a third‑party dependency** only, please report it upstream and let us know privately so we can track it.

### What to include

* A clear description of the issue and **impact**
* **Steps to reproduce** (PoC), affected version/commit, and environment
* Minimal logs, screenshots, or crash output
* Suggested fix or workaround, if available

Please avoid excessive data exfiltration and **do not** include sensitive personal data. If exploitation requires credentials, use your own test accounts only.

---

## Triage & Response SLAs

We aim to be prompt and proportionate:

* **Acknowledgement:** within **72 hours**
* **Initial triage & severity:** within **7 days**
* **Fix or advisory:** target **≤ 90 days** from acknowledgement (faster for critical issues or active exploitation)

These targets may vary depending on complexity and scope. We will keep you updated during remediation.

### Severity

We classify severity using a CVSS‑style assessment (e.g., critical/high/medium/low) considering exploitability, impact, and affected deployment scenarios.

---

## Coordinated Disclosure

We practice **coordinated vulnerability disclosure** (CVD):

* We collaborate privately with reporters to **verify**, **fix**, and **prepare an advisory**.
* We may request a reasonable **embargo** while a fix is prepared and widely available.
* We will publish a **Repository Security Advisory** (and request a **CVE ID** when appropriate) once a fix, mitigation, or rationale is ready; or earlier if there is evidence of active exploitation.

We prefer not to publicly disclose exploit details until users can reasonably update.

---

## Publishing Advisories

When an issue is confirmed, we will:

1. Create a **GitHub Security Advisory** in this repository.
2. Assign a **CVE ID** (if applicable) and publish patches, workarounds, and upgrade guidance.
3. Credit reporters (opt‑in) in the advisory and release notes.

---

## Scope

Vulnerabilities in **ResoBoost’s** source code, build and release artifacts, and GitHub automation are in scope. The following **are generally out of scope** unless they demonstrate clear security impact within ResoBoost:

* Issues in third‑party dependencies without a ResoBoost‑specific exploit path
* Social engineering, physical attacks, or account‑takeover scenarios unrelated to project code/config
* Denial of service via excessive resource consumption using standard tools (unless it leads to persistent impact)
* Missing best‑practice headers on non‑production demo content
* Findings that require unrealistic privileges or non‑default configurations

If you’re unsure about scope, report privately—we’ll help assess.

---

## Safe Harbor (Good‑Faith Research)

We welcome good‑faith security research. We will not pursue or support legal action against researchers for:

* Testing **within the scope** above, in a way that avoids privacy violations or service degradation
* Reporting vulnerabilities **promptly and privately**
* Making a good‑faith effort to avoid accessing or destroying data beyond what’s necessary to demonstrate the vulnerability

This safe‑harbor statement does not waive the rights of project contributors or users and does not permit unlawful behavior. When in doubt, contact us first.

---

## Hall of Fame (Optional)

With your permission, we credit reporters in a `SECURITY-ACKNOWLEDGEMENTS.md` (or the advisory) after remediation. Please include the name/handle you’d like us to use.

---

## Development Notes (for Maintainers)

* Keep **Private Vulnerability Reporting** enabled in the repository settings when possible.
* Use **temporary private forks** for coordinated fixes.
* Publish advisories with clear affected versions, patches, and upgrade paths; link commits/releases.

---

**Last updated:** 16 September 2025

**Security contact:** *\[ADD EMAIL / KEY]*
