<script setup lang="ts">
import { storeToRefs } from 'pinia'
import AccountUsageMeter from '@/components/console/AccountUsageMeter.vue'
import { useAccountUsageReading } from '@/composables/useAccountUsageReading'
import { useNow } from '@/composables/useNow'
import { fetchClaudeAccountUsage } from '@/features/claude/lib/claude-account-usage'
import { fetchCodexAccountUsage } from '@/features/codex/lib/codex-account-usage'
import QueueBeacon from '@/features/queue/components/QueueBeacon.vue'
import ReviewQueueButton from '@/features/catalog/components/ReviewQueueButton.vue'
import { useConsoleStore } from '@/stores/console.store'

const { reviewQueue } = storeToRefs(useConsoleStore())
const now = useNow(15_000)
const claude = useAccountUsageReading(now, fetchClaudeAccountUsage)
const codex = useAccountUsageReading(now, fetchCodexAccountUsage)
const { available: codexInstalled } = codex
</script>

<template>
  <div
    class="header-capsule min-w-0 shrink"
    role="group"
    aria-label="Live status"
    data-testid="header-status"
  >
    <AccountUsageMeter provider="CLAUDE" :reading="claude" :show-provider="codexInstalled" />
    <template v-if="codexInstalled">
      <span class="h-4 w-px bg-border" aria-hidden="true" />
      <AccountUsageMeter provider="CODEX" :reading="codex" show-provider />
    </template>
    <span class="h-4 w-px bg-border" aria-hidden="true" />
    <QueueBeacon />
    <template v-if="reviewQueue.length">
      <span class="h-4 w-px bg-border" aria-hidden="true" />
      <ReviewQueueButton />
    </template>
  </div>
</template>
