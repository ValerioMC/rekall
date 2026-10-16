<script setup lang="ts">
/**
 * The tasks a note sits on, under its header. The task in view is a chip of its own; the note's
 * other open tasks fold behind a count once there are two, and finished ones always do.
 */
import { computed, ref, watch } from 'vue'
import { storeToRefs } from 'pinia'
import CloseGlyph from '@/components/ui/CloseGlyph.vue'
import { useConsoleStore } from '@/stores/console.store'
import { useAsyncAction } from '@/composables/useAsyncAction'
import type { RekallDocument } from '@/features/catalog/model/catalog'
import type { TaskId } from '@/model/branded'

const props = defineProps<{ note: RekallDocument }>()

defineEmits<{ assign: [] }>()

const store = useConsoleStore()
const { selectedTaskId, tasks: allTasks } = storeToRefs(store)
const { run } = useAsyncAction()

const doneTaskIds = computed(() => {
  const ids = new Set<TaskId>()
  for (const task of allTasks.value) if (task.status === 'DONE') ids.add(task.id)
  return ids
})

const liveTasks = computed(() => props.note.tasks.filter((task) => !doneTaskIds.value.has(task.id)))
const doneTasks = computed(() => props.note.tasks.filter((task) => doneTaskIds.value.has(task.id)))
const showDoneTasks = ref(false)

const liveInView = computed(() => liveTasks.value.filter((task) => task.id === selectedTaskId.value))
const otherLiveTasks = computed(() => liveTasks.value.filter((task) => task.id !== selectedTaskId.value))
const foldsOtherLive = computed(() => otherLiveTasks.value.length > 1)
const showOtherLiveTasks = ref(false)
const shownLiveTasks = computed(() =>
  !foldsOtherLive.value || showOtherLiveTasks.value ? [...liveInView.value, ...otherLiveTasks.value] : liveInView.value
)

const alsoOn = computed(() => props.note.tasks.filter((task) => task.id !== selectedTaskId.value))

watch(
  () => props.note.id,
  () => {
    showDoneTasks.value = false
    showOtherLiveTasks.value = false
  }
)

async function detachFrom(taskId: TaskId): Promise<void> {
  const note = props.note
  await run(
    () => store.saveNote(note.id, { taskIds: note.tasks.filter((task) => task.id !== taskId).map((task) => task.id) }),
    'Removed from that task.'
  )
}
</script>

<template>
  <div
    class="flex shrink-0 flex-wrap items-center gap-2 border-b border-border bg-surface px-5 py-2.5"
    data-testid="note-membership"
  >
    <span class="eyebrow">
      On {{ note.tasks.length }}
      task{{ note.tasks.length === 1 ? '' : 's' }}
    </span>

    <span
      v-for="task in shownLiveTasks"
      :key="task.id"
      data-testid="note-task-chip"
      class="inline-flex items-center gap-1.5 rounded-full border py-0.5 pl-2.5 pr-1 font-mono text-caption transition-colors"
      :class="
        task.id === selectedTaskId
          ? 'border-accent bg-accent-soft text-accent'
          : 'border-border-strong bg-surface-raised text-text-muted'
      "
    >
      <button class="focus-ring rounded-key" :title="task.title" @click="store.selectTask(task.id)">
        {{ task.projectLabel }}/{{ task.label }}
      </button>
      <button
        class="focus-ring hit-target relative -m-0.5 grid size-5 place-items-center rounded-full text-text-subtle transition-colors hover:bg-danger-soft hover:text-danger"
        :aria-label="`Remove this note from ${task.title}`"
        @click="detachFrom(task.id)"
      >
        <CloseGlyph small />
      </button>
    </span>

    <button
      v-if="foldsOtherLive"
      class="focus-ring inline-flex items-center gap-1.5 rounded-full border border-border-strong bg-surface-raised py-0.5 pl-2 pr-2.5 font-mono text-caption text-text-muted transition-colors hover:border-border-strong hover:text-text"
      :aria-expanded="showOtherLiveTasks"
      data-testid="note-open-toggle"
      @click="showOtherLiveTasks = !showOtherLiveTasks"
    >
      <svg
        class="size-2.5 transition-transform"
        :class="showOtherLiveTasks && 'rotate-90'"
        viewBox="0 0 24 24"
        fill="none"
        aria-hidden="true"
      >
        <path
          d="M9 6l6 6-6 6"
          stroke="currentColor"
          stroke-width="2.4"
          stroke-linecap="round"
          stroke-linejoin="round"
        />
      </svg>
      {{ otherLiveTasks.length }} {{ liveInView.length ? 'more ' : '' }}open
    </button>

    <button
      v-if="doneTasks.length && liveTasks.length"
      class="focus-ring inline-flex items-center gap-1.5 rounded-full border border-border-strong bg-surface-raised py-0.5 pl-2 pr-2.5 font-mono text-caption text-text-muted transition-colors hover:border-border-strong hover:text-text"
      :aria-expanded="showDoneTasks"
      data-testid="note-done-toggle"
      @click="showDoneTasks = !showDoneTasks"
    >
      <svg
        class="size-2.5 transition-transform"
        :class="showDoneTasks && 'rotate-90'"
        viewBox="0 0 24 24"
        fill="none"
        aria-hidden="true"
      >
        <path
          d="M9 6l6 6-6 6"
          stroke="currentColor"
          stroke-width="2.4"
          stroke-linecap="round"
          stroke-linejoin="round"
        />
      </svg>
      {{ doneTasks.length }} done
    </button>

    <template v-if="showDoneTasks || !liveTasks.length">
      <span
        v-for="task in doneTasks"
        :key="task.id"
        data-testid="note-task-chip-done"
        class="inline-flex items-center gap-1.5 rounded-full border border-border bg-surface py-0.5 pl-2.5 pr-1 font-mono text-caption text-text-subtle transition-colors"
      >
        <span class="size-[6px] shrink-0 rounded-full bg-safe" aria-hidden="true" />
        <button
          class="focus-ring rounded-key"
          :title="`${task.title} (done)`"
          @click="store.selectTask(task.id)"
        >
          {{ task.projectLabel }}/{{ task.label }}
        </button>
        <button
          class="focus-ring hit-target relative -m-0.5 grid size-5 place-items-center rounded-full text-text-subtle transition-colors hover:bg-danger-soft hover:text-danger"
          :aria-label="`Remove this note from ${task.title}`"
          @click="detachFrom(task.id)"
        >
          <CloseGlyph small />
        </button>
      </span>
    </template>

    <button
      class="focus-ring inline-flex items-center gap-1 rounded-full border border-dashed border-border-strong py-0.5 pl-2 pr-2.5 font-mono text-caption text-text-muted transition-colors hover:border-accent hover:text-text"
      data-testid="assign-open"
      @click="$emit('assign')"
    >
      <span class="text-accent">+</span> task
    </button>
    <span v-if="alsoOn.length" class="w-full text-caption text-text-subtle">
      Editing here changes what
      {{ alsoOn.length === 1 ? 'that other task' : 'those other tasks' }} load too.
    </span>
  </div>
</template>
