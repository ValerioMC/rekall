import { defineStore } from 'pinia'
import { computed, ref } from 'vue'
import { fetchClaudeInstallation, installClaudeIntegration } from '@/api/claude.api'
import type { ClaudeInstallation } from '@/model/claude'

/**
 * How Claude Code is registered against this instance. Shared by the Settings section and the
 * warning on the Settings key, so repairing in one clears the other.
 */
export const useClaudeConnectionStore = defineStore('claudeConnection', () => {
  const installation = ref<ClaudeInstallation | null>(null)

  /** Only a registration that exists but is stale: the Repair button is the cure for exactly this. */
  const needsRepair = computed<boolean>(() => installation.value?.status === 'OUTDATED')

  async function load(): Promise<void> {
    installation.value = await fetchClaudeInstallation()
  }

  async function install(): Promise<void> {
    installation.value = await installClaudeIntegration()
  }

  return { installation, needsRepair, load, install }
})
