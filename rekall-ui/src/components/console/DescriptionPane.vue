<script setup lang="ts">
import { computed, onUnmounted, ref, watch } from 'vue'
import { storeToRefs } from 'pinia'
import AppMarkdownEditor from '@/components/ui/AppMarkdownEditor.vue'
import { useConsoleStore } from '@/stores/console.store'
import { useAsyncAction } from '@/composables/useAsyncAction'
import { identityHue } from '@/common/identity'
import { rkCommand } from '@/common/format/rk-command'
import LaunchClaudeCodeButton from '@/components/claude/LaunchClaudeCodeButton.vue'
import type { TaskId } from '@/model/branded'

/**
 * The brief of the task in view: what the work is, at whatever length it takes to say.
 *
 * It gets the whole pane and a markdown editor because a description is rarely one sentence.
 * What has to be built, what it has to satisfy, what is deliberately out of scope: that is a
 * structured document, and it was being typed into a field the size of a tooltip.
 *
 * Three surfaces, one question each, and the distinction is why none of them is a tab of
 * another. The description is the standing brief and changes when the work is redefined. The
 * wrapup is where the implementation got to and is replaced at the end of every session. A note
 * is something you learned, keeps its own title, and can be attached to several tasks at once.
 *
 * The pane carries its project's colour on the rail above the title and in the grid behind it —
 * the same hue the navigator groups the task under. Three panes share this frame, and the
 * texture is what says which one is on screen before a word of it is read.
 */
const store = useConsoleStore()
const { selectedTask } = storeToRefs(store)
const { run } = useAsyncAction()

const mode = ref<'write' | 'read'>('read')

/**
 * Whether the editor is on screen at all.
 *
 * A task with no description opens on the page that says what one is for, because that is the
 * moment the three surfaces are easiest to confuse. Once there is text, or once you have asked
 * to write it, this pane is the editor and stays the editor — emptying the field mid-sentence
 * must not throw you back to the explanation.
 */
const showEditor = ref(false)
const draft = ref('')
let saveTimer: ReturnType<typeof setTimeout> | null = null

const hue = computed(() => identityHue(selectedTask.value?.projectId ?? ''))

/** Writes what is pending now, for the task it was typed on rather than the one now in view. */
function flush(taskId: TaskId): void {
  if (!saveTimer) return
  clearTimeout(saveTimer)
  saveTimer = null
  void run(() => store.saveTaskDescription(taskId, draft.value))
}

/**
 * Autosave, on the rhythm the notes and the wrapup use: unsaved the moment you type, written
 * when you pause. An emptied description saves like any other edit — clearing the brief is a
 * legitimate thing to do, and the store stores it as none rather than as an empty paragraph.
 */
function scheduleSave(): void {
  const task = selectedTask.value
  if (!task) return
  store.saveState = 'unsaved'
  if (saveTimer) clearTimeout(saveTimer)
  saveTimer = setTimeout(() => {
    saveTimer = null
    void run(() => store.saveTaskDescription(task.id, draft.value))
  }, 700)
}

watch(
  () => selectedTask.value?.id ?? null,
  (_id, previousId) => {
    if (previousId) flush(previousId)
    draft.value = selectedTask.value?.description ?? ''
    showEditor.value = draft.value.trim().length > 0
    // Reading is the default on a task that has one: this pane is opened to find out what the
    // work is far more often than to redefine it.
    mode.value = showEditor.value ? 'read' : 'write'
  },
  { immediate: true }
)

/** The same field, edited in the task dialog while this pane is open. Anything typed here wins
 *  until it is written, so a pending save is left alone. */
watch(
  () => selectedTask.value?.description ?? '',
  (value) => {
    if (saveTimer || value === draft.value) return
    draft.value = value
    if (value.trim()) showEditor.value = true
  }
)

const anchor = computed(() => selectedTask.value?.anchor ?? '')

const copied = ref(false)

async function copyAnchor(): Promise<void> {
  await navigator.clipboard?.writeText(rkCommand(anchor.value))
  copied.value = true
  setTimeout(() => (copied.value = false), 1400)
}

function beginWriting(): void {
  showEditor.value = true
  mode.value = 'write'
}

onUnmounted(() => {
  if (selectedTask.value) flush(selectedTask.value.id)
})
</script>

