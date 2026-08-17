<script setup lang="ts">
import { computed, nextTick, onMounted, onUnmounted, ref } from 'vue'
import AppBadge from '@/components/ui/AppBadge.vue'
import AppButton from '@/components/ui/AppButton.vue'
import AppConfirm from '@/components/ui/AppConfirm.vue'
import DatabaseFolderField from '@/components/setup/DatabaseFolderField.vue'
import RestartingOverlay from '@/components/setup/RestartingOverlay.vue'
import { fetchDatabaseStatus, forgetDatabase, renameDatabase } from '@/api/settings.api'
import { useDatabaseSetup } from '@/composables/useDatabaseSetup'
import { useModalGate } from '@/composables/useModalGate'
import { trapTabKey } from '@/common/a11y/focus-trap'
import { useToastStore } from '@/stores/toast.store'
import type { DatabaseEntry, DatabaseStatus } from '@/model/settings'

/**
 * One section today — Database — because that is the only setting Rekall has. Structured as a
 * dialog rather than a route: the console has no router, and a settings screen is a place you
 * pass through, not a place you stay.
 */
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
const closeButton = ref<HTMLButtonElement | null>(null)
const panel = ref<HTMLElement | null>(null)

/**
 * Whether this panel may be dismissed right now. Once any restart-triggering action has been
 * submitted — switching, or adding a folder through the inline field below — the backend has
 * already committed to restarting; closing the panel at that point would only hide the
 * "Reconfiguring…" state, not stop it, and the page would then reload without warning whenever
 * it finishes.
 */
const canClose = computed(() => phase.value !== 'submitting' && phase.value !== 'restarting' && !addingNewIsBusy.value)

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

/**
 * Only Escape is intercepted here, and only to close the panel — never unconditionally, because
 * a capture-phase `stopPropagation()` on every key would also stop the event from ever reaching
 * this panel's own descendants, such as the rename field's own Enter-to-save. The console's
 * single-key shortcuts ('n', 'j', 't', …) are kept from leaking through separately, via
 * `useModalGate`, which `App.vue` checks before acting on any key at all.
 */
function onKeydown(event: KeyboardEvent): void {
  if (event.key === 'Escape' && canClose.value && !forgetting.value && editingId.value === null) {
    event.stopPropagation()
    emit('close')
    return
  }
  // While the "forget this database?" confirmation sits on top, its own trap is the one to run.
  if (panel.value && !forgetting.value) trapTabKey(panel.value, event)
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
    class="fade-in fixed inset-0 z-(--z-modal) grid place-items-center bg-black/70 p-5 backdrop-blur-sm"
    @click.self="canClose && emit('close')"
  >
    <div
      ref="panel"
      class="rise flex max-h-[85vh] w-full max-w-[560px] flex-col overflow-hidden rounded-[var(--radius-card)] border border-border-strong bg-surface shadow-modal"
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
          &times;
        </button>
      </header>

      <div class="min-h-0 flex-1 overflow-y-auto px-6 py-5">
        <p class="mb-3 text-[11px] font-semibold uppercase tracking-[0.09em] text-text-subtle">Database</p>

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
                  <p class="mt-1 truncate font-mono text-[11.5px] text-text-subtle" :title="entry.path">
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
                    variant="danger"
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
      </div>
    </div>

    <AppConfirm
      v-if="forgetting"
      :title="`Forget ${forgetting.label}?`"
      body="This only removes it from Rekall's list. The database file itself is untouched, and can be added back later by pointing at the same folder again."
      :blast="forgetting.path"
      confirm-label="Forget"
      @cancel="forgetting = null"
      @confirm="confirmForget"
    />
  </div>
</template>
