<script setup lang="ts">
import { computed, ref, watch } from 'vue'
import { storeToRefs } from 'pinia'
import AppButton from '@/components/ui/AppButton.vue'
import AppConfirm from '@/components/ui/AppConfirm.vue'
import AppMarkdownEditor from '@/components/ui/AppMarkdownEditor.vue'
import { useConsoleStore } from '@/stores/console.store'
import { useAsyncAction } from '@/composables/useAsyncAction'
import { relativeTime } from '@/common/format/relative-time'
import { rkCommand } from '@/common/format/rk-command'
import { WRAPUP_AUTHOR_LABEL } from '@/model/catalog'

/**
 * The state of the task in view: what its implementation currently is.
 *
 * Deliberately not the note editor with a different title. A note has a name, a kind and any
 * number of tasks it belongs to; a wrapup has none of those, because it is one answer to one
 * question about one task. Every control the note pane has that would be meaningless here is
 * absent rather than disabled.
 *
 * Written by Claude at the end of a session and corrected here. Both write the whole text —
 * there is no merge — so the pane says who wrote what is on screen, and how long ago.
 */
const store = useConsoleStore()
const { selectedTask, selectedWrapup, wrapupIsBehind } = storeToRefs(store)
const { run } = useAsyncAction()

const mode = ref<'write' | 'read'>('read')
const isConfirmingDelete = ref(false)

/**
 * Writing by hand starts as a local draft, not as an empty row.
 *
 * A wrapup with no body is refused by the server, and rightly: a task that claims a state and
 * has nothing to say for it is worse than one that says nothing. So the first character typed
 * is what creates it.
 */
const isDrafting = ref(false)
const draft = ref('')
let saveTimer: ReturnType<typeof setTimeout> | null = null

/**
 * The task is what this pane reloads on, and nothing else.
 *
 * Every write replaces the wrapup in the store with an equal but distinct object, and the
 * first one replaces a null with a row. Reacting to either would reload the editor and flip it
 * back to reading in the middle of a sentence — once on every autosave, and again the moment a
 * hand-written wrapup first saves. The only event that means "this pane is now about something
 * else" is a different task.
 */
watch(
  () => selectedTask.value?.id ?? null,
  () => {
    if (saveTimer) clearTimeout(saveTimer)
    draft.value = selectedWrapup.value?.bodyMarkdown ?? ''
    isDrafting.value = false
    // Reading is the default: this is the pane you open to find out where you left the work,
    // and it is rewritten far less often than it is read.
    mode.value = selectedWrapup.value ? 'read' : 'write'
  },
  { immediate: true }
)

/** The command that has Claude rewrite it, ready to paste next to the terminal. */
const command = computed(() =>
  selectedTask.value ? `${rkCommand(selectedTask.value.anchor)} wrapup` : ''
)

const writtenBy = computed(() =>
  selectedWrapup.value ? WRAPUP_AUTHOR_LABEL[selectedWrapup.value.writtenBy] : ''
)

const copied = ref<'anchor' | 'command' | null>(null)

async function copy(what: 'anchor' | 'command', text: string): Promise<void> {
  await navigator.clipboard?.writeText(text)
  copied.value = what
  setTimeout(() => (copied.value = null), 1400)
}

/**
 * Autosave, on the same rhythm as a note: unsaved the moment you type, written when you pause.
 * A blank draft is not sent, because deleting a wrapup is its own act and not an empty save.
 */
function scheduleSave(): void {
  const task = selectedTask.value
  if (!task) return
  if (!draft.value.trim()) {
    store.saveState = 'saved'
    return
  }
  store.saveState = 'unsaved'
  if (saveTimer) clearTimeout(saveTimer)
  saveTimer = setTimeout(() => {
    void run(() => store.saveWrapupBody(task.id, draft.value))
  }, 700)
}

function beginWriting(): void {
  isDrafting.value = true
  mode.value = 'write'
}

