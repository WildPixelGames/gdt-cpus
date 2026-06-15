# Output from examples/thread_priorities.rs on various platforms

## Windows 11

```bash
     Running `target\release\examples\thread_priorities.exe`
priority_capabilities() - what this process can currently get:
  effective ranks : [0, 1, 2, 3, 4, 5, 6]
  distinct levels : 7/7
  row detail      : concrete scheduler API and value applied below
  verdict         : the full ladder is effectively distinct here.

set_thread_priority() - what each request actually does:
  Background THREAD_PRIORITY -15
  Lowest THREAD_PRIORITY -2
  BelowNormal THREAD_PRIORITY -1
  Normal THREAD_PRIORITY 0
  AboveNormal THREAD_PRIORITY 1
  Highest THREAD_PRIORITY 2
  TimeCritical THREAD_PRIORITY 15

promote_thread_to_realtime() - the consent API:
  promoted : TimeCritical THREAD_PRIORITY 15
  demoted  : back to normal scheduling
```

## Linux (WSL2), Windows 11

```bash
     Running `target/release/examples/thread_priorities`
priority_capabilities() - what this process can currently get:
  effective ranks : [0, 1, 2, 3, 4, 5, 6]
  distinct levels : 7/7
  row detail      : concrete scheduler API and value applied below
  verdict         : the full ladder is effectively distinct here.

set_thread_priority() - what each request actually does:
  Background nice 19
  Lowest nice 10
  BelowNormal nice 5
  Normal nice 0
  AboveNormal nice -5
  Highest nice -10
  TimeCritical nice -20

promote_thread_to_realtime() - the consent API:
  promoted : TimeCritical [Realtime] SCHED_RR 85
  demoted  : back to normal scheduling
```

## Windows 11 (on Apple M3 Max via Parallels)

```bash
     Running `target\release\examples\thread_priorities.exe`
priority_capabilities() - what this process can currently get:
  effective ranks : [0, 1, 2, 3, 4, 5, 6]
  distinct levels : 7/7
  row detail      : concrete scheduler API and value applied below
  verdict         : the full ladder is effectively distinct here.

set_thread_priority() - what each request actually does:
  Background THREAD_PRIORITY -15
  Lowest THREAD_PRIORITY -2
  BelowNormal THREAD_PRIORITY -1
  Normal THREAD_PRIORITY 0
  AboveNormal THREAD_PRIORITY 1
  Highest THREAD_PRIORITY 2
  TimeCritical THREAD_PRIORITY 15

promote_thread_to_realtime() - the consent API:
  promoted : TimeCritical THREAD_PRIORITY 15
  demoted  : back to normal scheduling
```

## macOS 26.5.1

```bash
     Running `target/release/examples/thread_priorities`
priority_capabilities() - what this process can currently get:
  effective ranks : [0, 1, 2, 3, 4, 5, 6]
  distinct levels : 7/7
  row detail      : concrete scheduler API and value applied below
  verdict         : the full ladder is effectively distinct here.

set_thread_priority() - what each request actually does:
  Background QoS Background
  Lowest QoS Utility
  BelowNormal QoS Default
  Normal QoS UserInitiated
  AboveNormal QoS UserInteractive
  Highest QoS UserInteractive
  TimeCritical [Realtime] SCHED_RR 47

promote_thread_to_realtime() - the consent API:
  promoted : TimeCritical [Realtime] SCHED_RR 47
  demoted  : back to normal scheduling
```

## Linux (baremetal), CachyOS, Desktop, 7.0.11-1-cachyos

