<script setup lang="ts">
import { computed, ref } from 'vue'
import { storeToRefs } from 'pinia'
import DescriptionCard from '@/components/console/DescriptionCard.vue'
import TimeLogDialog from '@/components/console/TimeLogDialog.vue'
import StepsCard from '@/components/console/StepsCard.vue'
import TimerCard from '@/components/console/TimerCard.vue'
import WrapupCard from '@/components/console/WrapupCard.vue'
import { useConsoleStore } from '@/stores/console.store'
import { useAsyncAction } from '@/composables/useAsyncAction'
import { excerpt } from '@/common/format/excerpt'

/**
 * The notes on the selected task, with its timer, its description, its checklist and its wrapup
 * pinned above them.
 *
 * The timer leads because it is the one thing here that is live rather than written: everything
 * below it describes the task, this counts while you read it. Then the three questions in the
 * order they are asked, what is this task, what is left of it and where did it get to, with the
 * notes as the background to all three. Each is one row, always present, even absent: an empty
 * row is the reason to write the thing, and every task has a timer the moment it exists.
 */
const store = useConsoleStore()
const {
  selectedTask,
  selectedDocId,
  taskDocuments,
  paneFocus,
  selectedTaskSteps,
  selectedWrapup,
  wrapupIsBehind,
  wrapupMissesSteps,
  selectedTaskEntries,
  runningEntries,
  isLoading
} = storeToRefs(store)
const { run } = useAsyncAction()

const showTimeLog = ref(false)
const isRunningHere = computed(() =>
  runningEntries.value.some((entry) => entry.taskId === selectedTask.value?.id)
)

async function startTimer(): Promise<void> {
  if (!selectedTask.value) return
  await run(() => store.startTimer(selectedTask.value!.id))
}

async function pauseTimer(): Promise<void> {
  if (!selectedTask.value) return
  await run(() => store.pauseTimer(selectedTask.value!.id))
}
</script>

<template>
  <section
    class="flex min-h-0 w-(--spacing-notelist) shrink-0 flex-col border-r border-border bg-surface"
    aria-label="Notes on the selected task"
  >
    <header class="flex h-(--spacing-header) shrink-0 items-center gap-2 border-b border-border px-3.5">
      <span class="min-w-0 flex-1">
        <span class="block truncate text-[13.5px] font-semibold text-text">
          {{ selectedTask?.title ?? 'Notes' }}
        </span>
        <span
          v-if="selectedTask"
          class="block truncate font-mono text-[10.5px] text-anchor/80"
          :title="selectedTask.anchor"
        >
          {{ selectedTask.anchor }}
        </span>
      </span>
      <span v-if="taskDocuments.length" class="font-mono text-[11px] text-text-subtle">
        {{ taskDocuments.length }}
      </span>
    </header>

    <div class="min-h-0 flex-1 overflow-y-auto p-2">
      <div v-if="isLoading" class="flex flex-col gap-2" aria-hidden="true">
        <div class="skeleton h-14 rounded-[var(--radius-control)]" />
        <div v-for="row in 3" :key="row" class="skeleton h-16 rounded-[var(--radius-control)]" />
      </div>

      <p v-else-if="!selectedTask" class="px-2 py-2 text-[12.5px] leading-relaxed text-text-subtle">
        Pick a task to see its notes.
      </p>

      <template v-else>
        <TimerCard
          :entries="selectedTaskEntries"
          :is-running="isRunningHere"
          @start="startTimer"
          @pause="pauseTimer"
          @open-log="showTimeLog = true"
        />
        <DescriptionCard
          :description="selectedTask.description"
          :selected="paneFocus === 'description'"
          @open="store.openDescription()"
        />
        <StepsCard
          :steps="selectedTaskSteps"
          :selected="paneFocus === 'steps'"
          @open="store.openSteps()"
        />
        <WrapupCard
          :wrapup="selectedWrapup"
          :selected="paneFocus === 'wrapup'"
          :behind="wrapupIsBehind"
          :misses-steps="wrapupMissesSteps"
          @open="store.openWrapup()"
        />

        <p
          v-if="!taskDocuments.length"
          class="px-2 py-2 text-[12.5px] leading-relaxed text-text-subtle"
        >
          No note on this task yet. Press
          <kbd class="rounded border border-border px-1 font-mono text-[10px]">N</kbd>
          to write the first one.
        </p>
      </template>

      <button
        v-for="document in taskDocuments"
        :key="document.id"
        data-testid="note-card"
        class="focus-ring mb-1 block w-full rounded-[var(--radius-control)] border p-2.5 text-left transition-all"
        :class="
          document.id === selectedDocId && paneFocus === 'note'
            ? 'border-accent bg-surface-raised shadow-lift'
            : 'border-transparent hover:border-border-strong hover:bg-surface-raised'
        "
        :aria-current="document.id === selectedDocId && paneFocus === 'note'"
        @click="store.selectDocument(document.id)"
      >
        <span class="flex items-center gap-2">
          <span class="min-w-0 flex-1 truncate text-[13px] font-medium text-text">
            {{ document.title }}
          </span>
          <span
            class="shrink-0 rounded border border-border bg-surface-raised px-1.5 py-px font-mono text-[9.5px] text-text-muted"
          >
            {{ document.kind }}
          </span>
          <span
            v-if="document.tasks.length > 1"
            class="anchor-chip shrink-0 px-1.5 py-px text-[9.5px]"
            :title="`On ${document.tasks.length} tasks`"
          >
            &#8942; {{ document.tasks.length }}
          </span>
        </span>
        <span class="mt-1 line-clamp-2 block text-[11.5px] leading-relaxed text-text-subtle">
          {{ excerpt(document.bodyMarkdown) }}
        </span>
      </button>
    </div>
  </section>

  <TimeLogDialog
    v-if="showTimeLog && selectedTask"
    :task="selectedTask"
    :entries="selectedTaskEntries"
    @close="showTimeLog = false"
  />
</template>
