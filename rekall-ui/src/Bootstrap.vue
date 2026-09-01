<script setup lang="ts">
import { onMounted, onUnmounted, ref } from 'vue'
import FirstRunSetup from '@/components/setup/FirstRunSetup.vue'
import DatabaseUnreachable from '@/components/setup/DatabaseUnreachable.vue'
import RunningTasksDock from '@/components/shell/RunningTasksDock.vue'
import AppLogo from '@/components/ui/AppLogo.vue'
import { fetchDatabaseStatus } from '@/api/settings.api'
import { useConsoleStore } from '@/stores/console.store'
import { useToastStore } from '@/stores/toast.store'
import type { DatabaseStatus } from '@/model/settings'

const store = useConsoleStore()
const toast = useToastStore()
const status = ref<DatabaseStatus | null>(null)
const failed = ref(false)

let refreshing = false

/**
 * What changed while this window was in the background.
 *
 * A wrapup is written by a Claude session through MCP, and the window that was open when it
 * happened knows nothing about it: it would go on showing an empty card until someone reloaded
 * the page. Coming back to the window is the moment the answer is wanted, so that is when it is
 * read again.
 *
 * Not while something here is waiting to be saved. A pane holds the draft being typed, and
 * replacing the record underneath it mid-edit is how a paragraph disappears.
 */
async function refreshWhatChangedElsewhere(): Promise<void> {
  if (refreshing || status.value?.status !== 'READY') return
  if (document.visibilityState !== 'visible' || store.saveState !== 'saved') return
  refreshing = true
  try {
    await store.refreshEverything()
  } catch (caught) {
    toast.notifyError(caught)
  } finally {
    refreshing = false
  }
}

onMounted(async () => {
  try {
    status.value = await fetchDatabaseStatus()
    if (status.value.status === 'READY') await store.load()
  } catch {
    failed.value = true
  }
  // Both, because they do not fire together: switching applications is a focus change, and
  // hiding the window or the whole application is a visibility change.
  window.addEventListener('focus', refreshWhatChangedElsewhere)
  document.addEventListener('visibilitychange', refreshWhatChangedElsewhere)
})

onUnmounted(() => {
  window.removeEventListener('focus', refreshWhatChangedElsewhere)
  document.removeEventListener('visibilitychange', refreshWhatChangedElsewhere)
})
</script>

<template>
  <template v-if="status">
    <template v-if="status.status === 'READY'">
      <router-view />
      <RunningTasksDock />
    </template>
    <FirstRunSetup v-else-if="status.status === 'SETUP_NEEDED'" />
    <DatabaseUnreachable v-else :status="status" />
  </template>
  <div
    v-else-if="failed"
    class="grid h-full place-items-center p-6 text-center text-[13px] text-text-subtle"
  >
    Rekall couldn't be reached. Make sure it's running, then reload the page.
  </div>
  <!-- The one instant nothing is known yet: which of the three screens above this becomes.
       A blank canvas reads as broken; the mark fading in reads as a product about to answer. -->
  <div v-else class="fade-in grid h-full place-items-center" role="status" aria-live="polite">
    <span class="sr-only">Loading</span>
    <AppLogo :size="40" class="halo animate-pulse rounded-[9px]" aria-hidden="true" />
  </div>
</template>