```bash
     Running `target/release/examples/thread_priorities`
priority_capabilities() - what this process can currently get:
  effective ranks : [0, 1, 2, 3, 4, 5, 6]
  distinct levels : 7/7
  row detail      : concrete scheduler API and value applied below
  verdict         : the full ladder is effectively distinct here.

set_thread_priority() - what each request actually does:
  Background nice 19
  Lowest nice 10
  BelowNormal nice 5
  Normal nice 0
  AboveNormal [Brokered] nice -5
  Highest [Brokered] nice -10
  TimeCritical [Brokered, Clamped] nice -15

promote_thread_to_realtime() - the consent API:
  promoted : TimeCritical [Realtime] SCHED_RR 85
  demoted  : back to normal scheduling
```

## Linux (baremetal), Debian 13.5, Proxmox, 7.0.2-7-pve

```bash
     Running `target/release/examples/thread_priorities`
priority_capabilities() - what this process can currently get:
  effective ranks : [0, 1, 2, 3, 4, 5, 6]
  distinct levels : 7/7
  row detail      : concrete scheduler API and value applied below
  verdict         : the full ladder is effectively distinct here.

set_thread_priority() - what each request actually does:
  Background nice 19
  Lowest nice 10
  BelowNormal nice 5
  Normal nice 0
  AboveNormal nice -5
  Highest nice -10
  TimeCritical nice -20

promote_thread_to_realtime() - the consent API:
  promoted : TimeCritical [Realtime] SCHED_RR 85
  demoted  : back to normal scheduling
```

## Linux (LXC, limited to 2 cores, inside Proxmox), Debian 13.5, 7.0.2-7-pve

```bash
     Running `target/release/examples/thread_priorities`
priority_capabilities() - what this process can currently get:
  effective ranks : [0, 1, 2, 3, 4, 5, 6]
  distinct levels : 7/7
  row detail      : concrete scheduler API and value applied below
  verdict         : the full ladder is effectively distinct here.

set_thread_priority() - what each request actually does:
  Background nice 19
  Lowest nice 10
  BelowNormal nice 5
  Normal nice 0
  AboveNormal -> Normal [NoBroker] nice 0
  Highest -> Normal [NoBroker] nice 0
  TimeCritical -> Normal [NoBroker] nice 0

promote_thread_to_realtime() - the consent API:
  denied   : Permission denied: real-time promotion denied on every path (direct SCHED_RR, portal, rtkit): Permission denied: Setting SCHED_RR with priority 85: Operation not permitted (os error 1)
```

## Linux (baremetal), Debian 12.14, MS-R1, 6.6.10-cix-build-generic

```bash
     Running `target/release/examples/thread_priorities`
priority_capabilities() - what this process can currently get:
  effective ranks : [0, 1, 2, 3, 4, 5, 6]
  distinct levels : 7/7
  row detail      : concrete scheduler API and value applied below
  verdict         : the full ladder is effectively distinct here.

set_thread_priority() - what each request actually does:
  Background nice 19
  Lowest nice 10
  BelowNormal nice 5
  Normal nice 0
  AboveNormal [Brokered] nice -5
  Highest [Brokered] nice -10
  TimeCritical [Brokered, Clamped] nice -15

promote_thread_to_realtime() - the consent API:
  promoted : TimeCritical [Realtime] SCHED_RR 20
  demoted  : back to normal scheduling
```

## Linux (baremetal), SteamOS, Steam Deck, 6.11.11-valve29-1-neptune-611-g2dcfaf4df7ac

```bash
     Running `target/release/examples/thread_priorities`
priority_capabilities() - what this process can currently get:
  effective ranks : [0, 1, 2, 3, 4, 5, 6]
  distinct levels : 7/7
  row detail      : concrete scheduler API and value applied below
  verdict         : the full ladder is effectively distinct here.

set_thread_priority() - what each request actually does:
  Background nice 19
  Lowest nice 10
  BelowNormal nice 5
  Normal nice 0
  AboveNormal [Brokered] nice -5
  Highest [Brokered] nice -10
  TimeCritical [Brokered, Clamped] nice -15

promote_thread_to_realtime() - the consent API:
  promoted : TimeCritical [Realtime] SCHED_RR 20
  demoted  : back to normal scheduling
```
