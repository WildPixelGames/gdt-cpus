# Output from examples/l3_domains.rs on various platforms

## Windows 11

```bash
     Running `target\release\examples\l3_domains.exe`
CPU: AMD Ryzen 9 5950X 16-Core Processor - 16 cores / 32 threads, 2 L3 domain(s):
  domain 0:    32 MiB,  8 cores, 16 threads
  domain 1:    32 MiB,  8 cores, 16 threads

1000000 round trips per configuration...

SMT siblings, one core      (lp   0 <-> lp   1):     26.4 ns/round-trip
Two cores, SAME L3 domain   (lp   0 <-> lp   2):     45.7 ns/round-trip
Two cores, CROSS L3 domains (lp   0 <-> lp  16):    200.5 ns/round-trip

Crossing the L3 fabric costs 4.4x an in-domain round trip; pin cooperating threads with l3_domain_mask().
```

## Linux (WSL2), Windows 11

```bash
     Running `target/release/examples/l3_domains`
CPU: AMD Ryzen 9 5950X 16-Core Processor - 16 cores / 32 threads, 1 L3 domain(s):
  domain 0:    32 MiB, 16 cores, 32 threads

1000000 round trips per configuration...

SMT siblings, one core      (lp   0 <-> lp   1):     54.1 ns/round-trip
Two cores, SAME L3 domain   (lp   0 <-> lp   2):     57.7 ns/round-trip
Two cores, CROSS L3 domains : single L3 domain on this machine, skipped

(Run this on a chiplet CPU - multi-CCD Ryzen/Threadripper or hybrid Intel -
to see the cross-domain latency cliff the L3 table exists for.)
```

## Windows 11 (on Apple M3 Max via Parallels)

```bash
     Running `target\release\examples\l3_domains.exe`
CPU: Apple Silicon - 8 cores / 8 threads, 0 L3 domain(s):

1000000 round trips per configuration...

SMT siblings, one core      : no SMT on this machine, skipped
Two cores, SAME L3 domain   : domain 0 has a single core, skipped
Two cores, CROSS L3 domains : single L3 domain on this machine, skipped

(Run this on a chiplet CPU - multi-CCD Ryzen/Threadripper or hybrid Intel -
to see the cross-domain latency cliff the L3 table exists for.)
```

## macOS 26.5.1

```bash
     Running `target/release/examples/l3_domains`
CPU: Apple M3 Max - 16 cores / 16 threads, 0 L3 domain(s):

1000000 round trips per configuration...

SMT siblings, one core      : no SMT on this machine, skipped
Two cores, SAME L3 domain   : domain 0 has a single core, skipped
Two cores, CROSS L3 domains : single L3 domain on this machine, skipped

(Run this on a chiplet CPU - multi-CCD Ryzen/Threadripper or hybrid Intel -
to see the cross-domain latency cliff the L3 table exists for.)
```

## Linux (baremetal), CachyOS, Desktop, 7.0.11-1-cachyos

```bash
     Running `target/release/examples/l3_domains`
CPU: AMD Ryzen 9 5950X 16-Core Processor - 16 cores / 32 threads, 2 L3 domain(s):
  domain 0:    32 MiB,  8 cores, 16 threads
  domain 1:    32 MiB,  8 cores, 16 threads

1000000 round trips per configuration...

SMT siblings, one core      (lp   0 <-> lp  16):     24.9 ns/round-trip
Two cores, SAME L3 domain   (lp   0 <-> lp   1):     41.9 ns/round-trip
Two cores, CROSS L3 domains (lp   0 <-> lp   8):    191.3 ns/round-trip

Crossing the L3 fabric costs 4.6x an in-domain round trip; pin cooperating threads with l3_domain_mask().
```

## Linux (baremetal), Debian 13.5, Proxmox, 7.0.2-7-pve

```bash
     Running `target/release/examples/l3_domains`
CPU: Intel(R) Core(TM) i7-6700 CPU @ 3.40GHz - 4 cores / 8 threads, 1 L3 domain(s):
  domain 0:     8 MiB,  4 cores,  8 threads

1000000 round trips per configuration...

SMT siblings, one core      (lp   0 <-> lp   4):     48.8 ns/round-trip
Two cores, SAME L3 domain   (lp   0 <-> lp   1):    116.8 ns/round-trip
Two cores, CROSS L3 domains : single L3 domain on this machine, skipped

(Run this on a chiplet CPU - multi-CCD Ryzen/Threadripper or hybrid Intel -
to see the cross-domain latency cliff the L3 table exists for.)
```

## Linux (LXC, limited to 2 cores, inside Proxmox), Debian 13.5, 7.0.2-7-pve

```bash
     Running `target/release/examples/l3_domains`
CPU: Intel(R) Core(TM) i7-6700 CPU @ 3.40GHz - 2 cores / 2 threads, 1 L3 domain(s):
  domain 0:     8 MiB,  2 cores,  8 threads

1000000 round trips per configuration...

SMT siblings, one core      : no SMT on this machine, skipped
Two cores, SAME L3 domain   (lp   1 <-> lp   4):    125.3 ns/round-trip
Two cores, CROSS L3 domains : single L3 domain on this machine, skipped

(Run this on a chiplet CPU - multi-CCD Ryzen/Threadripper or hybrid Intel -
to see the cross-domain latency cliff the L3 table exists for.)
```

## Linux (baremetal), Debian 12.14, MS-R1, 6.6.10-cix-build-generic

```bash
     Running `target/release/examples/l3_domains`
CPU: CIX P1 CP8180 - 12 cores / 12 threads, 1 L3 domain(s):
  domain 0:    12 MiB, 12 cores, 12 threads

1000000 round trips per configuration...

SMT siblings, one core      : no SMT on this machine, skipped
Two cores, SAME L3 domain   (lp   0 <-> lp   1):    233.2 ns/round-trip
Two cores, CROSS L3 domains : single L3 domain on this machine, skipped

(Run this on a chiplet CPU - multi-CCD Ryzen/Threadripper or hybrid Intel -
to see the cross-domain latency cliff the L3 table exists for.)
```

## Linux (baremetal), SteamOS, Steam Deck, 6.11.11-valve29-1-neptune-611-g2dcfaf4df7ac

```bash
     Running `target/release/examples/l3_domains`
CPU: AMD Custom APU 0405 - 4 cores / 8 threads, 1 L3 domain(s):
  domain 0:     4 MiB,  4 cores,  8 threads

1000000 round trips per configuration...

SMT siblings, one core      (lp   0 <-> lp   1):     80.3 ns/round-trip
Two cores, SAME L3 domain   (lp   0 <-> lp   2):    101.2 ns/round-trip
Two cores, CROSS L3 domains : single L3 domain on this machine, skipped

(Run this on a chiplet CPU - multi-CCD Ryzen/Threadripper or hybrid Intel -
to see the cross-domain latency cliff the L3 table exists for.)
```
