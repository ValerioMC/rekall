<script setup lang="ts">
import { computed, nextTick, onMounted, onUnmounted, ref } from 'vue'
import AppBadge from '@/components/ui/AppBadge.vue'
import AppButton from '@/components/ui/AppButton.vue'
import AppConfirm from '@/components/ui/AppConfirm.vue'
import ClaudeCodeSection from '@/components/settings/ClaudeCodeSection.vue'
import NotificationsSection from '@/components/settings/NotificationsSection.vue'
import BackupsSection from '@/components/settings/BackupsSection.vue'
import DatabaseFolderField from '@/components/setup/DatabaseFolderField.vue'
import RestartingOverlay from '@/components/setup/RestartingOverlay.vue'
import CloseGlyph from '@/components/ui/CloseGlyph.vue'
import { fetchDatabaseStatus, forgetDatabase, renameDatabase } from '@/api/settings.api'
import { useDatabaseSetup } from '@/composables/useDatabaseSetup'
import { useModalGate } from '@/composables/useModalGate'
import { trapTabKey } from '@/common/a11y/focus-trap'
import { useToastStore } from '@/stores/toast.store'
import type { DatabaseEntry, DatabaseStatus } from '@/model/settings'

const emit = defineEmits<{ close: [] }>()

const toast = useToastStore()
const { open: openModal, close: closeModal } = useModalGate()
const { phase, error, switchTo } = useDatabaseSetup()

const status = ref<DatabaseStatus | null>(null)
const loading = ref(true)
const addingNew = ref(false)
const addingNewIsBusy = ref(false)
const switchingId = ref<string | null>(null)
const editingId = ref<string | null>(null)
const editingLabel = ref('')
const editingInput = ref<HTMLInputElement | null>(null)
const forgetting = ref<DatabaseEntry | null>(null)
/** A backup confirm or restore is open in the section below: the panel stays put under it. */
const backupBusy = ref(false)
const closeButton = ref<HTMLButtonElement | null>(null)
const panel = ref<HTMLElement | null>(null)

const previewingPath = ref<DatabaseEntry | null>(null)
const pathPreviewStyle = ref<Record<string, string>>({})
const PATH_PREVIEW_WIDTH = 420

function openPathPreview(event: MouseEvent | FocusEvent, entry: DatabaseEntry): void {
  const rect = (event.currentTarget as HTMLElement).getBoundingClientRect()
  const left = Math.min(Math.max(12, rect.left), window.innerWidth - PATH_PREVIEW_WIDTH - 12)
  pathPreviewStyle.value = { top: `${rect.bottom + 6}px`, left: `${left}px` }
  previewingPath.value = entry
}

function closePathPreview(): void {
  previewingPath.value = null
}

const canClose = computed(
  () => phase.value !== 'submitting' && phase.value !== 'restarting' && !addingNewIsBusy.value && !backupBusy.value
)

async function load(): Promise<void> {
  loading.value = true
  try {
    status.value = await fetchDatabaseStatus()
  } catch (caught) {
    toast.notifyError(caught)
  } finally {
    loading.value = false
  }
}

async function onSwitch(entry: DatabaseEntry): Promise<void> {
  switchingId.value = entry.id
  await switchTo(entry)
  switchingId.value = null
}

async function beginRename(entry: DatabaseEntry): Promise<void> {
  editingId.value = entry.id
  editingLabel.value = entry.label
  await nextTick()
  editingInput.value?.focus()
  editingInput.value?.select()
}

async function saveRename(): Promise<void> {
  if (!editingId.value || !editingLabel.value.trim()) return
  try {
    await renameDatabase(editingId.value, editingLabel.value.trim())
    editingId.value = null
    await load()
  } catch (caught) {
    toast.notifyError(caught)
  }
}

async function confirmForget(): Promise<void> {
  if (!forgetting.value) return
  try {
    await forgetDatabase(forgetting.value.id)
    forgetting.value = null
    await load()
    toast.notify('Forgotten. The database file itself was not touched.')
  } catch (caught) {
    toast.notifyError(caught)
  }
}

function onKeydown(event: KeyboardEvent): void {
  if (event.key === 'Escape' && canClose.value && !forgetting.value && editingId.value === null) {
    event.stopPropagation()
    emit('close')
    return
  }
  if (panel.value && !forgetting.value && !backupBusy.value) trapTabKey(panel.value, event)
}

onMounted(async () => {
  openModal()
  window.addEventListener('keydown', onKeydown, true)
  void load()
  await nextTick()
  closeButton.value?.focus()
})
onUnmounted(() => {
  closeModal()
  window.removeEventListener('keydown', onKeydown, true)
})
</script>

