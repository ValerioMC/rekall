<script setup lang="ts">
import { computed } from 'vue'

const props = defineProps<{ sql: string }>()

const KEYWORDS =
  /\b(create|table|alter|drop|add|column|constraint|primary key|foreign key|references|on delete|not null|default|index|using|trigger|before|update|for each row|execute function|set|where|is null|check|in|cascade|restrict|gin)\b/gi

/**
 * Highlights keywords, identifiers and literals.
 *
 * The plan screen exists so the SQL gets read rather than skimmed, and an undifferentiated
 * wall of monospace does not get read.
 *
 * The result is rendered with v-html, which the linter is disabled for in the template below.
 * That is safe here for a specific reason: the input is escaped on the first three lines of
 * this function, and every tag added afterwards is a literal in this file. No part of the SQL
 * string can reach the output as markup.
 */
const highlighted = computed(() => {
  const escaped = props.sql
    .replace(/&/g, '&amp;')
    .replace(/</g, '&lt;')
    .replace(/>/g, '&gt;')

  return escaped
    .replace(/&quot;|"([^"]*)"/g, (_match, name: string) => `<span class="text-accent">"${name}"</span>`)
    .replace(/'([^']*)'/g, (_match, value: string) => `<span class="text-safe">'${value}'</span>`)
    .replace(KEYWORDS, (keyword) => `<span class="text-text-muted font-semibold">${keyword}</span>`)
})
</script>

<template>
  <!-- eslint-disable vue/no-v-html -->
  <pre
    data-testid="plan-sql"
    class="overflow-x-auto rounded-[var(--radius-control)] border border-border bg-canvas px-3.5 py-3 font-mono text-[12px] leading-relaxed text-text-subtle"
  ><code v-html="highlighted" /></pre>
</template>
