# Migration Guide: 0.2606.0 -> 0.2606.1

Short version: **the Rust API needs no changes** - 0.2606.1 only adds surface. The **C ABI is a
recompile** - two `#[repr(C)]` structs grew a field, so any C/C++ consumer must rebuild against the
regenerated header. Nothing was removed or renamed.

## Rust: nothing to do

Every addition is additive. New names, no changed signatures:

- `L2Domain` - a new type: the cores sharing one L2 cache instance, with its own `size_bytes`, an
  `AffinityMask`, a `core_count`, and an `l3_domain` back-link to the L3 it nests inside.
- `CpuInfo::l2_domains: Vec<L2Domain>` and `CpuInfo::l2_domain_mask(u16)` - the L2 table, mirroring
  `l3_domains` / `l3_domain_mask`.
- `Lp::l2_domain: u16` (and the `Lp::NO_L2` sentinel) - each logical processor's index into
  `l2_domains`.
- `impl Extend<usize> for AffinityMask` - `mask.extend(ids)`; `FromIterator` now delegates to it.

The one thing to know: if you build a `CpuInfo` or `Lp` by **exhaustive struct literal**, or destructure
one with a pattern that names every field, the compiler will flag the new fields. In practice these come
out of `CpuInfo::detect()`, so almost nobody constructs them by hand - add the new fields if you do.

### AffinityMask Debug/Display output changed

`AffinityMask` now renders logical-processor sets as coalesced ranges. The shape changed, not any method.
If something parses the old output, update it (and prefer `iter()` / `count()` over parsing text):

- `Debug` -> developer view: `AffinityMask { cores: [0-3, 6-9], count: 8 }`
- `Display` -> bare value: `[0-3, 6-9]` (`[]` when empty)

## C ABI: recompile against the regenerated header

`GdtCpusLp` and `GdtCpusCpuInfo` each gained a field, so their `#[repr(C)]` layout changed. This is a hard
ABI break: a binary built against the 0.2606.0 header reads the wrong offsets. Regenerate `gdt_cpus.h`
(produced on build) and recompile - no source changes are required for code that does not touch L2.

New C surface:

- `GdtCpusLp.l2_domain` (`uint32_t`, `GDT_CPUS_NO_L2` when none) - new field.
- `GdtCpusCpuInfo.l2_domain_count` (`uint64_t`) - new field.
- `GdtCpusL2Domain { size_bytes, core_count, lp_count, l3_domain }` - new struct.
- `gdt_cpus_get_l2_domain(index, out)` and `gdt_cpus_get_l2_domain_lp(domain, lp_index, out_os_id)` - new
  accessors, same shape as the L3 ones.
- `GDT_CPUS_NO_L2` (`UINT32_MAX`) - new sentinel.

Enumerating the L2 group of each domain:

```c
GdtCpusCpuInfo info;
gdt_cpus_cpu_info(&info);
for (uint64_t d = 0; d < info.l2_domain_count; ++d) {
    GdtCpusL2Domain l2;
    gdt_cpus_get_l2_domain(d, &l2);
    // l2.lp_count logical processors share l2.size_bytes;
    // l2.l3_domain is the parent L3 index (GDT_CPUS_NO_L3 if none).
}
```
