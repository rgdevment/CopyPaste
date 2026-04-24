# Security Policy

## Security Matters

**CopyPaste** handles your clipboard history—that can include sensitive stuff. I take security seriously because you're trusting this tool with content that might be personal or confidential.

**This isn't corporate security theater.** This is a personal project shared with the community. It's built on trust—transparency in the code, responsibility when issues come up, and treating security researchers as partners.

I'm not protecting a brand or business. I'm protecting _you_ and everyone using this tool.

---

## 🔒 What We Do to Keep You Safe

### Privacy by Design

- **100% Local Storage** — Your clipboard history never leaves your machine. No cloud sync, no telemetry, no remote servers.
- **Sensitive Data Exclusion** — Password manager content (1Password, Bitwarden, etc.) is automatically excluded from history.
- **No Tracking** — I don't collect anything. No analytics, no usage data, nothing.

### Security Features

- **Local SQLite Database** — Your clipboard history is stored in a local database on your machine, not in the cloud.
- **Configurable Retention** — Automatically delete old clipboard items based on your retention settings.
- **Open Source** — Every line of code is public. You can inspect, audit, and verify what we're doing.
- **Signed Release Manifest** — The update notifier fetches a small JSON file signed with an Ed25519 key. The signature is verified locally before the file is trusted, so a compromised mirror cannot inject a fake "latest version" or a malicious install URL. If the signature fails, the manifest is discarded.
- **Minimum Supported Version Enforcement** — When a release contains a critical fix (e.g. a data-corruption or security issue), the signed manifest can mark older versions as blocked. Standalone builds (Windows / macOS / Linux) then show a full-screen prompt with direct install instructions. **Microsoft Store builds are never blocked** — updates on that platform are delivered on Microsoft's review schedule, which is outside our control, so blocking would leave users without a path forward.

### Development Practices

- **Modern Flutter Stack** — Built with Flutter and Dart, with dependencies regularly audited and updated.
- **Dependency Updates** — We regularly update dependencies to patch known vulnerabilities.
- **Code Reviews** — All contributions go through review before merging.

---

## 🚨 Supported Versions

Security updates are provided for:

| Version | Supported |
| :--- | :--- |
| Latest Release | ✅ Actively Supported |
| Beta Versions | ✅ Actively Supported |
| Older Releases | ❌ Not Supported (please update) |

