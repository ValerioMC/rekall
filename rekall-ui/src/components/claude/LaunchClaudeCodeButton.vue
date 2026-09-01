<script setup lang="ts">
import { computed, ref } from 'vue'
import AppButton from '@/components/ui/AppButton.vue'
import { canLaunchClaudeCode, launchClaudeCode } from '@/common/native/desktop'
import { skipsPermissions } from '@/common/config/claude-launch'
import { useToastStore } from '@/stores/toast.store'

/**
 * The shortest path between a record and a session that has read it.
 *
 * A terminal opens in the project's folder with `/rk` already running, so the anchor is never
 * copied, pasted or typed. It exists only inside the macOS application: a browser tab cannot
 * open a terminal, and the endpoint that would let it is one any other page could call.
 *
 * Without a folder the button stays and says so when it is pressed. Not disabled: a disabled
 * button shows no tooltip in this window, so the one explanation there was would be invisible to
 * exactly the person who needs it, on a button that looks broken.
 */
const props = defineProps<{
  /** The anchors as `/rk` takes them, which the server built. */
  anchors: string
  /** The project's folder, or null when nobody has set one. */
  folder: string | null
  /** Where to go and set it, in words, for the case where it is not on this screen. */
  missingHint?: string
}>()

const toast = useToastStore()

const available = canLaunchClaudeCode()
const launching = ref(false)

const missing = computed(() => props.missingHint ?? 'Set this project’s folder to open a session from it')

const title = computed(() =>
  props.folder ? `Opens a terminal in ${props.folder} running /rk ${props.anchors}` : missing.value
)

async function launch(): Promise<void> {
  if (launching.value) return
  if (!props.folder) {
    toast.notifyError(new Error(missing.value))
    return
  }
  launching.value = true
  try {
    const terminal = await launchClaudeCode({
      directory: props.folder,
      anchors: props.anchors,
      skipPermissions: skipsPermissions()
    })
    toast.notify(`Opened in ${terminal}.`)
  } catch (caught) {
    toast.notifyError(caught)
  } finally {
    launching.value = false
  }
}
</script>

<template>
  <AppButton
    v-if="available"
    size="sm"
    :variant="folder ? 'secondary' : 'ghost'"
    :loading="launching"
    :title="title"
    data-testid="launch-claude-code"
    @click="launch"
  >
    Open in Claude Code
  </AppButton>
</template>
