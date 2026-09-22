<script setup lang="ts">
import { computed, onMounted, ref, watch } from 'vue'
import AppButton from '@/components/ui/AppButton.vue'
import AppConfirm from '@/components/ui/AppConfirm.vue'
import RestartingOverlay from '@/components/setup/RestartingOverlay.vue'
import { useAsyncAction } from '@/composables/useAsyncAction'
import { reloadWhenBack } from '@/composables/useDatabaseSetup'
import { relativeTime } from '@/common/format/relative-time'
import {
  backupDownloadUrl,
  fetchBackupStatus,
  restoreBackup,
  restoreUploadedBackup,
  takeBackup
} from '@/api/backups.api'
import { BACKUP_REASON_LABEL, readableSize } from '@/model/backup'
import type { BackupFile, BackupStatus } from '@/model/backup'

/** True while a confirm is open or a restore is under way: the panel must not close under it. */
const emit = defineEmits<{ busy: [busy: boolean] }>()

const { run } = useAsyncAction()
const { run: runBackup, isRunning: backingUp } = useAsyncAction()
const { run: runRestore } = useAsyncAction()

const status = ref<BackupStatus | null>(null)
const restoring = ref<BackupFile | null>(null)
const uploadConfirm = ref<File | null>(null)
const phase = ref<'idle' | 'restarting' | 'timed-out'>('idle')
const fileInput = ref<HTMLInputElement | null>(null)

const busy = computed(() => restoring.value !== null || uploadConfirm.value !== null || phase.value !== 'idle')
watch(busy, (value) => emit('busy', value))

async function load(): Promise<void> {
  status.value = await run(() => fetchBackupStatus())
}

async function backUpNow(): Promise<void> {
  const taken = await runBackup(() => takeBackup(), 'Backed up.')
  if (taken) await load()
}

async function waitForTheRestart(): Promise<void> {
  phase.value = 'restarting'
  if (!(await reloadWhenBack())) phase.value = 'timed-out'
}

async function confirmRestore(): Promise<void> {
  const backup = restoring.value
  if (!backup) return
  const previous = await runRestore(() => restoreBackup(backup.name))
  restoring.value = null
  if (previous) await waitForTheRestart()
}

function chooseFile(): void {
  fileInput.value?.click()
}

function onFileChosen(event: Event): void {
  const input = event.target as HTMLInputElement
  uploadConfirm.value = input.files?.[0] ?? null
  input.value = ''
}

async function confirmUpload(): Promise<void> {
  const file = uploadConfirm.value
  if (!file) return
  const previous = await runRestore(() => restoreUploadedBackup(file))
  uploadConfirm.value = null
  if (previous) await waitForTheRestart()
}

onMounted(load)
</script>

<template>
  <section aria-labelledby="backups-heading" data-testid="backups-section">
    <h3 id="backups-heading" class="eyebrow mb-2">Backups</h3>

    <RestartingOverlay v-if="phase === 'restarting'" />

    <p v-else-if="phase === 'timed-out'" class="text-[12.5px] text-warn">
      Rekall is taking longer than expected to come back. Reload the page in a moment.
    </p>

    <p v-else-if="!status" class="text-[12.5px] text-text-subtle">Loading…</p>

    <p v-else-if="!status.available" class="text-[12.5px] leading-relaxed text-text-subtle">
      This database is not a file on disk, so there is nothing to back up.
    </p>

    <template v-else>
      <p class="text-[11.5px] leading-relaxed text-text-subtle">
        A copy of the whole database every {{ status.intervalHours }} hours and before every restore,
        the newest {{ status.keep }} kept, in
        <span class="break-all font-mono text-text-muted">{{ status.folder }}</span>. Download one to move
        this database to another machine, and restore it there.
      </p>

      <p
        v-if="status.lastFailure"
        class="mt-2 rounded-[var(--radius-control)] border border-warn bg-warn-soft px-3 py-2 text-[11.5px] text-warn"
      >
        The last backup failed: {{ status.lastFailure }}
      </p>

      <ul v-if="status.backups.length" class="mt-3 divide-y divide-border rounded-[var(--radius-control)] border border-border">
        <li
          v-for="backup in status.backups"
          :key="backup.name"
          class="flex items-center gap-3 px-3 py-2"
          data-testid="backup-row"
        >
          <span class="min-w-0 flex-1">
            <span class="block text-[12.5px] text-text">{{ relativeTime(backup.createdAt) }}</span>
            <span class="block text-[11px] text-text-subtle">
              {{ BACKUP_REASON_LABEL[backup.reason] }} · {{ readableSize(backup.sizeBytes) }}
            </span>
          </span>
          <a
            :href="backupDownloadUrl(backup.name)"
            download
            class="focus-ring rounded text-[11.5px] text-text-muted underline decoration-border-strong underline-offset-2 hover:text-text"
          >
            Download
          </a>
          <AppButton size="sm" variant="ghost" data-testid="backup-restore" @click="restoring = backup">
            Restore
          </AppButton>
        </li>
      </ul>
      <p v-else class="mt-3 text-[12px] text-text-subtle">No backup yet.</p>

      <div class="mt-3 flex flex-wrap gap-2">
        <AppButton size="sm" variant="secondary" :loading="backingUp" data-testid="backup-now" @click="backUpNow">
          Back up now
        </AppButton>
        <AppButton size="sm" variant="ghost" data-testid="backup-restore-file" @click="chooseFile">
          Restore from a file…
        </AppButton>
        <input ref="fileInput" type="file" accept=".zip,application/zip" class="hidden" @change="onFileChosen" />
      </div>
    </template>

    <Teleport to="body">
      <Transition name="dialog">
        <AppConfirm
          v-if="restoring"
          :title="`Restore the backup from ${relativeTime(restoring.createdAt)}?`"
          body="Everything in this database is replaced by that copy, and Rekall restarts on it. What is here now is backed up first, so this can be undone the same way."
          blast="replaces every company, project, task and note · backed up first"
          confirm-label="Restore and restart"
          @cancel="restoring = null"
          @confirm="confirmRestore"
        />
      </Transition>
      <Transition name="dialog">
        <AppConfirm
          v-if="uploadConfirm"
          :title="`Restore ${uploadConfirm.name}?`"
          body="Everything in this database is replaced by the one in that file, and Rekall restarts on it. What is here now is backed up first, and the file is kept with the other backups."
          blast="replaces every company, project, task and note · backed up first"
          confirm-label="Restore and restart"
          @cancel="uploadConfirm = null"
          @confirm="confirmUpload"
        />
      </Transition>
    </Teleport>
  </section>
</template>
