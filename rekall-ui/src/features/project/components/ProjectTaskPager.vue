<script setup lang="ts">
import { computed } from 'vue'
import { storeToRefs } from 'pinia'
import AppButton from '@/components/ui/AppButton.vue'
import SegmentedControl, { type SegmentOption } from '@/components/ui/SegmentedControl.vue'
import { pageNumbers } from '@/features/project/lib/task-pagination'
import { TASK_PAGE_SIZES, type TaskPageSize } from '@/features/project/model/project-task-list'
import { useProjectTasksStore } from '@/features/project/store/projectTasks.store'

/** Walks the pages of the task list: the tasks this page lists out of how many, how many a page holds, and the pages. */
const SIZE_OPTIONS: readonly SegmentOption<`${TaskPageSize}`>[] = TASK_PAGE_SIZES.map((size) => ({ value: `${size}`, label: `${size}` }))

const projectTasks = useProjectTasksStore()
const { paging, pageSize } = storeToRefs(projectTasks)

const size = computed({
  get: (): `${TaskPageSize}` => `${pageSize.value}`,
  set: (chosen) => (pageSize.value = Number(chosen) as TaskPageSize)
})
const numbers = computed(() => pageNumbers(paging.value.page, paging.value.pageCount))
</script>

<template>
  <nav
    class="sticky bottom-0 z-(--z-sticky) mt-auto flex h-13 items-center justify-end gap-5 whitespace-nowrap border-t border-border bg-canvas pl-56 pr-3 text-small text-text-muted"
    aria-label="Pages of tasks"
    data-testid="task-pager"
  >
    <span class="mr-auto tabular-nums" data-testid="task-pager-range">{{ paging.from }}–{{ paging.to }} of {{ paging.total }} tasks</span>
    <span class="flex items-center gap-2" data-testid="task-pager-size">
      Rows per page
      <SegmentedControl v-model="size" :options="SIZE_OPTIONS" label="Rows per page" size="sm" />
    </span>
    <span v-if="paging.pageCount > 1" class="flex items-center gap-1">
      <AppButton
        variant="ghost"
        size="icon"
        class="!size-7"
        :disabled="paging.page === 1"
        aria-label="Previous page"
        data-testid="task-pager-previous"
        @click="projectTasks.goToPage(paging.page - 1)"
      >
        <svg class="size-3" viewBox="0 0 12 12" fill="none" aria-hidden="true">
          <path d="M7.5 2.5 4 6l3.5 3.5" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round" />
        </svg>
      </AppButton>
      <template v-for="(number, index) in numbers" :key="number ?? `gap-${index}`">
        <span v-if="number === null" class="grid size-7 place-items-center text-text-subtle" aria-hidden="true">…</span>
        <button
          v-else
          type="button"
          class="focus-ring grid h-7 min-w-7 cursor-pointer place-items-center rounded-control border px-1.5 tabular-nums"
          :class="number === paging.page ? 'border-accent text-text' : 'border-transparent hover:bg-surface-raised hover:text-text'"
          :aria-current="number === paging.page ? 'page' : undefined"
          :aria-label="`Page ${number}`"
          data-testid="task-pager-page"
          @click="projectTasks.goToPage(number)"
        >
          {{ number }}
        </button>
      </template>
      <AppButton
        variant="ghost"
        size="icon"
        class="!size-7"
        :disabled="paging.page === paging.pageCount"
        aria-label="Next page"
        data-testid="task-pager-next"
        @click="projectTasks.goToPage(paging.page + 1)"
      >
        <svg class="size-3" viewBox="0 0 12 12" fill="none" aria-hidden="true">
          <path d="M4.5 2.5 8 6 4.5 9.5" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round" />
        </svg>
      </AppButton>
    </span>
  </nav>
</template>
