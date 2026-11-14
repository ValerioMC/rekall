<script setup lang="ts">
import type { ChangeTotals, RepositoryRef } from '@/features/commits/lib/change-sources'

/** Opens a repository's files in the column when the task changed more than one repository. */
defineProps<{ repository: RepositoryRef; totals: ChangeTotals }>()
</script>

<template>
  <div
    class="mx-6 mt-7 flex items-center gap-2.5 border-b border-border pb-2"
    role="heading"
    aria-level="3"
    data-testid="change-repository-band"
  >
    <svg class="size-3.5 shrink-0 text-text-subtle" viewBox="0 0 14 14" fill="none" aria-hidden="true">
      <path
        d="M3 2.5h6.2A1.8 1.8 0 0 1 11 4.3v7.2H4.8A1.8 1.8 0 0 1 3 9.7V2.5Z"
        stroke="currentColor"
        stroke-width="1.2"
        stroke-linejoin="round"
      />
      <path d="M3 9.7a1.8 1.8 0 0 1 1.8-1.8H11" stroke="currentColor" stroke-width="1.2" />
    </svg>
    <span class="min-w-0 truncate text-body font-semibold text-text">{{ repository.name }}</span>
    <span class="shrink-0 text-small text-text-muted">
      {{ totals.files }} {{ totals.files === 1 ? 'file' : 'files'
      }}<template v-if="totals.commits > 0"
        >, {{ totals.commits }} {{ totals.commits === 1 ? 'commit' : 'commits' }}</template
      >
    </span>
    <span class="ml-auto shrink-0 font-mono text-caption tabular-nums"
      ><span class="text-safe">+{{ totals.additions }}</span>
      <span class="text-danger">−{{ totals.deletions }}</span></span
    >
  </div>
</template>
