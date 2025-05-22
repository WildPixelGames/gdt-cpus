#include <stdio.h>
#include <stdlib.h>
#include <stdbool.h>

#include "gdt_cpus/gdt_cpus.h"

int main()
{
  GdtCpusCpuInfo info = {};
  int32_t res = gdt_cpus_cpu_info(&info);

  if (res != GDT_CPUS_ERROR_CODE_SUCCESS)
  {
    printf("Error retrieving CPU info: %s\n", gdt_cpus_error_code_description(res));
    return 1;
  }

  bool is_hybrid = false;
  res = gdt_cpus_is_hybrid(&is_hybrid);

  // Should always succeed if the first call to gdt_cpus_cpu_info() succeeded
  if (res != GDT_CPUS_ERROR_CODE_SUCCESS)
  {
    printf("Error checking hybrid architecture: %s\n", gdt_cpus_error_code_description(res));
    return 1;
  }

  printf("CPU Information:\n");
  printf("---------------\n");
  printf("Vendor: %s\n", info.vendor_name);
  printf("Model: %s\n", info.model_name);
  printf("Physical cores: %lld\n", info.total_physical_cores);
  printf("Logical cores: %lld\n", info.total_logical_processors);
  printf("Performance cores: %lld\n", info.total_performance_cores);
  printf("Efficiency cores: %lld\n", info.total_efficiency_cores);
  printf("Hybrid architecture: %s\n", is_hybrid ? "Yes" : "No");

  for (uint64_t socket_id = 0; socket_id < info.sockets_count; ++socket_id)
  {
    GdtCpusSocketInfo socket = {};
    res = gdt_cpus_get_socket_info(socket_id, &socket);

    if (res != GDT_CPUS_ERROR_CODE_SUCCESS)
    {
      printf("Error retrieving socket info: %s\n", gdt_cpus_error_code_description(res));
      return 1;
    }

    printf("\nProcessor #%lld (Socket ID: %lld)\n", socket_id, socket.id);

    if (socket.has_l3_cache)
    {
      GdtCpusCacheInfo l3_cache = {};
      res = gdt_cpus_get_l3_cache_info(socket_id, &l3_cache);

      if (res != GDT_CPUS_ERROR_CODE_SUCCESS)
      {
        printf("Error retrieving L3 cache info: %s\n", gdt_cpus_error_code_description(res));
        return 1;
      }

      printf("  L3 Cache: %lld KB\n", l3_cache.size_bytes / 1024);
    }

    printf("  Cores:\n");
    for (uint64_t core_id = 0; core_id < socket.cores_count; ++core_id)
    {
      GdtCpusCoreInfo core = {};
      res = gdt_cpus_get_core_info(socket_id, core_id, &core);

      if (res != GDT_CPUS_ERROR_CODE_SUCCESS)
      {
        printf("Error retrieving core info: %s\n", gdt_cpus_error_code_description(res));
        return 1;
      }

      printf(
          "    Core #%lld: %s core with %lld threads\n",
          core.id,
          gdt_cpus_core_type_description(core.core_type),
          core.logical_processor_ids_count);

      if (core.has_l1_instruction_cache)
      {
        GdtCpusCacheInfo l1i_cache = {};
        res = gdt_cpus_get_l1i_cache_info(socket_id, core_id, &l1i_cache);

        if (res != GDT_CPUS_ERROR_CODE_SUCCESS)
        {
          printf("Error retrieving L1i Cache info: %s\n", gdt_cpus_error_code_description(res));
          return 1;
        }

        printf("      L1i Cache: %lld KB\n", l1i_cache.size_bytes / 1024);
      }
      else
      {
        printf("      L1i Cache: Not available\n");
      }

      if (core.has_l1_data_cache)
      {
        GdtCpusCacheInfo l1d_cache = {};
        res = gdt_cpus_get_l1d_cache_info(socket_id, core_id, &l1d_cache);

        if (res != GDT_CPUS_ERROR_CODE_SUCCESS)
        {
          printf("Error retrieving L1d Cache info: %s\n", gdt_cpus_error_code_description(res));
          return 1;
        }

        printf("      L1d Cache: %lld KB\n", l1d_cache.size_bytes / 1024);
      }
      else
      {
        printf("      L1d Cache: Not available\n");
      }

      if (core.has_l2_cache)
      {
        GdtCpusCacheInfo l2_cache = {};
        res = gdt_cpus_get_l2_cache_info(socket_id, core_id, &l2_cache);

        if (res != GDT_CPUS_ERROR_CODE_SUCCESS)
        {
          printf("Error retrieving L2 Cache info: %s\n", gdt_cpus_error_code_description(res));
          return 1;
        }

        printf("      L2 Cache: %lld KB\n", l2_cache.size_bytes / 1024);
      }
      else
      {
        printf("      L2 Cache: Not available\n");
      }
    }
  }

  uint32_t features = 0;
  res = gdt_cpus_get_features(&features);

  if (res != GDT_CPUS_ERROR_CODE_SUCCESS)
  {
    printf("Error retrieving CPU features: %s\n", gdt_cpus_error_code_description(res));
    return 1;
  }

// Example of checking for a feature (adjust feature name based on your arch)
#if defined(GDT_CPUS_ARCH_X86_64)
  if ((features & GDT_CPUS_CPU_FEATURES_SSE2) == GDT_CPUS_CPU_FEATURES_SSE2)
  {
    printf("SSE2 Supported: Yes\n");
  }
  else
  {
    printf("SSE2 Supported: No\n");
  }
#elif defined(GDT_CPUS_ARCH_AARCH64)
  if ((features & GDT_CPUS_CPU_FEATURES_NEON) == GDT_CPUS_CPU_FEATURES_NEON)
  {
    printf("NEON Supported: Yes\n");
  }
  else
  {
    printf("NEON Supported: No\n");
  }
#endif

  const char *desc_success = gdt_cpus_error_code_description(GDT_CPUS_ERROR_CODE_SUCCESS);
  printf("Error code 0 means: %s\n", desc_success);

  res = gdt_cpus_pin_thread_to_core(0);

  if (res != GDT_CPUS_ERROR_CODE_SUCCESS)
  {
    printf("Error pinning thread to core: %s\n", gdt_cpus_error_code_description(res));
  }
  else
  {
    printf("Thread pinned to core 0.\n");
  }

  res = gdt_cpus_set_thread_priority(GDT_CPUS_THREAD_PRIORITY_HIGHEST);

  if (res != GDT_CPUS_ERROR_CODE_SUCCESS)
  {
    printf("Error setting thread priority: %s\n", gdt_cpus_error_code_description(res));
  }
  else
  {
    printf("Thread priority set to GDT_CPUS_THREAD_PRIORITY_HIGHEST.\n");
  }

  return 0;
}