**We strongly recommend always using the latest version** from the [Releases Page](https://github.com/rgdevment/CopyPaste/releases/latest).

---

## 🐛 Reporting a Vulnerability

If you discover a security vulnerability in **CopyPaste**, please help us protect our users by reporting it responsibly.

### What Qualifies as a Security Vulnerability?

**Please report:**

- ✅ Unauthorized access to clipboard history
- ✅ Privilege escalation issues
- ✅ Data leakage or unintended storage of sensitive information
- ✅ Injection attacks (SQL, command, etc.)
- ✅ Bypass of sensitive data exclusion mechanisms
- ✅ Critical bugs that could lead to data loss or corruption

**Not security issues:**

- ❌ Feature requests or enhancements
- ❌ General bugs that don't have security implications
- ❌ Issues with third-party dependencies (report those upstream)
- ❌ Windows SmartScreen warnings (see [README](README.md) for explanation)

### How to Report Securely

**DO NOT** open a public GitHub issue for security vulnerabilities. Instead, use one of these private channels:

#### 📧 Email (Simplest & Direct)

Send an email to: **<github@apirest.cl>**

**Subject:** `[SECURITY] Brief description of the issue`

This is the fastest way to reach us. We check email daily and will respond within 48 hours.

#### 🔒 GitHub Security Advisory (Alternative)

1. Go to the [Security tab](https://github.com/rgdevment/CopyPaste/security) in the repository
2. Click **"Report a vulnerability"**
3. Fill in the details using the template provided
4. Submit privately — only maintainers will see it

**Choose whichever method is most comfortable for you.** What matters is that we hear from you.

**Include in your report:**

- **Description** — Clear explanation of the vulnerability
- **Impact** — What could an attacker do? Who is affected?
- **Steps to Reproduce** — How can we reproduce the issue?
- **CopyPaste Version** — Which version is affected?
- **OS and version** — e.g., Windows 11 23H2, macOS Sequoia 15.1, Ubuntu 24.04
- **Proof of Concept** (optional) — Code or screenshots demonstrating the issue
- **Suggested Fix** (optional) — If you have ideas on how to fix it

### What Happens Next?

1. **Acknowledgment (Within 48 Hours)**
   - I'll confirm I received your report
   - I'll let you know if I need more information

2. **Investigation (1-7 Days)**
   - I'll reproduce and analyze the issue
   - Assess severity and impact
   - Develop a fix

3. **Resolution**
   - Create a patch and test it thoroughly
   - Coordinate a release timeline with you
   - Credit you in the release notes (if you want)

4. **Disclosure (After Fix is Released)**
   - Publish a security advisory
   - Notify users to update
   - You can publicly disclose (coordinated disclosure)

### Response Time Expectations

| Severity | Response Time | Fix Target |
| :--- | :---: | :---: |
| **Critical** (Remote code execution, data breach) | 24 hours | 1-3 days |
| **High** (Privilege escalation, significant data leak) | 48 hours | 3-7 days |
| **Medium** (Limited scope, requires user interaction) | 3 days | 1-2 weeks |
| **Low** (Minimal impact, edge cases) | 1 week | Next release |

**I'm one person** (with community help), but I take security seriously. If you don't hear back within the expected timeframe, please follow up—things might've gotten lost.

---

## 🤝 Responsible Disclosure

I believe in **coordinated disclosure** to protect users:

- **Please give me reasonable time to fix the issue** before publicly disclosing it
- I aim to release fixes within 7 days for critical issues
- I'll work with you on a disclosure timeline that protects users
- I'll credit you in the release notes (unless you prefer to remain anonymous)

### My Promise to Security Researchers

**I WILL:**

- ✅ Treat you with respect and gratitude—you're helping protect users
- ✅ Respond promptly to your report (within 48 hours)
- ✅ Keep you updated throughout the investigation and fix process
- ✅ Credit your work publicly (if you want)
- ✅ Be transparent about the timeline and progress

**I will NEVER:**

- ❌ Threaten legal action against good-faith security researchers
- ❌ Ignore or dismiss legitimate reports
- ❌ Retaliate against reporters in any way
- ❌ Use intimidation tactics or silence critics
- ❌ Blame you for finding vulnerabilities in the code

**Security research makes everyone safer.** I'm grateful for your work and will treat you as a valued partner in protecting the community.

---

## 🏆 Security Researchers Hall of Fame

We're grateful to the security researchers who help make **CopyPaste** safer:

<!-- This section will be updated as we receive and resolve security reports -->

- _No security issues reported yet. Help us stay secure!_

**Want to be listed here?** Report a verified security vulnerability and choose to be credited. We'll add your name (or handle) and a link to your profile if you'd like.

---

## 📚 Additional Security Resources

### For Users

- **Keep CopyPaste Updated** — Enable automatic updates or check for new releases regularly
- **Review Clipboard History** — Periodically check what's being stored and delete sensitive items
- **Configure Retention** — Set shorter retention periods if you handle highly sensitive data
- **Use Password Managers** — Their clipboard content is automatically excluded from history

### For Developers

- **Read the Code** — The entire codebase is open source: [CopyPaste Repository](https://github.com/rgdevment/CopyPaste)
- **Review Dependencies** — Check `pubspec.yaml` for third-party packages we use
- **Security Best Practices** — Follow secure coding guidelines when contributing

---

## 🔐 Cryptographic Disclosure

**CopyPaste does not currently use cryptographic functions for data storage.**

- Clipboard history is stored in **plaintext** in a local SQLite database
- Database files are protected by **OS-level file system permissions** (Windows, macOS, and Linux)
- No encryption is applied to stored clipboard data

**Why?**

- The database is local-only and protected by your OS user account
- Encryption would add complexity and potential key management issues
- Performance and startup time would be impacted
- You control physical access to your machine

**Future Consideration:**
If there's community demand for at-rest encryption, we're open to discussing it. Open an issue if this is important to you.

---

## 💬 Questions or Concerns?

We're here to help and answer questions:

- **Security Questions:** Email us at **<github@apirest.cl>** — we're happy to discuss concerns privately
- **General Questions:** [Open a Discussion](https://github.com/rgdevment/CopyPaste/discussions) — ask publicly, we'll answer openly
- **Vulnerability Reports:** Use the private channels above — never post security issues publicly
- **Policy Feedback:** [Open an Issue](https://github.com/rgdevment/CopyPaste/issues/new) — help us improve this policy

**Security is everyone's responsibility.** Thank you for helping keep **CopyPaste** safe for everyone using it.

**Remember:** If you're unsure whether something is a security issue, reach out anyway. I'd rather have a conversation than miss a real problem.

---

<div align="center">
  <p><em>Built securely, transparently, and with ❤️.</em></p>
</div>
