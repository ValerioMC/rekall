import { computed, onMounted } from 'vue'
import { storeToRefs } from 'pinia'
import { useAsyncAction } from '@/composables/useAsyncAction'
import { useClaudeResourcesStore } from '@/features/claude-resources/store/claudeResources.store'
import { skillOptions } from '@/features/skills/model/skill-options'

/** The skills Claude Code can invoke, read once from `~/.claude` the first time anything asks. */
export function useSkillCatalog() {
  const resources = useClaudeResourcesStore()
  const { catalog, loaded } = storeToRefs(resources)
  const { run } = useAsyncAction()

  const options = computed(() => skillOptions(catalog.value?.skills ?? []))
  const names = computed(() => new Set(options.value.map((option) => option.name)))

  onMounted(() => {
    if (!loaded.value) void run(() => resources.load())
  })

  return { options, names, loaded }
}
