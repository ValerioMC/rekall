<script setup lang="ts">
import { computed, ref, watch } from 'vue'
import { storeToRefs } from 'pinia'
import AppButton from '@/components/ui/AppButton.vue'
import CopyGlyph from '@/components/ui/CopyGlyph.vue'
import AppMarkdownEditor from '@/components/ui/AppMarkdownEditor.vue'
import AppModeToggle from '@/components/ui/AppModeToggle.vue'
import AppSelect from '@/components/ui/AppSelect.vue'
import NoteAssignmentDialog from '@/components/console/NoteAssignmentDialog.vue'
import NoteScopeSelect from '@/components/console/NoteScopeSelect.vue'
import AppConfirm from '@/components/ui/AppConfirm.vue'
import LaunchClaudeCodeButton from '@/components/claude/LaunchClaudeCodeButton.vue'
import ConsoleHome from '@/components/console/ConsoleHome.vue'
import NoteContextModeSwitch from '@/components/console/NoteContextModeSwitch.vue'
import NoteMembershipStrip from '@/components/console/NoteMembershipStrip.vue'
import { useConsoleStore } from '@/stores/console.store'
import { useAsyncAction } from '@/composables/useAsyncAction'
import { rkCommand } from '@/common/format/rk-command'
import { DOCUMENT_KINDS } from '@/model/catalog'
import type { NoteScope } from '@/model/note-scope'

const store = useConsoleStore()
const {
  selectedDocument,
  selectedTaskId,
  navMode,
  tasks: allTasks
} = storeToRefs(store)
const { run } = useAsyncAction()

const KIND_OPTIONS = DOCUMENT_KINDS.map((kind) => ({
  value: kind,
  label: kind.charAt(0).toUpperCase() + kind.slice(1)
}))

const mode = ref<'write' | 'read'>('write')
const isAssigning = ref(false)
const isConfirmingDelete = ref(false)

function toggleMode(): void {
  if (!selectedDocument.value) return
  mode.value = mode.value === 'write' ? 'read' : 'write'
}

defineExpose({ toggleMode })

/**
 * A note belongs to no one project, so its path picker opens in the folder of the task it was
 * reached from when that task holds it, and otherwise in the first of its tasks that has one.
 */
const notePathRoot = computed<string | null>(() => {
  const note = selectedDocument.value
  if (!note) return null
  const onIds = note.tasks.map((ref) => ref.id)
  const ordered = selectedTaskId.value && onIds.includes(selectedTaskId.value)
    ? [selectedTaskId.value, ...onIds]
    : onIds
  for (const id of ordered) {
    const folder = allTasks.value.find((task) => task.id === id)?.projectRepoFolder
    if (folder) return folder
  }
  return null
})

const draft = ref({ title: '', kind: 'notes', bodyMarkdown: '' })
let saveTimer: ReturnType<typeof setTimeout> | null = null

watch(
  selectedDocument,
  (document) => {
    if (saveTimer) clearTimeout(saveTimer)
    if (!document) return
    draft.value = {
      title: document.title,
      kind: document.kind,
      bodyMarkdown: document.bodyMarkdown
    }
  },
  { immediate: true }
)

function scheduleSave(): void {
  const document = selectedDocument.value
  if (!document) return
  if (saveTimer) clearTimeout(saveTimer)
  const { title, kind, bodyMarkdown } = draft.value
  if (title === document.title && kind === document.kind && bodyMarkdown === document.bodyMarkdown) return
  store.saveState = 'unsaved'
  saveTimer = setTimeout(() => {
    void run(() => store.saveNote(document.id, { ...draft.value }))
  }, 700)
}

const anchorTask = computed(() => {
  const document = selectedDocument.value
  if (!document) return null
  const onCurrent = document.tasks.find((task) => task.id === selectedTaskId.value)
  const ref = onCurrent ?? document.tasks[0]
  return allTasks.value.find((task) => task.id === ref?.id) ?? null
})

const anchor = computed(() => anchorTask.value?.anchor ?? selectedDocument.value?.tasks[0]?.anchor ?? '')

const anchorFolder = computed(() => anchorTask.value?.projectRepoFolder ?? null)

/** Browsing notes, the placements column owns membership, so the strip here would say it twice. */
const showMembership = computed(() => navMode.value === 'tasks')

const justCopied = ref(false)

async function copyAnchor(): Promise<void> {
  await navigator.clipboard?.writeText(rkCommand(anchor.value))
  justCopied.value = true
  setTimeout(() => (justCopied.value = false), 1400)
}

