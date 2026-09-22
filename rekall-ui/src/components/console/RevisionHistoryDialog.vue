<script setup lang="ts">
import { computed, nextTick, onMounted, onUnmounted, ref } from 'vue'
import AppButton from '@/components/ui/AppButton.vue'
import AppMarkdownEditor from '@/components/ui/AppMarkdownEditor.vue'
import { useModalGate } from '@/composables/useModalGate'
import { useAsyncAction } from '@/composables/useAsyncAction'
import { trapTabKey } from '@/common/a11y/focus-trap'
import { relativeTime } from '@/common/format/relative-time'
import { fetchRevisions } from '@/api/revisions.api'
import { useConsoleStore } from '@/stores/console.store'
import { WRAPUP_AUTHOR_LABEL } from '@/model/catalog'
import type { TaskId } from '@/model/branded'
import type { RevisionKind, TaskRevision } from '@/model/revision'

const props = defineProps<{ taskId: TaskId; taskTitle: string; kind: RevisionKind }>()
const emit = defineEmits<{ close: []; restored: [] }>()

const store = useConsoleStore()
const { open: openModal, close: closeModal } = useModalGate()
const { run: runLoad, isRunning: loading } = useAsyncAction()
const { run: runRestore, isRunning: restoring } = useAsyncAction()

const panel = ref<HTMLElement | null>(null)
const revisions = ref<TaskRevision[]>([])
const selectedId = ref<string | null>(null)

const noun = computed(() => (props.kind === 'WRAPUP' ? 'wrapup' : 'description'))
const selected = computed(() => revisions.value.find((revision) => revision.id === selectedId.value) ?? null)

function authorOf(revision: TaskRevision): string {
  return revision.writtenBy ? `by ${WRAPUP_AUTHOR_LABEL[revision.writtenBy]}` : ''
}

async function restore(): Promise<void> {
  const revision = selected.value
  if (!revision || restoring.value) return
  const done = await runRestore(
    () => store.restoreRevision(props.taskId, revision.id),
    `Earlier ${noun.value} restored. The one it replaced is in the history now.`
  )
  if (done !== null) emit('restored')
}

function onKeydown(event: KeyboardEvent): void {
  if (event.key === 'Escape') {
    event.stopPropagation()
    if (!restoring.value) emit('close')
    return
  }
  if (panel.value) trapTabKey(panel.value, event)
}

onMounted(async () => {
  openModal()
  window.addEventListener('keydown', onKeydown, true)
  const loaded = await runLoad(() => fetchRevisions(props.taskId, props.kind))
  revisions.value = loaded ?? []
  selectedId.value = revisions.value[0]?.id ?? null
  await nextTick()
  panel.value?.querySelector<HTMLElement>('[data-revision-row]')?.focus()
})

onUnmounted(() => {
  closeModal()
  window.removeEventListener('keydown', onKeydown, true)
})
</script>

<template>
  <div
    class="fixed inset-0 z-(--z-modal) grid place-items-center bg-black/70 p-5 backdrop-blur-sm"
    @click.self="emit('close')"
  >
    <div
      ref="panel"
      class="dialog-panel flex h-[min(640px,85vh)] w-full max-w-[880px] flex-col overflow-hidden rounded-[var(--radius-card)] border border-border-strong bg-surface shadow-modal"
      role="dialog"
      aria-modal="true"
      :aria-label="`Earlier versions of the ${noun}`"
      data-testid="revision-history-dialog"
    >
      <header class="flex shrink-0 items-center gap-3 border-b border-border px-5 py-4">
        <span class="min-w-0 flex-1">
          <span class="block text-[15px] font-semibold tracking-[-0.01em] text-text">
            Earlier versions of the {{ noun }}
          </span>
          <span class="mt-0.5 block truncate text-[12.5px] leading-relaxed text-text-muted">
            {{ taskTitle }} · kept whenever something replaced or deleted it, newest first
          </span>
        </span>
        <button
          class="focus-ring grid size-7 shrink-0 place-items-center rounded-md text-text-subtle transition-colors hover:bg-surface-raised hover:text-text disabled:cursor-not-allowed disabled:opacity-40"
          aria-label="Close"
          :disabled="restoring"
          @click="emit('close')"
        >
          &times;
        </button>
      </header>

      <p v-if="loading" class="px-5 py-10 text-[13px] text-text-muted">Loading…</p>

      <p
        v-else-if="revisions.length === 0"
        class="px-5 py-10 text-[13px] text-text-muted"
        data-testid="revision-history-empty"
      >
        Nothing has replaced this {{ noun }} yet. When something does, the version it replaced is
        kept here.
      </p>

      <div v-else class="flex min-h-0 flex-1">
        <ul
          class="w-[240px] shrink-0 overflow-y-auto border-r border-border bg-canvas py-1.5"
          aria-label="Versions"
        >
          <li v-for="revision in revisions" :key="revision.id">
            <button
              data-revision-row
              class="focus-ring flex w-full flex-col items-start gap-0.5 px-4 py-2.5 text-left transition-colors hover:bg-surface-raised"
              :class="revision.id === selectedId && 'bg-accent-soft'"
              :aria-current="revision.id === selectedId ? 'true' : undefined"
              data-testid="revision-row"
              @click="selectedId = revision.id"
            >
              <span class="text-[12.5px] font-medium text-text">
                Replaced {{ relativeTime(revision.replacedAt) }}
              </span>
              <span class="text-[11.5px] text-text-muted">
                <template v-if="revision.writtenAt">Written {{ relativeTime(revision.writtenAt) }}</template>
                {{ authorOf(revision) }}
              </span>
            </button>
          </li>
        </ul>

        <div class="min-h-0 min-w-0 flex-1 overflow-y-auto p-4" data-testid="revision-preview">
          <AppMarkdownEditor v-if="selected" :key="selected.id" :model-value="selected.bodyMarkdown" readonly />
        </div>
      </div>

      <footer class="flex shrink-0 items-center justify-end gap-2 border-t border-border bg-canvas px-5 py-3">
        <span class="mr-auto text-[11.5px] text-text-subtle">
          Restoring keeps the current {{ noun }} here too, so it can be undone.
        </span>
        <AppButton variant="ghost" size="sm" :disabled="restoring" @click="emit('close')">
          Close
        </AppButton>
        <AppButton
          variant="primary"
          size="sm"
          data-testid="revision-restore"
          :disabled="!selected"
          :loading="restoring"
          @click="restore"
        >
          Restore this version
        </AppButton>
      </footer>
    </div>
  </div>
</template>
