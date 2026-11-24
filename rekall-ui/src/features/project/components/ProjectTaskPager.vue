<script setup lang="ts">
import AppButton from '@/components/ui/AppButton.vue'
import type { PageOfRows } from '@/features/project/lib/task-pagination'

/** Walks the pages of the ledger: which tasks this page lists out of how many, and the keys to the next and the previous one. */
defineProps<{ paging: PageOfRows<unknown> }>()
const emit = defineEmits<{ go: [page: number] }>()
</script>

<template>
  <nav
    v-if="paging.pageCount > 1"
    class="flex h-12 items-center gap-3 border-t border-border px-3 text-small text-text-muted"
    aria-label="Pages of tasks"
    data-testid="task-pager"
  >
    <span class="tabular-nums" data-testid="task-pager-range">{{ paging.from }}–{{ paging.to }} of {{ paging.total }} tasks</span>
    <span class="flex-1" />
    <span class="tabular-nums text-text-subtle" data-testid="task-pager-page">Page {{ paging.page }} of {{ paging.pageCount }}</span>
    <AppButton
      variant="secondary"
      size="sm"
      :disabled="paging.page === 1"
      aria-label="Previous page"
      data-testid="task-pager-previous"
      @click="emit('go', paging.page - 1)"
    >
      Previous
    </AppButton>
    <AppButton
      variant="secondary"
      size="sm"
      :disabled="paging.page === paging.pageCount"
      aria-label="Next page"
      data-testid="task-pager-next"
      @click="emit('go', paging.page + 1)"
    >
      Next
    </AppButton>
  </nav>
</template>
