<script setup lang="ts">
import type { HelpContextId } from '@/features/settings/model/help-context'
import { SETTINGS_GROUPS, SETTINGS_SECTION_LABEL, type SettingsSection } from '@/features/settings/model/settings-section'
import SettingsHelpTopics from '@/features/settings/components/SettingsHelpTopics.vue'

defineProps<{ section: SettingsSection; topic: HelpContextId }>()
</script>

<template>
  <nav
    class="flex flex-col gap-5 min-[860px]:sticky min-[860px]:top-4 min-[860px]:max-h-[calc(100vh-7rem)] min-[860px]:self-start min-[860px]:overflow-y-auto"
    aria-label="Settings sections"
    data-testid="settings-rail"
  >
    <div v-for="group in SETTINGS_GROUPS" :key="group.label" class="flex flex-col gap-0.5">
      <p class="px-3 pb-1 text-caption text-text-subtle">{{ group.label }}</p>
      <div class="flex flex-row flex-wrap gap-1 min-[860px]:flex-col min-[860px]:gap-0.5">
        <template v-for="id in group.sections" :key="id">
          <RouterLink
            :to="`/settings/${id}`"
            class="focus-ring rounded-control px-3 py-2 text-body transition-colors duration-(--motion-quick)"
            :class="section === id ? 'bg-surface-raised text-text' : 'text-text-muted hover:bg-surface-hover hover:text-text'"
            :aria-current="section === id ? 'page' : undefined"
            :data-testid="`settings-rail-${id}`"
          >
            {{ SETTINGS_SECTION_LABEL[id] }}
          </RouterLink>
          <SettingsHelpTopics v-if="id === 'help' && section === 'help'" :topic="topic" />
        </template>
      </div>
    </div>
  </nav>
</template>
