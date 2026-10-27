<script setup lang="ts">
import AppCard from '@/components/ui/AppCard.vue'
import { relativeTime } from '@/common/format/relative-time'
import { EPIC_ACTIVITY_LABEL } from '@/features/catalog/model/epic'
import type { EpicActivity, EpicActivityKind } from '@/features/catalog/model/epic'
import type { Task } from '@/features/catalog/model/catalog'
import type { TaskId } from '@/model/branded'

/** What moved lately on the epic's tasks, newest first, as a rail of moments. */
const props = defineProps<{ activity: readonly EpicActivity[]; children: readonly Task[]; loaded: boolean }>()

defineEmits<{ open: [id: TaskId] }>()

const KIND_DOT: Readonly<Record<EpicActivityKind, string>> = {
  STEP_STARTED: 'bg-anchor',
  STEP_CLAIMED: 'bg-accent',
  STEP_ACCEPTED: 'bg-safe',
  TASK_CLAIMED: 'bg-accent',
  TASK_ACCEPTED: 'bg-safe',
  WRAPUP_WRITTEN: 'bg-text-muted',
  COMMIT_LOGGED: 'bg-epic'
}

function titleOf(taskId: TaskId): string {
  return props.children.find((task) => task.id === taskId)?.title ?? 'A task'
}
</script>

<template>
  <AppCard :padded="false" data-testid="epic-activity">
    <header class="border-b border-border px-5 py-3">
      <span class="eyebrow text-caption">Activity</span>
    </header>

    <div v-if="!loaded" class="flex flex-col gap-2 p-5" aria-hidden="true">
      <div v-for="row in 4" :key="row" class="skeleton h-9 rounded-control" />
    </div>

    <p v-else-if="!activity.length" class="px-5 py-6 text-small leading-relaxed text-text-subtle">
      Nothing has moved on its tasks yet. Steps started, claims, accepted work, wrapups and commits show up here.
    </p>

    <ol v-else class="relative px-5 py-3">
      <li
        v-for="(entry, index) in activity"
        :key="`${entry.taskId}-${entry.kind}-${entry.at}-${index}`"
        class="relative flex gap-3 pb-3 last:pb-0"
      >
        <span class="relative flex w-2 shrink-0 justify-center" aria-hidden="true">
          <span class="mt-[5px] size-2 rounded-full ring-2 ring-surface" :class="KIND_DOT[entry.kind]" />
          <span v-if="index < activity.length - 1" class="absolute bottom-[-6px] top-4 w-px bg-border" />
        </span>
        <span class="min-w-0 flex-1">
          <span class="flex items-baseline gap-2">
            <span class="text-caption font-semibold text-text-muted">{{ EPIC_ACTIVITY_LABEL[entry.kind] }}</span>
            <time class="ml-auto shrink-0 text-micro text-text-subtle" :datetime="entry.at">{{ relativeTime(entry.at) }}</time>
          </span>
          <button
            class="focus-ring block max-w-full truncate rounded-key text-left text-small text-text transition-colors hover:text-accent"
            @click="$emit('open', entry.taskId)"
          >
            {{ titleOf(entry.taskId) }}
          </button>
          <span v-if="entry.subject" class="block truncate text-caption text-text-subtle">{{ entry.subject }}</span>
        </span>
      </li>
    </ol>
  </AppCard>
</template>