async function moveTo(scope: NoteScope): Promise<void> {
  const document = selectedDocument.value
  if (!document) return
  await run(() => store.saveNote(document.id, { scope }), 'Moved.')
}

async function confirmDelete(): Promise<void> {
  const document = selectedDocument.value
  if (!document) return
  // A save still queued behind the debounce would land on a note that no longer exists.
  if (saveTimer) clearTimeout(saveTimer)
  await run(() => store.deleteNote(document.id), `Deleted ${document.title}`)
  isConfirmingDelete.value = false
}
</script>

<template>
  <section class="flex min-h-0 flex-1 flex-col bg-canvas" aria-label="Note">
    <ConsoleHome v-if="!selectedDocument" />

    <template v-else>
      <header class="texture-dots flex shrink-0 items-start gap-3.5 border-b border-border px-5 py-3.5">
        <div class="min-w-0 flex-1">
          <input
            v-model="draft.title"
            data-testid="note-title"
            class="focus-ring w-full truncate rounded-key border-0 bg-transparent text-heading font-semibold tracking-[-0.015em] text-text outline-none"
            aria-label="Note title"
            @input="scheduleSave"
          />
          <button
            class="anchor-chip focus-ring mt-1.5 inline-flex items-center gap-2 px-2.5 py-1 text-caption transition-colors hover:border-anchor"
            :class="justCopied && 'flash'"
            :title="justCopied ? 'Copied' : `Copy ${rkCommand(anchor)}`"
            data-testid="copy-anchor"
            @click="copyAnchor"
          >
            <span class="opacity-60">/rk</span>
            <span>{{ anchor }}</span>
            <CopyGlyph :copied="justCopied" />
          </button>
        </div>

        <div class="flex shrink-0 items-center gap-2">
          <LaunchClaudeCodeButton
            :anchors="anchor"
            :folder="anchorFolder"
            missing-hint="Open this note from the task you want to launch, and set that project's folder on its page"
          />
          <div class="w-[128px]">
            <AppSelect v-model="draft.kind" :options="KIND_OPTIONS" @change="scheduleSave" />
          </div>
          <AppButton variant="danger-quiet" size="md" data-testid="note-delete" @click="isConfirmingDelete = true">
            <svg class="size-3.5" viewBox="0 0 16 16" fill="none" aria-hidden="true">
              <path d="M2.5 4.5h11M6 4.5V3h4v1.5M4 4.5l.7 8.6a1 1 0 0 0 1 .9h4.6a1 1 0 0 0 1-.9l.7-8.6" stroke="currentColor" stroke-width="1.3" stroke-linecap="round" stroke-linejoin="round" />
            </svg>
            Delete
          </AppButton>
        </div>
      </header>

      <NoteMembershipStrip v-if="showMembership" :note="selectedDocument" @assign="isAssigning = true" />

      <div class="flex min-h-0 flex-1 flex-col">
        <div class="flex shrink-0 items-center gap-2 border-b border-border px-5 py-1.5">
          <NoteContextModeSwitch :note="selectedDocument" />
          <NoteScopeSelect :note="selectedDocument" @change="moveTo" />
          <AppModeToggle v-model="mode" class="ml-auto" />
        </div>

        <div class="min-h-0 flex-1 overflow-y-auto p-4">
          <Transition name="fade-quick" mode="out-in">
            <AppMarkdownEditor
              v-if="mode === 'write'"
              key="write"
              v-model="draft.bodyMarkdown"
              height="100%"
              :path-root="notePathRoot"
              @update:model-value="scheduleSave"
            />
            <AppMarkdownEditor v-else key="read" :model-value="draft.bodyMarkdown" readonly />
          </Transition>
        </div>
      </div>
    </template>

    <Transition name="dialog">
      <NoteAssignmentDialog v-if="isAssigning" @close="isAssigning = false" />
    </Transition>

    <Transition name="dialog">
      <AppConfirm
        v-if="isConfirmingDelete && selectedDocument"
        :title="`Delete ${selectedDocument.title}?`"
        :body="
          selectedDocument.tasks.length > 1
            ? 'This note is on more than one task. Deleting removes it everywhere.'
            : 'This removes the note and its content.'
        "
        :blast="`removes it from ${selectedDocument.tasks.length} task${selectedDocument.tasks.length === 1 ? '' : 's'} · not recoverable`"
        confirm-label="Delete note"
        @cancel="isConfirmingDelete = false"
        @confirm="confirmDelete"
      />
    </Transition>
  </section>
</template>
