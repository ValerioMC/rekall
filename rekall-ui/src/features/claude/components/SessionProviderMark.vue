<script setup lang="ts">
import { SESSION_PROVIDER_PATH } from '@/features/claude/lib/session-provider-glyph'
import type { SessionProvider } from '@/features/claude/model/session-provider'

/** The mark of the company behind a session provider: Claude in its coral, OpenAI in the text tone. */
withDefaults(defineProps<{ provider: SessionProvider; size?: number }>(), { size: 14 })

const BRAND: Readonly<Record<SessionProvider, string>> = {
  CLAUDE: 'text-brand-claude',
  CODEX: 'text-text'
}
</script>

<template>
  <svg
    class="shrink-0"
    :class="BRAND[provider]"
    :width="size"
    :height="size"
    viewBox="0 0 24 24"
    fill="currentColor"
    aria-hidden="true"
    :data-provider="provider"
  >
    <path :d="SESSION_PROVIDER_PATH[provider]" />
  </svg>
</template>
