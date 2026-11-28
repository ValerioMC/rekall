<script setup lang="ts">
import { RouterLink } from 'vue-router'
import AppBadge from '@/components/ui/AppBadge.vue'
import EpicGlyph from '@/features/catalog/components/epic/EpicGlyph.vue'
import type { Task } from '@/features/catalog/model/catalog'

/** The title cell of a task row: the title is the link that opens the task. */
defineProps<{
  task: Task
  filed: boolean
  isEpicRow: boolean
  nested?: boolean
  /** The epic of a task whose epic is not listed with it. */
  epicTitle?: string | null
}>()
</script>

<template>
  <span class="flex min-w-0 items-center gap-2" :class="[filed ? 'font-normal' : 'font-medium text-text', nested && 'pl-3']">
    <EpicGlyph v-if="isEpicRow" :size="15" />
    <RouterLink
      :to="`/task/${task.id}/description`"
      class="focus-ring min-w-0 truncate rounded-key decoration-accent underline-offset-4 hover:underline"
      :class="[filed && 'text-text-muted', isEpicRow && 'font-semibold']"
      :title="`Open ${task.title}`"
      data-testid="task-row-title"
      @click.stop
      @keydown.enter.stop
    >
      {{ task.title }}
    </RouterLink>
    <span
      v-if="epicTitle"
      class="inline-flex min-w-0 max-w-[40%] shrink items-center gap-1 rounded-key bg-epic-soft px-1.5 py-px text-caption text-epic"
      data-testid="task-row-epic"
      :title="`In the epic ${epicTitle}`"
    >
      <EpicGlyph :size="11" />
      <span class="truncate">{{ epicTitle }}</span>
    </span>
    <AppBadge v-if="task.quick" class="shrink-0" tone="neutral" data-testid="task-row-quick">Quick</AppBadge>
  </span>
</template>
