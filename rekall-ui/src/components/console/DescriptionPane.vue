<script setup lang="ts">
import { computed, onUnmounted, ref, watch } from 'vue'
import { storeToRefs } from 'pinia'
import AppMarkdownEditor from '@/components/ui/AppMarkdownEditor.vue'
import AppInput from '@/components/ui/AppInput.vue'
import AppCheckbox from '@/components/ui/AppCheckbox.vue'
import { useConsoleStore } from '@/stores/console.store'
import { useAsyncAction } from '@/composables/useAsyncAction'
import { identityHue } from '@/common/identity'
import { rkCommand } from '@/common/format/rk-command'
import LaunchClaudeCodeButton from '@/components/claude/LaunchClaudeCodeButton.vue'
import ClaudeSessionLauncher from '@/components/claude/ClaudeSessionLauncher.vue'
import type { TaskId } from '@/model/branded'

const store = useConsoleStore()
const { selectedTask } = storeToRefs(store)
const { run } = useAsyncAction()

const mode = ref<'write' | 'read'>('read')

const showEditor = ref(false)
const draft = ref('')
let saveTimer: ReturnType<typeof setTimeout> | null = null

const hue = computed(() => identityHue(selectedTask.value?.projectId ?? ''))

function flush(taskId: TaskId): void {
  if (!saveTimer) return
  clearTimeout(saveTimer)
  saveTimer = null
  void run(() => store.saveTaskDescription(taskId, draft.value))
}

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
    mode.value = showEditor.value ? 'read' : 'write'
  },
  { immediate: true }
)

watch(
  () => selectedTask.value?.description ?? '',
  (value) => {
    if (saveTimer || value === draft.value) return
    draft.value = value
    if (value.trim()) showEditor.value = true
  }
)

const anchor = computed(() => selectedTask.value?.anchor ?? '')

// "Generate wrapup", set once on the task so the console does not retype it after every step.
// The toggle saves on the spot; the directive it reveals autosaves on a pause, like the
// description does.
const autoWrapup = ref(false)
const wrapupDirective = ref('')
let wrapupTimer: ReturnType<typeof setTimeout> | null = null

function flushWrapup(taskId: TaskId): void {
  if (!wrapupTimer) return
  clearTimeout(wrapupTimer)
  wrapupTimer = null
  void run(() => store.saveTaskWrapup(taskId, autoWrapup.value, wrapupDirective.value))
}

function toggleAutoWrapup(value: boolean): void {
  const task = selectedTask.value
  if (!task) return
  if (wrapupTimer) {
    clearTimeout(wrapupTimer)
    wrapupTimer = null
  }
  autoWrapup.value = value
  void run(() => store.saveTaskWrapup(task.id, value, wrapupDirective.value))
}

function scheduleWrapupSave(): void {
  const task = selectedTask.value
  if (!task) return
  store.saveState = 'unsaved'
  if (wrapupTimer) clearTimeout(wrapupTimer)
  wrapupTimer = setTimeout(() => {
    wrapupTimer = null
    void run(() => store.saveTaskWrapup(task.id, autoWrapup.value, wrapupDirective.value))
  }, 700)
}

watch(
  () => selectedTask.value?.id ?? null,
  (_id, previousId) => {
    if (previousId) flushWrapup(previousId)
    autoWrapup.value = selectedTask.value?.autoWrapup ?? false
    wrapupDirective.value = selectedTask.value?.wrapupDirective ?? ''
  },
  { immediate: true }
)

watch(
  () => [selectedTask.value?.autoWrapup ?? false, selectedTask.value?.wrapupDirective ?? ''] as const,
  ([auto, directive]) => {
    if (wrapupTimer) return
    autoWrapup.value = auto
    wrapupDirective.value = directive
  }
)

const openSteps = computed(() => {
  const task = selectedTask.value
  return task ? task.stepCount - task.stepsDone : 0
})

// The description's own lifecycle, shown only while the task has no checklist. Same four values
// a step walks, read at task scope: RUNNING is a session on the anchor, CLAIMED is a
// Claude-written wrapup, DONE is "you accepted it".
const reviewState = computed(() => {
  const task = selectedTask.value
  return task && task.reviewActive && task.stepCount === 0 ? task.reviewState : null
})

const sendingBack = ref(false)
const sendBackNote = ref('')

async function acceptDescription(): Promise<void> {
  const task = selectedTask.value
  if (!task) return
  await run(() => store.acceptTask(task.id))
}

