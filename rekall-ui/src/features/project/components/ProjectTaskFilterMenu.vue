<script setup lang="ts">
import { computed, ref } from 'vue'
import AppButton from '@/components/ui/AppButton.vue'
import DropPanel from '@/components/ui/DropPanel.vue'
import { TASK_FILTER_LABEL, type TaskFilterKey } from '@/features/project/model/project-task-list'
import { useProjectTasksStore } from '@/features/project/store/projectTasks.store'

/** One filter of the task list: its values with how many tasks hold each, any number of them ticked. */
const props = defineProps<{ filterKey: TaskFilterKey }>()

const projectTasks = useProjectTasksStore()
const isOpen = ref(false)

const label = computed(() => TASK_FILTER_LABEL[props.filterKey])
const picked = computed(() => projectTasks.filters[props.filterKey])
const options = computed(() => (isOpen.value ? projectTasks.optionsOf(props.filterKey) : []))
const summary = computed(() => {
  if (picked.value.length === 0) return ''
  const first = projectTasks.optionsOf(props.filterKey).find((option) => option.value === picked.value[0])
  const rest = picked.value.length > 1 ? ` +${picked.value.length - 1}` : ''
  return `${first?.label ?? ''}${rest}`
})
</script>

<template>
  <DropPanel v-model:open="isOpen" :label="`Filter by ${label.toLowerCase()}`" :data-testid="`task-filter-${filterKey}`">
    <template #trigger="{ open, toggle }">
      <AppButton
        variant="ghost"
        class="!border"
        :class="picked.length > 0 ? '!border-border-field bg-surface-raised !text-text' : '!border-border'"
        aria-haspopup="true"
        :aria-expanded="open"
        data-testid="task-filter-trigger"
        @click="toggle"
      >
        {{ label }}
        <span v-if="summary" class="max-w-36 truncate font-semibold" data-testid="task-filter-summary">{{ summary }}</span>
        <svg class="size-3" viewBox="0 0 12 12" fill="none" aria-hidden="true">
          <path d="m3 4.5 3 3 3-3" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round" />
        </svg>
      </AppButton>
    </template>

    <div class="flex max-h-80 min-w-56 flex-col gap-0.5 overflow-y-auto p-1.5">
      <label
        v-for="option in options"
        :key="option.value"
        class="flex h-(--spacing-control) shrink-0 cursor-pointer items-center gap-2.5 rounded-control px-2 text-small text-text hover:bg-surface-hover"
        data-testid="task-filter-option"
      >
        <input
          type="checkbox"
          class="focus-ring size-3.5 accent-(--color-accent)"
          :checked="picked.includes(option.value)"
          :data-testid="`task-filter-value-${option.value}`"
          @change="projectTasks.toggleFilter(filterKey, option.value)"
        />
        <span class="max-w-72 truncate">{{ option.label }}</span>
        <span class="ml-auto pl-4 text-caption tabular-nums text-text-subtle">{{ option.count }}</span>
      </label>
      <p v-if="options.length === 0" class="px-2 py-1.5 text-small text-text-subtle">Nothing to filter in this view.</p>
      <div v-if="picked.length > 0" class="mt-1 flex justify-end border-t border-border pt-1.5">
        <AppButton variant="ghost" size="sm" data-testid="task-filter-reset" @click="projectTasks.resetFilter(filterKey)">Reset</AppButton>
      </div>
    </div>
  </DropPanel>
</template>
