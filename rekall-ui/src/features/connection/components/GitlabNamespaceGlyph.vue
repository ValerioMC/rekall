<script setup lang="ts">
import type { GitlabNamespaceKind } from '@/features/connection/model/connection'

defineProps<{ kind: GitlabNamespaceKind }>()
</script>

<template>
  <svg class="size-3.5 shrink-0" viewBox="0 0 16 16" fill="none" stroke="currentColor" stroke-width="1.4" aria-hidden="true">
    <path v-if="kind === 'GROUP'" d="M2 4.5A1.5 1.5 0 0 1 3.5 3H6l1.5 1.5h5A1.5 1.5 0 0 1 14 6v5.5a1.5 1.5 0 0 1-1.5 1.5h-9A1.5 1.5 0 0 1 2 11.5z" />
    <template v-else>
      <path d="M4 2.5h7A1.5 1.5 0 0 1 12.5 4v9.5H5A1.5 1.5 0 0 1 3.5 12V3A.5.5 0 0 1 4 2.5Z" />
      <path d="M3.5 12A1.5 1.5 0 0 1 5 10.5h7.5" />
    </template>
  </svg>
</template>
