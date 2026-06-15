#include <stdio.h>
#include <stdlib.h>
#include <stdbool.h>
#include <inttypes.h>

#include "gdt_cpus/gdt_cpus.h"

int main(void)
{
  GdtCpusCpuInfo info = {0};
  int32_t res = gdt_cpus_cpu_info(&info);

  if (res != GDT_CPUS_ERROR_CODE_SUCCESS)
  {
    printf("Error retrieving CPU info: %s\n", gdt_cpus_error_code_description(res));
    return 1;
  }

  bool is_hybrid = false;
  res = gdt_cpus_is_hybrid(&is_hybrid);
  if (res != GDT_CPUS_ERROR_CODE_SUCCESS)
  {
    printf("Error checking hybrid architecture: %s\n", gdt_cpus_error_code_description(res));
    return 1;
  }

  printf("CPU Information:\n");
  printf("---------------\n");
  printf("Vendor: %s\n", info.vendor_name);
  printf("Model: %s\n", info.model_name);
  printf("Sockets: %" PRIu64 "\n", info.socket_count);
  printf("Physical cores: %" PRIu64 "\n", info.core_count);
  printf("Logical cores: %" PRIu64 "\n", info.lp_count);
  printf("Performance cores: %" PRIu64 "\n", info.performance_cores);
  printf("Efficiency cores: %" PRIu64 "\n", info.efficiency_cores);
  printf("LP-Efficiency cores: %" PRIu64 "\n", info.lp_efficiency_cores);
  printf("NUMA nodes: %" PRIu64 "\n", info.numa_node_count);
  printf("Hybrid architecture: %s\n", is_hybrid ? "Yes" : "No");

  printf("\nL3 domains: %" PRIu64 "\n", info.l3_domain_count);
  for (uint64_t d = 0; d < info.l3_domain_count; ++d)
  {
    GdtCpusL3Domain domain = {0};
    res = gdt_cpus_get_l3_domain(d, &domain);
    if (res != GDT_CPUS_ERROR_CODE_SUCCESS)
    {
      printf("Error retrieving L3 domain: %s\n", gdt_cpus_error_code_description(res));
      return 1;
    }
    printf("  domain %" PRIu64 ": %" PRIu64 " MiB, %u cores, %u threads (lps:",
           d, domain.size_bytes / (1024 * 1024), domain.core_count, domain.lp_count);
    for (uint32_t i = 0; i < domain.lp_count; ++i)
    {
      uint32_t os_id = 0;
      if (gdt_cpus_get_l3_domain_lp(d, i, &os_id) == GDT_CPUS_ERROR_CODE_SUCCESS)
      {
        printf(" %u", os_id);
      }
    }
    printf(")\n");
  }

  printf("\nPer-kind caches:\n");
  const GdtCpusCoreKind kinds[] = {
      GDT_CPUS_CORE_KIND_PERFORMANCE,
      GDT_CPUS_CORE_KIND_EFFICIENCY,
      GDT_CPUS_CORE_KIND_LP_EFFICIENCY,
  };
  for (size_t k = 0; k < sizeof(kinds) / sizeof(kinds[0]); ++k)
  {
    uint64_t cores_of_kind = 0;
    if (gdt_cpus_num_cores_of_kind(kinds[k], &cores_of_kind) != GDT_CPUS_ERROR_CODE_SUCCESS ||
        cores_of_kind == 0)
    {
      continue;
    }

    GdtCpusCacheInfo l1d = {0}, l1i = {0}, l2 = {0};
    gdt_cpus_get_l1d_cache(kinds[k], &l1d);
    gdt_cpus_get_l1i_cache(kinds[k], &l1i);
    gdt_cpus_get_l2_cache(kinds[k], &l2);
    printf("  %s (%" PRIu64 " cores): L1d %" PRIu64 " KB / L1i %" PRIu64 " KB / L2 %" PRIu64
           " KB (L2 shared by %u threads)\n",
           gdt_cpus_core_kind_description(kinds[k]), cores_of_kind,
           l1d.size_bytes / 1024, l1i.size_bytes / 1024, l2.size_bytes / 1024, l2.shared_by);
  }

  printf("\nLogical processors:\n");
  for (uint64_t i = 0; i < info.lp_count; ++i)
  {
    GdtCpusLp lp = {0};
    res = gdt_cpus_get_lp(i, &lp);
    if (res != GDT_CPUS_ERROR_CODE_SUCCESS)
    {
      printf("Error retrieving LP %" PRIu64 ": %s\n", i, gdt_cpus_error_code_description(res));
      return 1;
    }
    printf("  lp %3u: core %3u smt %u socket %u ", lp.os_id, lp.core, lp.smt_index, lp.socket);
    if (lp.l3_domain == GDT_CPUS_NO_L3)
    {
      printf("l3-domain - ");
    }
    else
    {
      printf("l3-domain %u ", lp.l3_domain);
    }
    printf("numa %u perf %4u kind %s", lp.numa_node, lp.perf_hint,
           gdt_cpus_core_kind_description(lp.kind));
    /* Raw ARM MIDR part; 0 on x86 (no such field). Lets you tell cores of
       different microarchitectures apart -- e.g. A720 (0x0d81) vs A520
       (0x0d80) on a big.LITTLE chip -- without a name table. */
    if (lp.cpu_part != 0)
    {
      printf(" part 0x%04x", lp.cpu_part);
    }
    printf("\n");
  }

  return 0;
}