<template>
  <div
    class="fixed inset-0 z-(--z-modal) grid place-items-center bg-black/70 p-5 backdrop-blur-sm"
    @click.self="canClose && emit('close')"
  >
    <div
      ref="panel"
      class="dialog-panel flex max-h-[85vh] w-full max-w-[560px] flex-col overflow-hidden rounded-[var(--radius-card)] border border-border-strong bg-surface shadow-modal"
      role="dialog"
      aria-modal="true"
      aria-label="Settings"
      data-testid="settings-panel"
    >
      <header class="flex items-center gap-3 border-b border-border px-6 py-4">
        <span class="text-[15px] font-semibold tracking-[-0.01em] text-text">Settings</span>
        <button
          ref="closeButton"
          type="button"
          :disabled="!canClose"
          :title="canClose ? 'Close' : 'A restart is in progress and has to finish first'"
          class="focus-ring ml-auto grid size-7 place-items-center rounded-md text-text-subtle transition-colors hover:bg-surface-raised hover:text-text disabled:cursor-not-allowed disabled:opacity-40"
          aria-label="Close"
          @click="canClose && emit('close')"
        >
          <CloseGlyph />
        </button>
      </header>

      <div class="min-h-0 flex-1 overflow-y-auto px-6 py-5">
        <p class="mb-3 eyebrow text-[11px]">Database</p>

        <RestartingOverlay v-if="phase === 'restarting'" />

        <div v-else-if="loading" class="space-y-2" aria-hidden="true">
          <div class="skeleton h-14 rounded-[var(--radius-control)]" />
          <div class="skeleton h-14 rounded-[var(--radius-control)]" />
        </div>

        <template v-else-if="status">
          <p v-if="error" class="mb-3 text-[12.5px] text-danger" role="alert">{{ error }}</p>

          <ul class="space-y-2">
            <li
              v-for="entry in status.databases"
              :key="entry.id"
              class="rounded-[var(--radius-control)] border px-3.5 py-3"
              :class="entry.active ? 'selected-row border-accent-deep' : 'border-border bg-canvas'"
              data-testid="database-row"
            >
              <div class="flex items-start justify-between gap-3">
                <div class="min-w-0 flex-1">
                  <div class="flex flex-wrap items-center gap-x-2 gap-y-1">
                    <template v-if="editingId === entry.id">
                      <input
                        :ref="(el) => (editingInput = el as HTMLInputElement | null)"
                        v-model="editingLabel"
                        aria-label="Rename database"
                        class="focus-ring h-7 w-full max-w-[220px] rounded-md border border-border bg-canvas px-2 text-[13px] text-text"
                        @keydown.enter="saveRename"
                        @keydown.esc.stop="editingId = null"
                      />
                      <AppButton size="sm" variant="ghost" @click="saveRename">Save</AppButton>
                      <AppButton size="sm" variant="ghost" @click="editingId = null">Cancel</AppButton>
                    </template>
                    <template v-else>
                      <span class="truncate text-[13.5px] font-medium text-text">{{ entry.label }}</span>
                      <button
                        type="button"
                        class="focus-ring shrink-0 rounded text-[11px] text-text-subtle underline decoration-dotted hover:text-text"
                        @click="beginRename(entry)"
                      >
                        rename
                      </button>
                    </template>
                    <AppBadge v-if="entry.active" tone="accent" dot>In use</AppBadge>
                    <AppBadge v-else-if="!entry.reachable" tone="danger">Unreachable</AppBadge>
                  </div>
                  <p
                    class="mt-1 truncate font-mono text-[11.5px] text-text-subtle"
                    tabindex="0"
                    @mouseenter="openPathPreview($event, entry)"
                    @mouseleave="closePathPreview"
                    @focus="openPathPreview($event, entry)"
                    @blur="closePathPreview"
                  >
                    {{ entry.path }}
                  </p>
                </div>
                <div class="flex shrink-0 items-center gap-1.5">
                  <AppButton
                    v-if="!entry.active && entry.reachable"
                    size="sm"
                    variant="secondary"
                    :loading="switchingId === entry.id"
                    :disabled="phase === 'submitting'"
                    @click="onSwitch(entry)"
                  >
                    Switch
                  </AppButton>
                  <AppButton
                    v-if="!entry.active"
                    size="sm"
                    variant="danger-quiet"
                    :disabled="phase === 'submitting'"
                    @click="forgetting = entry"
                  >
                    Forget
                  </AppButton>
                </div>
              </div>
            </li>
          </ul>

          <div class="mt-4">
            <AppButton v-if="!addingNew" variant="ghost" size="sm" @click="addingNew = true">
              + Add another database
            </AppButton>
            <DatabaseFolderField
              v-else
              autofocus
              cancellable
              submit-label="Add database"
              @cancel="addingNew = false"
              @busy="addingNewIsBusy = $event"
            />
          </div>
        </template>

        <div v-else class="flex flex-col items-start gap-3 text-[12.5px] text-text-subtle">
          <p>Couldn't load the database list.</p>
          <AppButton size="sm" variant="secondary" @click="load">Retry</AppButton>
        </div>

        <ClaudeCodeSection class="mt-6 border-t border-border pt-5" />
        <BackupsSection class="mt-6 border-t border-border pt-5" @busy="backupBusy = $event" />
        <NotificationsSection class="mt-6 border-t border-border pt-5" />
      </div>
    </div>

    <Teleport to="body">
      <div
        v-if="previewingPath"
        class="pointer-events-none fixed z-(--z-toast) max-w-[420px] rounded-[var(--radius-control)] border border-border-strong bg-surface-raised px-2.5 py-1.5 font-mono text-[11px] break-all text-text shadow-modal"
        :style="pathPreviewStyle"
        role="tooltip"
      >
        {{ previewingPath.path }}
      </div>
    </Teleport>

    <Transition name="dialog">
      <AppConfirm
        v-if="forgetting"
        :title="`Forget ${forgetting.label}?`"
        body="This only removes it from Rekall's list. The database file itself is untouched, and can be added back later by pointing at the same folder again."
        :blast="forgetting.path"
        confirm-label="Forget"
        @cancel="forgetting = null"
        @confirm="confirmForget"
      />
    </Transition>
  </div>
</template>