async function confirmDelete(): Promise<void> {
  const task = selectedTask.value
  if (!task) return
  await run(() => store.removeWrapup(task.id), 'Wrapup deleted')
  isConfirmingDelete.value = false
}
</script>

<template>
  <section class="flex min-h-0 flex-1 flex-col bg-canvas" aria-label="Wrapup">
    <p v-if="!selectedTask" class="px-9 py-12 text-[13px] text-text-muted">
      Pick a task to see what it currently is.
    </p>

    <template v-else>
      <header class="flex shrink-0 items-start gap-3.5 border-b border-border px-5 py-3.5">
        <div class="min-w-0 flex-1">
          <p class="text-[10px] font-semibold uppercase tracking-[0.09em] text-text-subtle">
            Wrapup
          </p>
          <h2 class="truncate text-[19px] font-semibold tracking-[-0.015em] text-text">
            {{ selectedTask.title }}
          </h2>
          <button
            class="anchor-chip focus-ring mt-1.5 inline-flex items-center gap-2 px-2.5 py-1 text-[11.5px] transition-colors hover:border-anchor"
            :class="copied === 'anchor' && 'flash'"
            data-testid="copy-wrapup-anchor"
            @click="copy('anchor', rkCommand(selectedTask.anchor))"
          >
            <span class="opacity-60">/rk</span>
            <span>{{ selectedTask.anchor }}</span>
            <span class="opacity-70">{{ copied === 'anchor' ? 'copied' : 'copy' }}</span>
          </button>
        </div>

        <div v-if="selectedWrapup" class="flex shrink-0 items-center gap-2">
          <div class="flex gap-0.5 rounded-[7px] bg-surface p-0.5">
            <button
              v-for="option in (['write', 'read'] as const)"
              :key="option"
              class="focus-ring h-6 rounded-[5px] px-2.5 text-[11.5px] capitalize transition-colors"
              :class="mode === option ? 'bg-surface-raised text-text' : 'text-text-subtle hover:text-text'"
              :aria-pressed="mode === option"
              @click="mode = option"
            >
              {{ option }}
            </button>
          </div>
          <AppButton variant="danger" size="sm" @click="isConfirmingDelete = true">Delete</AppButton>
        </div>
      </header>

      <!-- Provenance, because both of you write here and neither merges with the other. -->
      <div
        v-if="selectedWrapup"
        class="flex shrink-0 flex-wrap items-center gap-x-3 gap-y-1.5 border-b border-border bg-surface px-5 py-2.5"
      >
        <span class="inline-flex items-center gap-1.5 text-[11.5px] text-text-muted">
          <svg
            v-if="selectedWrapup.writtenBy === 'CLAUDE'"
            class="size-3 shrink-0 text-text-subtle"
            viewBox="0 0 12 12"
            aria-hidden="true"
          >
            <path
              d="M6 0.5v3M6 8.5v3M0.5 6h3M8.5 6h3M2.3 2.3l2.1 2.1M7.6 7.6l2.1 2.1M9.7 2.3 7.6 4.4M4.4 7.6l-2.1 2.1"
              stroke="currentColor"
              stroke-width="1"
              stroke-linecap="round"
            />
          </svg>
          <svg v-else class="size-3 shrink-0 text-text-subtle" viewBox="0 0 12 12" aria-hidden="true">
            <path
              d="M2 10 3 6.6l4.4-4.4a1.2 1.2 0 0 1 1.7 1.7L4.7 8.3 2 10Z"
              fill="none"
              stroke="currentColor"
              stroke-width="1"
              stroke-linejoin="round"
            />
          </svg>
          Written by
          <span class="font-medium text-text">{{ writtenBy }}</span>
          <span class="text-text-muted">
            &middot; {{ relativeTime(selectedWrapup.updatedAt) }}
          </span>
        </span>

        <span
          v-if="wrapupIsBehind > 0"
          class="inline-flex items-center gap-1.5 rounded-full bg-warn-soft px-2 py-0.5 text-[11px] text-warn"
        >
          <span class="size-[5px] shrink-0 rounded-full bg-warn" aria-hidden="true" />
          {{ wrapupIsBehind }} note{{ wrapupIsBehind === 1 ? ' is' : 's are' }} newer than this
        </span>

        <button
          class="focus-ring ml-auto inline-flex items-center gap-2 rounded-[var(--radius-control)] border border-border-strong px-2.5 py-1 font-mono text-[11px] text-text-muted transition-colors hover:border-anchor hover:text-anchor"
          :class="copied === 'command' && 'flash'"
          data-testid="copy-wrapup-command"
          :title="'Paste this next to the terminal to have Claude rewrite it'"
          @click="copy('command', command)"
        >
          <span>{{ command }}</span>
          <span class="opacity-70">{{ copied === 'command' ? 'copied' : 'copy' }}</span>
        </button>
      </div>

      <!-- Nothing written yet. The primary path is Claude, so that is what the page leads with. -->
      <div v-if="!selectedWrapup && !isDrafting" class="min-h-0 flex-1 overflow-y-auto">
        <div class="max-w-[560px] px-9 py-12">
          <p class="text-[10px] font-semibold uppercase tracking-[0.09em] text-text-subtle">
            Wrapup
          </p>
          <h2 class="mb-1.5 mt-1.5 text-[26px] font-semibold leading-tight tracking-[-0.025em] text-text">
            Nobody has said what this is yet
          </h2>
          <p class="mb-6 text-[13px] leading-relaxed text-text-muted">
            One per task, replaced each time it is written. It describes the implementation as it
            stands — what exists, how it fits, what is still open — and never what changed along
            the way. That is what the notes are for.
          </p>

          <button
            class="focus-ring group flex w-full items-center gap-3 rounded-[var(--radius-control)] border border-border bg-surface px-3.5 py-3 text-left transition-all hover:-translate-y-px hover:border-anchor hover:bg-surface-raised hover:shadow-lift"
            :class="copied === 'command' && 'flash'"
            data-testid="copy-wrapup-command"
            @click="copy('command', command)"
          >
            <span class="min-w-0 flex-1">
              <span class="block truncate font-mono text-[12.5px] text-anchor">{{ command }}</span>
              <span class="mt-0.5 block text-[11.5px] text-text-muted">
                Run this at the end of a session and Claude writes it.
              </span>
            </span>
            <span class="shrink-0 text-[11px] text-text-subtle group-hover:text-anchor">
              {{ copied === 'command' ? 'copied' : 'copy' }}
            </span>
          </button>

          <p class="mt-6 text-[12.5px] text-text-muted">
            Or
            <button
              class="focus-ring rounded text-accent underline decoration-accent/40 underline-offset-2 hover:decoration-accent"
              data-testid="write-wrapup-by-hand"
              @click="beginWriting"
            >
              write it yourself
            </button>
            . Claude replaces what you write the next time it runs that command.
          </p>
        </div>
      </div>

      <div v-else class="flex min-h-0 flex-1 flex-col">
        <p
          v-if="isDrafting && !selectedWrapup"
          class="shrink-0 border-b border-border bg-surface px-5 py-2 text-[11.5px] text-text-muted"
        >
          This saves itself. Describe the state, not the session.
        </p>
        <div class="min-h-0 flex-1 overflow-y-auto p-4">
          <AppMarkdownEditor
            v-if="mode === 'write'"
            v-model="draft"
            height="100%"
            @update:model-value="scheduleSave"
          />
          <AppMarkdownEditor v-else :model-value="draft" readonly />
        </div>
      </div>
    </template>

    <AppConfirm
      v-if="isConfirmingDelete && selectedTask"
      :title="`Delete the wrapup on ${selectedTask.title}?`"
      body="The task and its notes stay. What goes is the description of where the implementation currently stands."
      blast="not recoverable · the next `/rk … wrapup` starts from nothing"
      confirm-label="Delete wrapup"
      @cancel="isConfirmingDelete = false"
      @confirm="confirmDelete"
    />
  </section>
</template>
