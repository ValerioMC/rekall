<script setup lang="ts">
import { computed, onUnmounted, ref, watch } from 'vue'
import { useRoute } from 'vue-router'
import AppCatalogHeader from '@/features/catalog/components/AppCatalogHeader.vue'
import FormPanel from '@/components/ui/FormPanel.vue'
import PictureCleanupPanel from '@/features/picture/components/PictureCleanupPanel.vue'
import AppearanceSection from '@/features/settings/components/AppearanceSection.vue'
import BackupsSection from '@/features/settings/components/BackupsSection.vue'
import { preferredProvider, setPreferredProvider } from '@/common/config/session-launch'
import CodexSection from '@/features/settings/components/CodexSection.vue'
import SessionProviderSection from '@/features/settings/components/SessionProviderSection.vue'
import ClaudeCodeSection from '@/features/settings/components/ClaudeCodeSection.vue'
import ConnectionsSection from '@/features/settings/components/ConnectionsSection.vue'
import DatabaseSection from '@/features/settings/components/DatabaseSection.vue'
import GuardProtectionSection from '@/features/settings/components/GuardProtectionSection.vue'
import HelpSection from '@/features/settings/components/HelpSection.vue'
import KeyboardSection from '@/features/settings/components/KeyboardSection.vue'
import LangfuseSection from '@/features/settings/components/LangfuseSection.vue'
import NotificationsSection from '@/features/settings/components/NotificationsSection.vue'
import ReviewSection from '@/features/settings/components/ReviewSection.vue'
import TimeUsageSection from '@/features/settings/components/TimeUsageSection.vue'
import VersionSection from '@/features/settings/components/VersionSection.vue'
import SettingsRail from '@/features/settings/components/SettingsRail.vue'
import { helpContextOf } from '@/features/settings/model/help-context'
import { settingsSectionOf } from '@/features/settings/model/settings-section'
import { useClaudeConnectionStore } from '@/features/claude/store/claudeConnection.store'

/** Settings at `/settings/:section?/:topic?`: a grouped rail beside the section open; Help takes a topic from the rail. */
const route = useRoute()
const claudeConnection = useClaudeConnectionStore()
const provider = ref(preferredProvider())
watch(provider, setPreferredProvider, { flush: 'sync' })

const section = computed(() => settingsSectionOf(typeof route.params.section === 'string' ? route.params.section : undefined))
const helpTopic = computed(() => helpContextOf(typeof route.params.topic === 'string' ? route.params.topic : undefined).id)

// Settings is where the Claude registration gets repaired or changed by hand, so read it again on leaving.
onUnmounted(() => void claudeConnection.load().catch(() => undefined))
</script>

<template>
  <div class="min-h-0 flex-1 page-scroll bg-canvas" data-testid="settings-page">
    <AppCatalogHeader title="Settings" />
    <div class="grid gap-8 px-8 py-6 min-[860px]:grid-cols-[220px_minmax(0,1fr)]">
      <SettingsRail :section="section" :topic="helpTopic" />

      <main class="min-w-0">
        <HelpSection v-if="section === 'help'" :context-id="helpTopic" />
        <div v-else class="form-grid">
          <FormPanel v-if="section === 'general'"><DatabaseSection /></FormPanel>
          <AppearanceSection v-else-if="section === 'appearance'" />
          <ReviewSection v-else-if="section === 'review'" />
          <template v-else-if="section === 'sessions'">
            <FormPanel class="overflow-visible"><SessionProviderSection v-model="provider" /></FormPanel>
            <FormPanel v-if="provider === 'CLAUDE'"><ClaudeCodeSection /></FormPanel>
            <FormPanel v-else><CodexSection /></FormPanel>
            <FormPanel><GuardProtectionSection /></FormPanel>
          </template>
          <FormPanel v-else-if="section === 'notifications'"><NotificationsSection /></FormPanel>
          <TimeUsageSection v-else-if="section === 'time'" />
          <FormPanel v-else-if="section === 'connections'"><ConnectionsSection /></FormPanel>
          <template v-else-if="section === 'data'">
            <PictureCleanupPanel class="col-span-full" />
            <FormPanel title="Backups"><BackupsSection class="p-5" /></FormPanel>
            <FormPanel title="Langfuse"><LangfuseSection class="p-5" /></FormPanel>
          </template>
          <FormPanel v-else-if="section === 'keyboard'"><KeyboardSection /></FormPanel>
          <FormPanel v-else><VersionSection /></FormPanel>
        </div>
      </main>
    </div>
  </div>
</template>
