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

const section = computed(() => settingsSectionOf(typeof route.params.section === 'string' ? route.params.section : undefined))
const helpTopic = computed(() => helpContextOf(typeof route.params.topic === 'string' ? route.params.topic : undefined).id)

// Settings is where the Claude registration gets repaired or changed by hand, so read it again on leaving.
onUnmounted(() => void claudeConnection.load().catch(() => undefined))
</script>

<template>
  <div class="min-h-0 flex-1 page-scroll bg-canvas" data-testid="settings-page">
    <AppCatalogHeader title="Settings" />
    <div class="mx-auto grid max-w-[1100px] gap-8 px-8 py-6 min-[860px]:grid-cols-[220px_minmax(0,1fr)]">
      <SettingsRail :section="section" :topic="helpTopic" />

      <main class="min-w-0" :class="section === 'help' ? 'max-w-[820px]' : 'max-w-[620px]'">
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
        <HelpSection v-else-if="section === 'help'" :context-id="helpTopic" />
        <VersionSection v-else />
      </main>
    </div>
  </div>
</template>
