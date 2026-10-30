<script setup lang="ts">
import { computed, defineAsyncComponent, onMounted, onUnmounted, ref } from 'vue'
import { useRoute } from 'vue-router'
import FirstRunSetup from '@/features/settings/components/FirstRunSetup.vue'
import DatabaseUnreachable from '@/features/settings/components/DatabaseUnreachable.vue'
import ShellDock from '@/components/shell/ShellDock.vue'
import AppHeaderChrome from '@/components/shell/AppHeaderChrome.vue'
import LeaveDialog from '@/components/shell/LeaveDialog.vue'
import UpdateDialog from '@/components/shell/UpdateDialog.vue'
import AppLogo from '@/components/ui/AppLogo.vue'
import { fetchDatabaseStatus } from '@/features/settings/api/settings.api'
import { useConsoleStore } from '@/stores/console.store'
import { useTerminalStore } from '@/features/claude/store/terminal.store'
import { useUpdateStore } from '@/features/settings/store/update.store'
import { useToastStore } from '@/stores/toast.store'
import { useTokenUsageStore } from '@/features/tokens/store/tokenUsage.store'
import { useVulnerabilityStore } from '@/features/vulnerability/store/vulnerability.store'
import { useChatDockArea } from '@/features/chat/composables/useChatDockArea'
import { useChatWindowShown } from '@/features/chat/composables/useChatWindowShown'
import { useRevealOpenedChat } from '@/features/chat/composables/useRevealOpenedChat'
import type { DatabaseStatus } from '@/features/settings/model/settings'

// Loaded only once a chat session exists, like the terminal pane.
const ChatWindow = defineAsyncComponent(() => import('@/features/chat/components/ChatWindow.vue'))

const store = useConsoleStore()
const terminals = useTerminalStore()
const update = useUpdateStore()
const toast = useToastStore()
const status = ref<DatabaseStatus | null>(null)
const failed = ref(false)
const route = useRoute()
const pageArea = ref<HTMLElement | null>(null)
const chatWindowShown = useChatWindowShown()

// On the console the window docks beside the note pane, which App.vue arranges itself.
const chatOverPage = computed(() => route.name !== 'console' && chatWindowShown.value)
useChatDockArea(pageArea, false, () => route.name !== 'console')
useRevealOpenedChat()

let refreshing = false

async function refreshWhatChangedElsewhere(): Promise<void> {
  if (refreshing || status.value?.status !== 'READY') return
  if (document.visibilityState !== 'visible' || store.saveState !== 'saved') return
  refreshing = true
  try {
    await Promise.all([store.refreshEverything(), terminals.load(), useTokenUsageStore().load()])
  } catch (caught) {
    toast.notifyError(caught)
  } finally {
    refreshing = false
  }
}

onMounted(async () => {
  try {
    status.value = await fetchDatabaseStatus()
    if (status.value.status === 'READY') {
      await store.load()
      void terminals.load()
      // Opened only once the database is there: the store's stream would otherwise retry forever.
      void useTokenUsageStore().load().catch((caught: unknown) => toast.notifyError(caught))
      void useVulnerabilityStore().load().catch((caught: unknown) => toast.notifyError(caught))
      void update.checkAtStartup()
    }
  } catch {
    failed.value = true
  }
  window.addEventListener('focus', refreshWhatChangedElsewhere)
  document.addEventListener('visibilitychange', refreshWhatChangedElsewhere)
})

onUnmounted(() => {
  window.removeEventListener('focus', refreshWhatChangedElsewhere)
  document.removeEventListener('visibilitychange', refreshWhatChangedElsewhere)
})
</script>

<template>
  <LeaveDialog />
  <template v-if="status">
    <template v-if="status.status === 'READY'">
      <div class="flex h-full flex-col">
        <AppHeaderChrome />
        <div ref="pageArea" class="relative flex min-h-0 flex-1 flex-col">
          <router-view v-slot="{ Component }">
            <Transition name="pane" mode="out-in">
              <component :is="Component" />
            </Transition>
          </router-view>
          <Transition name="pane">
            <ChatWindow v-if="chatOverPage" />
          </Transition>
        </div>
      </div>
      <ShellDock />
      <UpdateDialog v-if="update.promptOpen" />
    </template>
    <FirstRunSetup v-else-if="status.status === 'SETUP_NEEDED'" />
    <DatabaseUnreachable v-else :status="status" />
  </template>
  <div
    v-else-if="failed"
    class="grid h-full place-items-center p-6 text-center text-body text-text-subtle"
  >
    Rekall couldn't be reached. Make sure it's running, then reload the page.
  </div>
  <div v-else class="fade-in grid h-full place-items-center" role="status" aria-live="polite">
    <span class="sr-only">Loading</span>
    <AppLogo :size="40" class="halo animate-pulse rounded-card" aria-hidden="true" />
  </div>
</template>
