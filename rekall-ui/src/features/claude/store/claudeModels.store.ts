import { ref } from 'vue'
import { defineStore } from 'pinia'
import { fetchClaudeModels } from '@/features/claude/api/claude.api'
import type { ClaudeModel } from '@/features/claude/model/claude'

/** The installed Claude Code's models. A failed read keeps the last answered list and raises `failed`. */
export const useClaudeModelsStore = defineStore('claudeModels', () => {
  const models = ref<readonly ClaudeModel[] | null>(null)
  const reading = ref(false)
  const failed = ref(false)

  async function load(): Promise<void> {
    reading.value = true
    try {
      models.value = await fetchClaudeModels()
      failed.value = false
    } catch {
      failed.value = true
    } finally {
      reading.value = false
    }
  }

  return { models, reading, failed, load }
})
