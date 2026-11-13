<script setup lang="ts">
import { computed, onUnmounted } from 'vue'
import { useRoute } from 'vue-router'
import AppCatalogHeader from '@/features/catalog/components/AppCatalogHeader.vue'
import AppearanceSection from '@/features/settings/components/AppearanceSection.vue'
import BackupsSection from '@/features/settings/components/BackupsSection.vue'
import ClaudeCodeSection from '@/features/settings/components/ClaudeCodeSection.vue'
import ConnectionsSection from '@/features/settings/components/ConnectionsSection.vue'
import DatabaseSection from '@/features/settings/components/DatabaseSection.vue'
import GuardProtectionSection from '@/features/settings/components/GuardProtectionSection.vue'
import KeyboardSection from '@/features/settings/components/KeyboardSection.vue'
import LangfuseSection from '@/features/settings/components/LangfuseSection.vue'
import NotificationsSection from '@/features/settings/components/NotificationsSection.vue'
import ReviewSection from '@/features/settings/components/ReviewSection.vue'
import TimeUsageSection from '@/features/settings/components/TimeUsageSection.vue'
import VersionSection from '@/features/settings/components/VersionSection.vue'
import { SETTINGS_SECTIONS, SETTINGS_SECTION_LABEL, settingsSectionOf } from '@/features/settings/model/settings-section'
import { useClaudeConnectionStore } from '@/features/claude/store/claudeConnection.store'

/** Settings at `/settings/:section?`: a rail of sections beside the one open. */
const route = useRoute()
const claudeConnection = useClaudeConnectionStore()

const section = computed(() => settingsSectionOf(typeof route.params.section === 'string' ? route.params.section : undefined))

// Settings is where the Claude registration gets repaired or changed by hand, so read it again on leaving.
onUnmounted(() => void claudeConnection.load().catch(() => undefined))
</script>

<template>
  <div class="min-h-0 flex-1 overflow-y-auto bg-canvas" data-testid="settings-page">
    <AppCatalogHeader title="Settings" />
    <div class="mx-auto grid max-w-[1100px] gap-8 px-8 py-6 min-[860px]:grid-cols-[220px_minmax(0,1fr)]">
      <nav class="flex flex-row flex-wrap gap-1 min-[860px]:flex-col" aria-label="Settings sections" data-testid="settings-rail">
        <RouterLink
          v-for="id in SETTINGS_SECTIONS"
          :key="id"
          :to="`/settings/${id}`"
          class="focus-ring rounded-control px-3 py-2 text-body transition-colors"
          :class="section === id ? 'bg-surface-raised text-text' : 'text-text-muted hover:bg-surface-hover hover:text-text'"
          :aria-current="section === id ? 'page' : undefined"
          :data-testid="`settings-rail-${id}`"
        >
          {{ SETTINGS_SECTION_LABEL[id] }}
        </RouterLink>
      </nav>

      <main class="min-w-0 max-w-[620px]">
        <DatabaseSection v-if="section === 'general'" />
        <AppearanceSection v-else-if="section === 'appearance'" />
        <ReviewSection v-else-if="section === 'review'" />
        <div v-else-if="section === 'sessions'" class="flex flex-col gap-6">
          <h2 class="text-heading font-medium text-text">Sessions and run queue</h2>
          <ClaudeCodeSection />
          <GuardProtectionSection class="border-t border-border pt-5" />
        </div>
        <NotificationsSection v-else-if="section === 'notifications'" />
        <TimeUsageSection v-else-if="section === 'time'" />
        <ConnectionsSection v-else-if="section === 'connections'" />
        <div v-else-if="section === 'data'" class="flex flex-col gap-6">
          <BackupsSection />
          <LangfuseSection class="border-t border-border pt-5" />
        </div>
        <KeyboardSection v-else-if="section === 'keyboard'" />
        <VersionSection v-else />
      </main>
    </div>
  </div>
</template>
