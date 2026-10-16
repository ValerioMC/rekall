<script setup lang="ts">
import { storeToRefs } from 'pinia'
import { useClaudeConnectionStore } from '@/features/claude/store/claudeConnection.store'

defineEmits<{ open: [] }>()

const { needsRepair } = storeToRefs(useClaudeConnectionStore())
</script>

<template>
  <button
    type="button"
    class="header-key focus-ring relative"
    data-testid="settings-trigger"
    :aria-label="needsRepair ? 'Settings: Claude Code needs repair' : 'Settings'"
    :title="needsRepair ? 'Claude Code needs repair. Open Settings to fix it.' : 'Settings'"
    @click="$emit('open')"
  >
    <svg class="size-4" viewBox="0 0 24 24" fill="none" aria-hidden="true">
      <path
        d="M12 15a3 3 0 1 0 0-6 3 3 0 0 0 0 6Z"
        stroke="currentColor"
        stroke-width="1.6"
        stroke-linecap="round"
        stroke-linejoin="round"
      />
      <path
        d="M12.22 2h-.44a2 2 0 0 0-2 2v.18a2 2 0 0 1-1 1.73l-.43.25a2 2 0 0 1-2 0l-.15-.08a2 2 0 0 0-2.73.73l-.22.38a2 2 0 0 0 .73 2.73l.15.1a2 2 0 0 1 1 1.72v.51a2 2 0 0 1-1 1.74l-.15.09a2 2 0 0 0-.73 2.73l.22.38a2 2 0 0 0 2.73.73l.15-.08a2 2 0 0 1 2 0l.43.25a2 2 0 0 1 1 1.73V20a2 2 0 0 0 2 2h.44a2 2 0 0 0 2-2v-.18a2 2 0 0 1 1-1.73l.43-.25a2 2 0 0 1 2 0l.15.08a2 2 0 0 0 2.73-.73l.22-.39a2 2 0 0 0-.73-2.73l-.15-.08a2 2 0 0 1-1-1.74v-.5a2 2 0 0 1 1-1.74l.15-.09a2 2 0 0 0 .73-2.73l-.22-.38a2 2 0 0 0-2.73-.73l-.15.08a2 2 0 0 1-2 0l-.43-.25a2 2 0 0 1-1-1.73V4a2 2 0 0 0-2-2z"
        stroke="currentColor"
        stroke-width="1.6"
        stroke-linecap="round"
        stroke-linejoin="round"
      />
    </svg>
    <!-- A triangle, not a dot: a dot beside a cog reads as "new", this has to read as "broken". -->
    <svg
      v-if="needsRepair"
      class="absolute -right-0.5 -top-0.5 size-3 text-warn"
      viewBox="0 0 12 12"
      fill="none"
      data-testid="settings-warning"
      aria-hidden="true"
    >
      <path d="M6 1.2 11 10.2H1L6 1.2Z" fill="currentColor" stroke="var(--color-canvas)" stroke-width="1" stroke-linejoin="round" />
      <path d="M6 4.6v2.6M6 8.6v.1" stroke="var(--color-canvas)" stroke-width="1.2" stroke-linecap="round" />
    </svg>
  </button>
</template>
