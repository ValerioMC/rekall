import { ref } from 'vue'
import { defineStore } from 'pinia'
import { fetchCodexModels, fetchCodexSetup, installCodexIntegration } from '@/features/codex/api/codex.api'
import type { CodexModel, CodexSetup } from '@/features/codex/model/codex'

export const useCodexStore = defineStore('codex', () => {
  const setup = ref<CodexSetup | null>(null)
  const models = ref<CodexModel[]>([])
  async function loadSetup(): Promise<void> {
    setup.value = await fetchCodexSetup()
  }

  async function loadModels(): Promise<void> {
    models.value = await fetchCodexModels()
  }

  async function install(): Promise<void> {
    setup.value = await installCodexIntegration()
  }

  return { setup, models, loadSetup, loadModels, install }
})