<template>
  <section class="flex min-h-0 flex-1 flex-col bg-canvas" aria-label="Description">
    <p v-if="!selectedTask" class="px-9 py-12 text-[13px] text-text-muted">
      Pick a task to see what it is.
    </p>

    <template v-else>
      <div class="relative shrink-0 overflow-hidden border-b border-border">
        <div
          class="texture-grid pointer-events-none absolute inset-0 opacity-60"
          :style="{ '--texture-tint': hue.base }"
          aria-hidden="true"
        />
        <div
          class="pointer-events-none absolute inset-x-0 top-0 h-px"
          :style="{ background: hue.line }"
          aria-hidden="true"
        />

        <header class="relative flex items-start gap-3.5 px-5 py-3.5">
          <div class="min-w-0 flex-1">
            <p class="text-[10px] font-semibold uppercase tracking-[0.09em] text-text-subtle">
              Description
            </p>
            <h2 class="truncate text-[19px] font-semibold tracking-[-0.015em] text-text">
              {{ selectedTask.title }}
            </h2>
            <button
              class="anchor-chip focus-ring mt-1.5 inline-flex items-center gap-2 px-2.5 py-1 text-[11.5px] transition-colors hover:border-anchor"
              :class="copied && 'flash'"
              data-testid="copy-description-anchor"
              @click="copyAnchor"
            >
              <span class="opacity-60">/rk</span>
              <span>{{ selectedTask.anchor }}</span>
              <span class="opacity-70">{{ copied ? 'copied' : 'copy' }}</span>
            </button>
          </div>

          <LaunchClaudeCodeButton
            class="shrink-0"
            :anchors="selectedTask.anchor"
            :folder="selectedTask.projectRepoFolder"
            missing-hint="Set this project's folder on its page to open a session from it"
          />

          <div v-if="showEditor" class="flex shrink-0 gap-0.5 rounded-[7px] bg-surface p-0.5">
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
        </header>
      </div>

      <!-- Where these words end up, which is the reason to keep them current. -->
      <div
        v-if="showEditor"
        class="flex shrink-0 flex-wrap items-center gap-x-3 gap-y-1.5 border-b border-border bg-surface px-5 py-2.5"
      >
        <span class="text-[11.5px] text-text-muted">
          The brief the work is measured against. Every
          <code class="text-anchor/80">/rk {{ anchor }}</code>
          hands it to Claude before anything else.
        </span>
      </div>

      <!-- Nothing written yet. The page is about what belongs here, because this is where the
           description, the wrapup and a note are easiest to confuse. -->
      <div v-if="!showEditor" class="min-h-0 flex-1 overflow-y-auto">
        <div class="max-w-[620px] px-9 py-12">
          <p class="text-[10px] font-semibold uppercase tracking-[0.09em] text-text-subtle">
            Description
          </p>
          <h2 class="mb-1.5 mt-1.5 text-[26px] font-semibold leading-tight tracking-[-0.025em] text-text">
            Nothing says what this task is
          </h2>
          <p class="mb-6 text-[13px] leading-relaxed text-text-muted">
            The brief: what has to be built, what it has to satisfy, what is out of scope. Markdown,
            at whatever length the work needs, corrected when the work is redefined rather than
            when it moves.
          </p>

          <button
            class="focus-ring rounded-[var(--radius-control)] border border-accent bg-accent-soft px-3.5 py-2 text-[12.5px] font-medium text-accent transition-colors hover:bg-accent hover:text-accent-ink"
            data-testid="write-description"
            @click="beginWriting"
          >
            Write the description
          </button>

          <!-- Three surfaces, one question each. Written down here because this is the pane
               that was missing, and the one whose job is easiest to give to the other two. -->
          <ul class="mt-9 border-t border-border">
            <li
              v-for="surface in [
                {
                  name: 'Description',
                  asks: 'What is this task?',
                  says: 'Changes when the work is redefined.',
                  glyph: 'page'
                },
                {
                  name: 'Wrapup',
                  asks: 'Where did the implementation get to?',
                  says: 'Rewritten at the end of every session.',
                  glyph: 'diamond'
                },
                {
                  name: 'Note',
                  asks: 'What did I learn?',
                  says: 'Keeps its title, and can be on several tasks.',
                  glyph: 'lines'
                }
              ]"
              :key="surface.name"
              class="flex items-start gap-3 border-b border-border py-3"
            >
              <svg
                class="mt-0.5 size-3 shrink-0"
                :class="surface.name === 'Description' ? 'text-accent' : 'text-text-subtle'"
                viewBox="0 0 12 12"
                fill="none"
                aria-hidden="true"
              >
                <template v-if="surface.glyph === 'page'">
                  <path d="M2.6 1.1h4.1l2.7 2.7v7.1H2.6z" stroke="currentColor" stroke-width="1.2" stroke-linejoin="round" />
                  <path d="M4.5 6.1h3.2M4.5 8.2h2" stroke="currentColor" stroke-width="1.1" stroke-linecap="round" />
                </template>
                <path
                  v-else-if="surface.glyph === 'diamond'"
                  d="M6 1.2 10.8 6 6 10.8 1.2 6z"
                  stroke="currentColor"
                  stroke-width="1.3"
                  stroke-linejoin="round"
                />
                <path
                  v-else
                  d="M1.6 3h8.8M1.6 6h8.8M1.6 9h5.4"
                  stroke="currentColor"
                  stroke-width="1.2"
                  stroke-linecap="round"
                />
              </svg>
              <span class="min-w-0 flex-1">
                <span class="flex flex-wrap items-baseline gap-x-2.5">
                  <span class="text-[12.5px] font-semibold text-text">{{ surface.name }}</span>
                  <span class="text-[12.5px] text-text-muted">{{ surface.asks }}</span>
                </span>
                <span class="mt-0.5 block text-[11.5px] text-text-subtle">{{ surface.says }}</span>
              </span>
            </li>
          </ul>
        </div>
      </div>

      <div v-else class="min-h-0 flex-1 overflow-y-auto p-4">
        <!-- Read on the pane's own width, as a note is. A measure column here was the reason a
             brief looked broken up: the text stopped short of the pane with the rest of the
             width visibly empty, and the same text in a note did not. -->
        <AppMarkdownEditor
          v-if="mode === 'write'"
          v-model="draft"
          height="100%"
          @update:model-value="scheduleSave"
        />
        <AppMarkdownEditor v-else :model-value="draft" readonly />
      </div>
    </template>
  </section>
</template>
