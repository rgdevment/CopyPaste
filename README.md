<div align="center">
  <img src="CopyPaste.UI/Assets/CopyPasteLogo.ico" width="140" height="140" alt="CopyPaste App Logo"/>

  <h1>CopyPaste</h1>
  <h3>The Modern Clipboard Manager for Windows</h3>

  <p>
    <strong>High Performance • Open Source • Native Design</strong>
  </p>

  <p>
    <a href="https://github.com/rgdevment/CopyPaste/actions">
      <img src="https://img.shields.io/github/actions/workflow/status/rgdevment/CopyPaste/ci.yml?style=flat-square&logo=github-actions&label=Build" alt="Build Status"/>
    </a>
    <a href="https://github.com/rgdevment/CopyPaste/releases">
      <img src="https://img.shields.io/github/v/release/rgdevment/CopyPaste?include_prereleases&style=flat-square&label=Latest&color=0078D4" alt="Latest Release"/>
    </a>
    <img src="https://img.shields.io/badge/Platform-Windows%2010%20%7C%2011-0078D4?style=flat-square&logo=windows" alt="Platform Windows"/>
    <a href="#license">
      <img src="https://img.shields.io/github/license/rgdevment/CopyPaste?style=flat-square&color=lightgrey" alt="License"/>
    </a>
    <a href="https://github.com/rgdevment/CopyPaste/releases">
      <img src="https://img.shields.io/github/downloads/rgdevment/CopyPaste/total?style=flat-square&color=success&label=Downloads" alt="Total Downloads"/>
    </a>
  </p>

  <p align="center">
    <a href="https://github.com/rgdevment/CopyPaste/releases/latest">
      <img src="https://img.shields.io/badge/📥_Download_CopyPaste_App-Click_Here-blueviolet?style=for-the-badge&logo=windows" alt="Download Now" height="40"/>
    </a>
  </p>
</div>

---

## 🚀 Introduction

**CopyPaste** is a next-generation **Clipboard Manager** designed specifically for Windows 10 and 11.

We prioritize **Native Aesthetics, Obsessive Performance, and Rock-Solid Stability.** It’s not just about low RAM usage; it’s about a design that feels right and a robustness you can rely on daily. Built on the bleeding edge with **.NET 10 Preview** and **WinUI 3**, it seamlessly integrates with your workflow, feeling less like a third-party tool and more like a feature Windows forgot to include.

> **⚠️ Disclaimer:** Currently tested and validated primarily on **Windows 11 (x64)**.

---

## 💡 The Motivation
**Why build another clipboard manager?**

There are certainly capable tools out there, but many fail to deliver a modern, cohesive **experience**. It's not just about being functional; software today should be attractive, comfortable, and intuitive.

We don't want a Windows XP or Windows 7 experience in 2026. We aim to go further.
* **The Goal:** Bring the polish and fluidity often associated with macOS utilities to Windows.
* **The Philosophy:** Adopt a pure Windows 11 design language so the app feels 100% native.
* **The Standard:** Efficiency (NativeAOT) is non-negotiable, but so is visual elegance.

**This is a passion project.** We are not selling data. We are not upselling "Pro" features. We are building the tool *we* needed for our daily development work, and we are sharing it with the community. Open Source, free, and transparent.

---

## ✨ Key Features (Alpha)

This project is currently in **Alpha / Proof of Concept**. We are pioneering desktop development with C# 14.

* 🎨 **Native & Modern UI:** Built with **WinUI 3**. It respects your system theme (Light/Dark) and uses standard Windows controls.
* ⚡ **Blazing Fast:** Compiled with **NativeAOT** for instant startup times and practically zero lag.
* 🧠 **Smart Memory:** Targeted to run between **30-60MB RAM**. (Configurable limits coming soon).
* 📂 **Rich History:** Seamlessly handles Text, Images, and File paths.
* 🔒 **Privacy First:** 100% Local. Your clipboard history never leaves your machine.

---

## 📸 Screenshots

> *Current status: The UI is fully functional and mimics the Windows 11 notification aesthetic. High-res screenshots and GIFs demonstrating the "CopyPaste App" workflow will be added here shortly.*

*(Placeholder for future GIF: Showing specific copy-paste action)*

---

## 🚀 Getting Started

### Installation
1.  Navigate to the [**Releases Page**](https://github.com/rgdevment/CopyPaste/releases).
2.  Choose your flavor:
    * **Installer (`.exe`):** (Recommended) Installs to AppData and creates shortcuts.
    * **Portable (`.zip`):** Just unzip and run `CopyPaste.exe`.
3.  **SmartScreen Note:** Since this is a community open-source project, the certificate is self-signed. If Windows warns you, click `More Info` -> `Run Anyway`.

### Compatibility
* **OS:** Windows 10 (1809+) or Windows 11.
* **Architecture:**
    * ✅ **x64:** Fully tested and supported.
    * 🧪 **ARM64:** Builds available (Experimental - Feedback needed from Surface/Snapdragon users!).

---

## 🚧 Roadmap & Transparency
We believe in radical honesty about the state of the app:

- [ ] **Internationalization:** UI is currently hardcoded in **Spanish**. English support arrives in v0.2.0.
- [ ] **Configuration UI:** Settings are currently static. A full preferences menu is in development.
- [ ] **Search:** Implementing fuzzy search for history items.

---

## 🤝 Contributing
**CopyPaste** is a community effort. We welcome anyone who shares our vision of high-quality, native Windows apps.

Please read our [**CONTRIBUTING.md**](CONTRIBUTING.md) for details on our code of conduct and the process for submitting pull requests.

**Specific Help Wanted:**
* 🧪 **ARM64 Testing:** If you have a Surface Pro X or Snapdragon Dev Kit, please test our ARM64 build and report issues!

---

## 🛠 Tech Stack (For Developers)
We are using this project to push the boundaries of modern .NET development on the Desktop:

| Technology | Purpose |
| :--- | :--- |
| **C# 14** | Latest language features for cleaner code. |
| **.NET 10 Preview** | "Bleeding edge" runtime performance. |
| **WinUI 3** | The native UI framework for Windows App SDK. |
| **NativeAOT** | Ahead-of-Time compilation (No JIT lag). |
| **Win32 Hooks** | Low-level clipboard monitoring. |

---

## 📜 License
**CopyPaste** - The ultimate clipboard tool for Windows.
Copyright (C) 2026 Mario Hidalgo G. (rgdevment)

This program comes with ABSOLUTELY NO WARRANTY.
This is free software, and you are welcome to redistribute it under certain conditions.
Distributed under the **GNU General Public License v3.0**. See `LICENSE` for more information.

<div align="center">
  <p>Built with ❤️, C#, and too much coffee.</p>
</div>
