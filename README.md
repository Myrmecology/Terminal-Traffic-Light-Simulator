# 🚦 Terminal Traffic Light Simulator 🚦💻🦀

A fast, clean, and fun **traffic light simulation** built in **100% pure Rust**. This terminal-based project displays animated traffic light behavior and simulates cars reacting in real time — all without using any GUI libraries or external engines.

> Designed to be both a fun programming exercise and a tool for expansion 

---

## 🎯 Features 🦀

- ✅ Real-time traffic light simulation (Red → Green → Yellow → Red...)
- ✅ Cars that stop on red, move on green, and slow on yellow
- ✅ Animated terminal output — no scrolling log, just clean screen updates
- ✅ Pure Rust — no external dependencies required (except for optional key input)
- ✅ Cross-platform support (built and tested on Windows with Bash + VS Code)

---

## 🧱 How It Works 🦀

- The simulation runs in a loop called a `tick`, updating light state and car behavior every cycle.
- Cars are represented as `🚗` or `🅿️` (parked), and move horizontally along a virtual road.
- The light controls car movement logic with simple, idiomatic Rust enums and pattern matching.

---

## 🛠 Getting Started 🦀

1. **Clone the repo**

```bash
git clone https://github.com/Myrmecology/Terminal-Traffic-Light-Simulator.git
cd traffic_light_simulator
Type cargo run and watch the simulation in action
Built With
Rust 🦀

📜 License
This project is licensed under the MIT License — see the LICENSE file for details.



