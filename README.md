
# <img width="60" height="60" src="https://github.com/vinceajello/drill/blob/main/resources/icon.png?raw=true" alt="drill app icon"> Drill — Multi-Platform SSH Tunnel Manager


**Drill** is a lightweight, user-friendly application written in Rust that aims to make SSH tunneling approachable and painless. With an intuitive graphical interface, you can quickly configure local and remote port forwarding, manage multiple tunnels, and monitor their status in real time — all from the status bar / system tray of your preferred OS.

---

## ✨ Features

* **Simple GUI** – Create and manage tunnels without memorizing SSH flags
* **Local & Remote Port Forwarding** – Easily configure both forwarding types
* **Tunnel Management** – Create, connect, disconnect, and delete tunnels in a few clicks
* **System Tray Integration** – Runs quietly in the background with quick access
* **Cross-Platform** – Designed to work across major operating systems

> 💾 **Data Storage**
> Drill stores its configuration and artifacts in:
> `/UserHomeDirectory/.drill`

---

> ⚠️ **Project Note**
> This app was “vibe-coded” using **Claude Haiku 4.5** while I’m actively learning Rust. A refactor is planned once I finish my Rust course—purely for fun, learning, and improving code quality 😄
>
> At the end of this README, you’ll also find a section outlining some **considerations and lessons learned** during this vibe-driven development process.

---

## 🧑‍💻 Building From Source

### Windows

1. **Install Rust**
   Download and install Rust from [rustup.rs](https://rustup.rs)

2. **Clone the Repository**

   ```bash
   git clone https://github.com/vinceajello/drill.git
   ```

3. **Navigate to the Project Directory**

   ```bash
   cd drill
   ```

4. **Build the Project**

   ```bash
   cargo build --release
   ```

The compiled executable will be available at:

```
target/release/drill.exe
```

---

### macOS

1. **Install Rust**
   Download and install Rust from [rustup.rs](https://rustup.rs)

2. **Clone the Repository**

   ```bash
   git clone https://github.com/vinceajello/drill.git
   ```

3. **Navigate to the Project Directory**

   ```bash
   cd drill
   ```

4. **Install cargo-packager**

   ```bash
   cargo install cargo-packager
   ```

5. **Build the macOS Application Bundle**

   ```bash
   cargo packager --release
   ```

The compiled `.app` bundle will be available in:

```
target/release/bundle/macos/
```

> **MacOS Note:** Since the app is not code-signed when built locally, macOS may prevent it from opening. If you see a security warning, go to **System Settings > Privacy & Security** and click **"Open Anyway"** to allow the app to run. Alternatively, right-click the app and select **"Open"** from the context menu.

---

### Linux

🚧 **Coming Soon**
Build instructions for Linux will be added shortly.

---

## 📦 Pre-Built Binaries

You can download the latest pre-built binaries from the
**GitHub Releases** page:

👉 [https://github.com/vinceajello/drill/releases](https://github.com/vinceajello/drill/releases)

---

## 🎨 Artwork & Graphics Attribution

Some of the graphical assets used in this project (such as the **tool icons / stickers**) were sourced from **Shmector.com**.

> shmector.com thank you very much for your work on this awesome icon set

* **Title:** Collection of tool stickers
* **Author:** SHMECTOR.COM
* **License:** CC0 1.0 Universal (Public Domain)
* **Source:** [https://shmector.com/free-vector/technics/collection_of_tool_stickers/12-0-1659](https://shmector.com/free-vector/technics/collection_of_tool_stickers/12-0-1659)

## 🤔 Vibe-Coding Considerations

Drill was developed for **macOS** in the span of one to two days of *vibe-coding* by a developer who, at the time, had little to no experience with Rust.

As a result, parts of the codebase may appear questionable, unconventional, or outright ugly to experienced Rust developers—and that’s a fair assessment. Code quality, architecture, and idiomatic Rust were not the primary goals during this phase.

The point, however, is outcome over perfection.

In roughly one day of work, Drill replaced another tool I had been using—one that followed a **subscription-based model** and was backed by a company. With a single day of focused effort, I now use a tool I own, understand, and control. I’m no longer a customer; I’m the maintainer.

This project is a reminder that:

* Software doesn’t have to be perfect to be useful
* Learning and building can happen simultaneously
* Owning your tools—even imperfect ones—can be empowering
* Vibe-Coding enables a new frontier for creative individuals enjoy it.

A refactor is planned once my Rust knowledge matures. Until then, Drill does its job, and that alone makes it a success.

---

## 🔧 Enjoy Your Drills

If you run into issues, have ideas, or just want to share feedback, feel free to open an issue or pull request.

Happy tunneling! 🚀

