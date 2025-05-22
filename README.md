# **gdt-cpus** – Game Developer's Toolkit for CPU Management

<p align="center"><b><i>Pin it. Prioritize it. Dominate it.</i></b></p>

<p align="center">
  <a href="#-quick-flex"><img src="https://img.shields.io/badge/Rust-E57324?style=for-the-badge&logo=rust&logoColor=white" alt="Rust"></a>
  <a href="https://crates.io/crates/gdt-cpus"><img src="https://img.shields.io/crates/v/gdt-cpus.svg?style=for-the-badge&color=orange"></a>
  <a href="https://docs.rs/gdt-cpus"><img src="https://img.shields.io/badge/docs‑rs-online-orange.svg?style=for-the-badge"></a>
  <a href="LICENSE"><img src="https://img.shields.io/badge/license-MIT%20OR%20Apache--2.0-orange?style=for-the-badge"></a>
</p>

---

You've got cores. *A lot of them.* Stop letting your OS babysit them like it's 2004.

With `gdt-cpus`, you **take control**. Hybrid architectures? P/E cores? SMT voodoo? Handled.

Windows, Linux, macOS? Handled.

Your ego? Also handled — this lib *knows* you're here to squeeze every last nanosecond.

---

## ✨ **Features That Actually Matter**

> *Monitoring and NUMA awareness? Out of scope for now.
> Why? Because you've got more important things to ship first.*

* 🗺️ **CPU Topology? Got it.**

  Vendor, model, sockets, cores, logical threads, cache hierarchies. No more guessing what you're running on.

* 🧟‍♂️ **Hybrid-Aware like a Boss**

  Detect and exploit P-cores and E-cores. Be the scheduler your OS wishes it could be.

* 🪢 **Thread Affinity API**

  Pin threads to specific cores. Dominate cache locality. Laugh at poor thread migrations.

* 🎚️ **Thread Priority Control**

  From *lowly background* to *time-critical god mode*.

* 🎮 **Game-Dev First**

  You won't find some academic NUMA experiments here. Just *useful* tools for real-time workloads.

* 🧩 **C FFI Support**

  Because your C++ friends need to know how to party too. (Or just call `gdt-cpus-sys` directly.)
  With full CMake support. No more CMake hell. See `examples/c/basic_info` for details.

* 🛡️ **Minimal Dependencies You'll Regret**

  `log`, `thiserror`. That's it. No shady transitive surprises.

  > (Okay fine… `raw-cpuid`, `windows`, `libc`, `mach` — because CPUs still mumble in syscalls.)

---

## 🚀 **Quick Flex**

```rust
use gdt_cpus::*;

println!("Physical cores: {}", num_physical_cores());
println!("Logical cores: {}", num_logical_cores());

pin_thread_to_core(0).expect("This thread is going nowhere else now.");
set_thread_priority(ThreadPriority::AboveNormal).unwrap();

if is_hybrid() {
    println!("P/E Cores? Oh, we're playing on expert difficulty.");
}
```

---

## 🏎️ **Under The Hood: How We Tame The Silicon Beast**

`gdt-cpus` isn't just calling `num_cpus::get()`. That's for amateurs. We dive deep into OS-specific APIs so you don't have to:

| OS | API Madness We Handle |
|----|------------------------|
| **Windows** | `GetLogicalProcessorInformationEx`, Registry, `SetThreadAffinityMask` |
| **Linux** | `sysfs`, `/proc/cpuinfo`, `cpuid`, `sched_setaffinity`, `setpriority` |
| **macOS** | `sysctl`, `cpuid` (x86), QoS, `thread_policy_set` |

All this pain, abstracted away into one beautiful, cross-platform Rust API. We do the dirty work. You reap the rewards.

> "Abstraction without insight is just hiding the problem. `gdt-cpus` gives you both."

---

## 🔥 **Hybrid CPU Dominance: P-Cores & E-Cores on a Leash**

Those P-cores and E-cores? They're not just suggestions. `gdt-cpus` tells you exactly what you've got, so you (or your job system) can:

### P-Cores (Performance)