async function sendBackDescription(): Promise<void> {
  const task = selectedTask.value
  if (!task) return
  const note = sendBackNote.value.trim()
  sendingBack.value = false
  sendBackNote.value = ''
  await run(() => store.sendBackTask(task.id, note || undefined))
}

async function markTaskDone(): Promise<void> {
  const task = selectedTask.value
  if (!task) return
  await run(() => store.setTaskStatus(task.id, 'DONE'))
}

watch(
  () => selectedTask.value?.id ?? null,
  () => {
    sendingBack.value = false
    sendBackNote.value = ''
  }
)

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
  if (selectedTask.value) {
    flush(selectedTask.value.id)
    flushWrapup(selectedTask.value.id)
  }
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
            <p class="eyebrow flex items-center gap-1.5">
              <span class="h-2.5 w-[3px] shrink-0 rounded-full bg-accent" aria-hidden="true" />
              Description
            </p>
            <h2 class="flex items-center gap-2 text-[19px] font-semibold tracking-[-0.015em] text-text">
              <span class="truncate">{{ selectedTask.title }}</span>
              <span
                v-if="reviewState === 'RUNNING'"
                class="inline-flex shrink-0 items-center gap-1.5 rounded-full bg-accent-soft px-2 py-0.5 text-[10.5px] font-semibold tracking-[0.02em] text-accent"
                data-testid="description-running-flag"
              >
                <span class="relative grid size-2 place-items-center" aria-hidden="true">
                  <span class="absolute inline-flex size-2 animate-ping rounded-full bg-accent/60" />
                  <span class="relative inline-flex size-1.5 rounded-full bg-accent" />
                </span>
                running
              </span>
              <span
                v-else-if="reviewState === 'CLAIMED'"
                class="shrink-0 rounded-full border border-accent/40 px-2 py-0.5 text-[10.5px] font-semibold tracking-[0.02em] text-accent"
                data-testid="description-claimed-flag"
              >
                Awaiting review
              </span>
              <span
                v-else-if="reviewState === 'DONE'"
                class="shrink-0 rounded-full bg-accent px-2 py-0.5 text-[10.5px] font-semibold tracking-[0.02em] text-accent-ink"
                data-testid="description-accepted-flag"
              >
                Accepted
              </span>
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

          <div class="flex shrink-0 items-center gap-1.5">
            <ClaudeSessionLauncher
              :task-id="selectedTask.id"
              :folder="selectedTask.projectRepoFolder"
            />
            <LaunchClaudeCodeButton
              :anchors="selectedTask.anchor"
              :folder="selectedTask.projectRepoFolder"
              missing-hint="Set this project's folder on its page to open a session from it"
            />
          </div>

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

      <div
        class="shrink-0 border-b border-border bg-surface px-5 py-2.5"
        data-testid="wrapup-directive"
      >
        <div class="flex flex-wrap items-center gap-x-3 gap-y-1">
          <AppCheckbox
            :model-value="autoWrapup"
            label="Generate the wrapup automatically"
            data-testid="wrapup-auto-toggle"
            @update:model-value="toggleAutoWrapup"
          />
          <span class="text-[11.5px] text-text-muted">
            Folded into every
            <code class="text-anchor/80">/rk {{ anchor }}</code>
            without the session being asked.
          </span>
        </div>

        <div
          v-if="autoWrapup"
          class="mt-2 flex items-center gap-2"
          data-testid="wrapup-directive-field"
        >
          <span class="eyebrow shrink-0 text-[9.5px]">Directive</span>
          <AppInput
            v-model="wrapupDirective"
            class="min-w-0 flex-1"
            placeholder="Optional: the words the wrapup should follow, as if typed after wrapup"
            aria-label="Standing wrapup directive"
            data-testid="wrapup-directive-input"
            @update:model-value="scheduleWrapupSave"
          />
        </div>
      </div>

      <div
        v-if="reviewState === 'RUNNING' || reviewState === 'CLAIMED' || reviewState === 'DONE'"
        class="shrink-0 border-b border-border bg-accent-soft/40 px-5 py-2.5"
        data-testid="description-review-bar"
      >
        <div v-if="!sendingBack" class="flex flex-wrap items-center gap-2">
          <span class="min-w-0 flex-1 text-[11.5px] leading-snug text-text-muted">
            <template v-if="reviewState === 'RUNNING'">
              A session is on this task. It moves to review on its own once the session writes the
              wrapup, and back to open when the session ends. Accept it now if you have seen enough.
            </template>
            <template v-else-if="reviewState === 'CLAIMED'">
              A session wrote the wrapup and claimed this. Accept it, or send it back to reopen it
              for another pass.
            </template>
            <template v-else>
              You accepted this. Send it back to reopen it, or mark the whole task done.
            </template>
          </span>
          <button
            v-if="reviewState === 'CLAIMED' || reviewState === 'RUNNING'"
            class="focus-ring h-7 shrink-0 rounded-[var(--radius-control)] border border-accent bg-accent-soft px-3 text-[11.5px] font-medium text-accent transition-colors hover:bg-accent hover:text-accent-ink"
            data-testid="description-accept"
            @click="acceptDescription"
          >
            Accept
          </button>
          <button
            class="focus-ring h-7 shrink-0 rounded-[var(--radius-control)] border border-border-strong px-3 text-[11.5px] font-medium text-text-subtle transition-colors hover:border-accent hover:text-accent"
            data-testid="description-open-wrapup"
            @click="store.openWrapup()"
          >
            Open the wrapup
          </button>
          <button
            v-if="reviewState === 'DONE' && selectedTask.status !== 'DONE'"
            class="focus-ring h-7 shrink-0 rounded-[var(--radius-control)] border border-accent bg-accent-soft px-3 text-[11.5px] font-medium text-accent transition-colors hover:bg-accent hover:text-accent-ink"
            data-testid="description-mark-done"
            @click="markTaskDone"
          >
            Mark task done
          </button>
          <button
            v-if="reviewState !== 'RUNNING'"
            class="focus-ring h-7 shrink-0 rounded-[var(--radius-control)] border border-border-strong px-3 text-[11.5px] font-medium text-text-subtle transition-colors hover:border-danger hover:text-danger"
            data-testid="description-send-back"
            @click="sendingBack = true"
          >
            Send back
          </button>
        </div>
        <div v-else class="flex flex-wrap items-center gap-2">
          <AppInput
            v-model="sendBackNote"
            class="min-w-0 flex-1"
            placeholder="Optional: what needs another pass. The next session sees this."
            aria-label="Send back note"
            data-testid="description-send-back-note"
          />
          <button
            class="focus-ring h-7 shrink-0 rounded-[var(--radius-control)] border border-danger bg-danger-soft px-3 text-[11.5px] font-medium text-danger transition-colors"
            data-testid="description-send-back-confirm"
            @click="sendBackDescription"
          >
            Send back
          </button>
          <button
            class="focus-ring h-7 shrink-0 rounded-[var(--radius-control)] border border-border-strong px-3 text-[11.5px] font-medium text-text-subtle transition-colors hover:text-text"
            @click="sendingBack = false"
          >
            Cancel
          </button>
        </div>
      </div>

      <div
        v-if="showEditor"
        class="flex shrink-0 flex-wrap items-center gap-x-3 gap-y-1.5 border-b border-border bg-surface px-5 py-2.5"
      >
        <span v-if="openSteps > 0" class="text-[11.5px] text-text-muted">
          {{ openSteps }} step{{ openSteps === 1 ? '' : 's' }} open, so this is what they are
          built against rather than a list of things to do. Every
          <code class="text-anchor/80">/rk {{ anchor }}</code>
          hands it over as the standing context.
        </span>
        <span v-else class="text-[11.5px] text-text-muted">
          The brief the work is measured against. Every
          <code class="text-anchor/80">/rk {{ anchor }}</code>
          hands it to Claude before anything else.
        </span>
      </div>

      <div v-if="!showEditor" class="min-h-0 flex-1 overflow-y-auto">
        <div class="max-w-[620px] px-9 py-12">
          <p class="eyebrow">
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
                  name: 'Steps',
                  asks: 'What is left of it?',
                  says: 'Ticked by hand as the work is reviewed.',
                  glyph: 'check'
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
                <template v-else-if="surface.glyph === 'check'">
                  <path d="M1.2 3.4 2.6 4.8l2.4-2.6" stroke="currentColor" stroke-width="1.3" stroke-linecap="round" stroke-linejoin="round" />
                  <path d="M6.8 3.6h4M6.8 8.4h4M1.4 8.4h3.4" stroke="currentColor" stroke-width="1.2" stroke-linecap="round" />
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
