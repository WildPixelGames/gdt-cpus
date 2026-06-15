// Thread-priority introspection over the C ABI: what can this machine deliver,
// and what did each request actually do? Mirrors the Rust `thread_priorities`
// example. The headline is GdtCpusAppliedPriority: a "successful" set on Linux
// can still mean you silently landed on Normal, so read the out-param, not just
// the return code.

#include <pthread.h>
#include <stdint.h>
#include <stdio.h>

#include "gdt_cpus/gdt_cpus.h"

static const char *prio(GdtCpusThreadPriority p)
{
  return gdt_cpus_thread_priority_description(p);
}

// Each level is applied on its OWN fresh thread (starting at nice 0). Setting
// them in sequence on one thread would route everything after the first through
// the broker - once a thread sits at a high nice, lowering it back is denied
// directly (the RLIMIT_NICE floor) - which would misreport the per-level grant.
typedef struct
{
  GdtCpusThreadPriority level;
  int32_t rc;
  GdtCpusAppliedPriority applied;
} SetTask;

static void *set_on_thread(void *arg)
{
  SetTask *t = (SetTask *)arg;
  t->rc = gdt_cpus_set_thread_priority(t->level, &t->applied);
  return NULL;
}

typedef struct
{
  int32_t promote_rc;
  GdtCpusAppliedPriority promoted;
  int32_t demote_rc;
} RtTask;

static void *promote_on_thread(void *arg)
{
  RtTask *t = (RtTask *)arg;
  t->promote_rc = gdt_cpus_promote_thread_to_realtime(1000 /* us budget */, &t->promoted);
  if (t->promote_rc == GDT_CPUS_ERROR_CODE_SUCCESS)
  {
    t->demote_rc = gdt_cpus_demote_thread_from_realtime();
  }
  return NULL;
}

int main(void)
{
  // 1. Pre-flight prediction - touches no thread, costs microseconds.
  GdtCpusPriorityCaps caps = {0};
  if (gdt_cpus_priority_capabilities(&caps) == GDT_CPUS_ERROR_CODE_SUCCESS)
  {
    printf("priority_capabilities: %u of 7 levels distinct; ranks [", caps.distinct_levels);
    for (int i = 0; i < 7; ++i)
    {
      printf("%s%u", i ? ", " : "", caps.effective_rank[i]);
    }
    printf("]\n");
  }

  // 2. Observed outcome - set each level on a fresh thread, report what stuck.
  printf("\nset_thread_priority - what each request actually does:\n");
  const GdtCpusThreadPriority levels[] = {
      GDT_CPUS_THREAD_PRIORITY_BACKGROUND,
      GDT_CPUS_THREAD_PRIORITY_LOWEST,
      GDT_CPUS_THREAD_PRIORITY_BELOW_NORMAL,
      GDT_CPUS_THREAD_PRIORITY_NORMAL,
      GDT_CPUS_THREAD_PRIORITY_ABOVE_NORMAL,
      GDT_CPUS_THREAD_PRIORITY_HIGHEST,
      GDT_CPUS_THREAD_PRIORITY_TIME_CRITICAL,
  };
  for (size_t i = 0; i < sizeof(levels) / sizeof(levels[0]); ++i)
  {
    SetTask task = {.level = levels[i]};
    pthread_t th;
    pthread_create(&th, NULL, set_on_thread, &task);
    pthread_join(th, NULL);

    if (task.rc != GDT_CPUS_ERROR_CODE_SUCCESS)
    {
      printf("  %-13s error: %s\n", prio(levels[i]), gdt_cpus_error_code_description(task.rc));
      continue;
    }
    printf("  requested %-13s effective %-13s grant %-9s", prio(task.applied.requested),
           prio(task.applied.effective), gdt_cpus_grant_description(task.applied.grant));
    if (task.applied.reason != GDT_CPUS_FALLBACK_REASON_NONE)
    {
      printf("  (fell short: %s", gdt_cpus_fallback_reason_description(task.applied.reason));
      if (task.applied.broker_error != GDT_CPUS_BROKER_ERROR_NONE)
      {
        printf(": %s", gdt_cpus_broker_error_description(task.applied.broker_error));
      }
      printf(")");
    }
    printf("\n");
  }

  // 3. The explicit real-time opt-in (consent API). Denial is an error code,
  //    never a silent degrade.
  printf("\npromote_thread_to_realtime - the consent API:\n");
  RtTask rt = {0};
  pthread_t rt_th;
  pthread_create(&rt_th, NULL, promote_on_thread, &rt);
  pthread_join(rt_th, NULL);
  if (rt.promote_rc == GDT_CPUS_ERROR_CODE_SUCCESS)
  {
    printf("  promoted: grant %s, effective %s\n", gdt_cpus_grant_description(rt.promoted.grant),
           prio(rt.promoted.effective));
    if (rt.demote_rc == GDT_CPUS_ERROR_CODE_SUCCESS)
    {
      printf("  demoted: back to normal scheduling\n");
    }
  }
  else
  {
    printf("  denied: %s\n", gdt_cpus_error_code_description(rt.promote_rc));
  }

  return 0;
}