Unleash the beasts for your critical, latency-sensitive work. Pin 'em if you got 'em (and if your OS isn't Apple).

### E-Cores (Efficiency)

Perfect for background tasks, parallel number crunching that can wait, or just saving your laptop's battery. Offload wisely.

`gdt-cpus` gives you the intel. Using it to make your app scream (or sip power) is up to you.

---

## 📊 **Performance Proof: Lower Latency, Smoother Frames**

Words are wind. `gdt-cpus` enables real, measurable wins:

| Scenario | Standard | With gdt-cpus | Improvement |
|----------|----------|---------------|-------------|
| **Audio Latency** (p99 Jitter µs, Linux i7) | 948,188 | ~0 (TimeCritical) | ~100% |
| **Frame Jitter** (p99 Jitter µs, Linux i7) | 52 | 5 (Highest Priority) | 90% |
| **Task Latency** (p99 µs, macOS M3 Max) | 1,732 | 997 (P/E Strategy) | 42% |
| **Streaming Pipeline** (p50 Latency µs, macOS) | 4,743 | 3,517 (Prioritized) | 26% |

These aren't just numbers, they're your app running better. You're welcome.

---

## 🤔 **gdt-cpus vs. The "Alternatives" (Bless Their Hearts)**

Sure, there are other ways to poke at your CPU. If you like basic, or platform-locked, or just... less.

| Capability | `gdt-cpus` (The Pro) | `num_cpus` (The Intern) | `raw-cpuid` (The x86 Nerd) | OS APIs (The DIY Nightmare) |
|------------|----------------------|--------------------------|----------------------------|-----------------------------|
| Core Counts | 👑 | 👍 | 🤷 | 😬 |
| Full Topology (Cache, Sockets) | 👑 | ❌ | 🔬 (x86) | 🤯 |
| P/E Core Detection | 👑 | ❌ | ❌ | 🧩 |
| Thread Affinity | 👑 | ❌ | ❌ | 🧩 |
| Thread Priority | 👑 | ❌ | ❌ | 🧩 |
| Cross-Platform Ease | 👑 | 👍 | limited | ❌❌❌ |

> We ❤️ `num_cpus` – full respect!<br>
> Our brains just speak in sarcasm & memes 🤷‍♂️<br>
> (`num_cpus` paved the way for CPU introspection in Rust – `gdt-cpus` just straps a rocket to it. 🚀)

Choose wisely. Or just choose `gdt-cpus` and be done with it.

---

## 🧠 **The SWOT Analysis (Because We're "Strategic")**

### 💪 Strengths (Obvious Stuff)

* Deep CPU insights, cross-platform.
* P/E core aware. Your hybrid CPU will love you.
* Thread pinning & priority control that *works*.
* Foundation for god-tier task systems (hi, `gdt-jobs`!).
* C FFI via `gdt-cpus-sys`? Check. Your C++ will thank you.

### 📉 Weaknesses (If We *Must*)

* Not magic. You still gotta write good code on top.
* Apple Silicon affinity? Apple says "lol no". We report that accurately.
* Might be overkill if all you need is `num_cpus::get()`. (But why settle?)

### 🚀 Opportunities (World Domination Plans)

* More NUMA smarts for server beasts.
* Even *more* detailed cache info. Because why not.
* Your favorite engine using `gdt-cpus` under the hood.

### ⚠️ Threats (The Competition... Kinda)

* OS schedulers *might* get smarter. Someday. Maybe.
* Someone writing an even *more* arrogant README. Unlikely.

---

## 🌐 **Proven on Real Hardware**

Tested across:

* 🐧 Linux with baremetal and containers (LXC-tested, yes, it even respects *your* artificial limits)
* 🪟 Windows (Hyper-Threading chaos? We navigate it.)
* 🍎 macOS (Including Apple Silicon and their obsession with Efficiency Cores)

> Curious what it actually prints? Check out docs/basic_info.md for full example output.

---

## **Versioning - CalVer, Deal With It**

> Wait, CalVer for a lib? Ya Idjits or something?<br>
> (Bobby Singer voice, obviously.)

Yep, we timestamp our releases instead of counting up semantic digits. Why? Because we're just built different. And because:

| CalVer Perk           | Why You Care |
|-----------------------|--------------|
|Instant age check      | `25.5.0` → May 2025. No need to diff tags to see if a crate is fossilized or fresh off the compiler.|
|Honesty about breakage | New month? Could be a breaking change. You’ll know from the number *and* from a Migration Guide `25.4` → `25.5` in the repo. We're not shy.|
|Works fine with Cargo  | `cargo add gdt-cpus@^25.5` still pins the May 2025 API line; you won’t auto-upgrade to `26.1` unless you explicitly ask for it. Cargo gets it.|
|Less bike-shedding     | We’d rather spend time tuning work-steal loops and optimizing P/E core scheduling than debating whether the last commit was “minor” or “patch”. Priorities, people.|

**TL;DR**:

Each year/month is an API epoch. If we break you, the migration doc shows the fix; if we don’t, cargo update is painless.

And if we mess up, the date tells you exactly when to roast us in Issues. 😎

> (We’re not idjits – just impatient.)

---

## **How Can I Contribute?**

Find something that’s missing, broken, or just less performant than your standards require.

Open an issue. Bonus points if you make a PR. A 🍪 if benchmarks go brrrrr.

But wait, where is the **CODE_OF_CONDUCT**?

**Code of what?** Quoting a famous internet meme:

> “Apologies for the very personal question, but were you homeschooled by a pigeon?”

We're all civilised here. Just don't be an asshole and we're good. 🤞🏻

And hey, mad props to the entire Rust community. Y'all make low-level coding sexy again. This stuff is built with love, for the love of the game (and performant Rust).

---

## 🧩 **Part of the GDT Ecosystem**

`gdt-cpus` is part of the Game Developer's Toolkit — libraries built with years of experience from top-tier studios:

* **gdt-cpus** - Pin it. Prioritize it. Dominate it. You're looking at it right now!
* **gdt-jobs** - High-performance task execution built for games and sims needing serious parallelism.
* **gdt-memory** (Coming Soon) - Because your RAM deserves a tyrant, not a suggestion.

---

## 📦 **Add to Your Project Like a Professional**

```bash
cargo add gdt-cpus
```

Or just copy-paste like it's still the 90s. We don't judge.

---

## 🔥 **Use Cases**

* Write a physics solver that doesn't feel like it's running on a potato.
* Make sure your background AI calculations stay *in the background*.
* Pin your loading threads to E-cores and gameplay to P-cores. Instant karma.
* Benchmark that ridiculous 64-core Threadripper you overpaid for.

> *Remember*: Your OS works *for* you, not the other way around.
> Pin those threads. Prioritize them. And go write code that makes the fans spin.

---

## ⚖️ **License**

MIT OR Apache-2.0 — because we believe in *freedom of choice* (and legally covering our butts).

---

<p align="center">Made with ❤️ by <a href="https://wildpixelgames.com">Wild Pixel Games</a> - We know CPUs.</p>
<p align="center"><i>"My CPU used to cry itself to sleep. Then I found <code>gdt-cpus</code>."</i> – A Very Smart Developer</p>
