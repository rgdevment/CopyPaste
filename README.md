<div align="center">
  <img src="CopyPaste.UI/Assets/CopyPasteLogo.ico" width="128" height="128" alt="CopyPaste Logo"/>
  <h1>CopyPaste</h1>
  <p>
    <strong>The clipboard manager Windows actually deserves.</strong>
  </p>

  <p>
    <a href="https://github.com/tu-usuario/CopyPaste/actions"><img src="https://img.shields.io/github/actions/workflow/status/tu-usuario/CopyPaste/ci.yml?style=flat-square&label=Build" alt="Build Status"/></a>
    <a href="#license"><img src="https://img.shields.io/github/license/tu-usuario/CopyPaste?style=flat-square" alt="License"/></a>
    <img src="https://img.shields.io/badge/Platform-Windows%2011-0078D4?style=flat-square&logo=windows" alt="Platform"/>
    <img src="https://img.shields.io/badge/.NET-10%20Preview-512BD4?style=flat-square" alt=".NET Version"/>
  </p>

  <p>
    <a href="https://github.com/tu-usuario/CopyPaste/releases/latest"><strong>📥 Download Latest Version</strong></a>
    ·
    <a href="https://github.com/tu-usuario/CopyPaste/issues">Report Bug</a>
    ·
    <a href="https://github.com/tu-usuario/CopyPaste/discussions">Request Feature</a>
  </p>
</div>

---

## 💡 The Motivation
**Why build another clipboard manager?**

Windows has plenty of clipboard managers. Some are functional but ugly. Others are "tanks"—heavy, feature-bloated software that consumes 500MB of RAM just to remember text.

We wanted something else.
* We wanted the **minimalist aesthetic** of macOS utilities.
* We wanted the **native feel** of Windows 11 notifications.
* We wanted **obsessive performance**.

**This is a passion project.** We are not selling anything. We are building the tool *we* needed for our daily work, and we are committed to maintaining and refining it for years to come. Open Source, free, and transparent.

---

## ✨ Features (Alpha)

This project is currently in **Alpha / Proof of Concept**. We are building on the bleeding edge.

* **Native & Modern:** Built with **WinUI 3 (Windows App SDK)**. It doesn't look like an alien app; it looks like part of Windows.
* **Performance First:** Compiled with **NativeAOT** for instant startup and low memory footprint (~30-60MB).
* **Zero Distractions:** It stays out of your way until you need it.
* **Rich History:** Supports Text, Images, and Files (drag & drop ready).

### 📸 Screenshots
*(Work in progress)*

> *Soon*

---

## 🛠 Tech Stack
We are using this project to push the boundaries of modern .NET development on the Desktop:

* **Language:** C# 14
* **Framework:** .NET 10 Preview
* **UI Framework:** WinUI 3
* **Architecture:** Dependency Injection, MVVM-like structure, Low-level Hooks.

---

## 🚀 Getting Started

### Installation
1.  Go to the [Releases Page](https://github.com/tu-usuario/CopyPaste/releases).
2.  Download the **Installer (`.exe`)** or the **Portable (`.zip`)** version.
3.  **Note:** Since this is an open-source project, the binary is self-signed. Windows SmartScreen might warn you. Click `More Info` -> `Run Anyway`.

### Compatibility
* **Supported:** Windows 10 (1809+), Windows 11.
* **Architectures:** x64 (Tested), ARM64 (Experimental - Feedack needed!).

---

## 🚧 Roadmap & Known Issues
We believe in radical transparency. Here is where we stand:

- [ ] **Internationalization:** The UI is currently hardcoded in **Spanish**. English support is coming in v0.2.0.
- [ ] **Settings UI:** Configuration is currently static. A settings menu is in development.
- [ ] **ARM64 Testing:** We build for ARM, but we haven't tested it on physical hardware yet.

---

## 🤝 Contributing
This is a community effort. We welcome anyone who shares our vision of high-quality, native Windows apps.

1.  **Fork** the repository.
2.  Create your **Feature Branch** (`git checkout -b feature/AmazingFeature`).
3.  **Commit** your changes.
4.  **Push** to the branch.
5.  Open a **Pull Request**.

**Specific Help Wanted:**
* If you have a Surface Pro X or any **Snapdragon** device, please test the ARM64 build and let us know if it works!

---

## 📜 License
Distributed under the **GNU General Public License v3.0**. See `LICENSE` for more information.

> **Built with ❤️ and C#**
