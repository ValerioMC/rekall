<script setup lang="ts">
import { computed } from 'vue'
import { HELP_TOOLS, type HelpToolName } from '@/features/settings/model/help-context'

const props = defineProps<{ names: readonly HelpToolName[] }>()

const tools = computed(() => props.names.map((name) => HELP_TOOLS[name]))
</script>

<template>
  <section class="flex flex-col gap-2" aria-labelledby="help-tools" data-testid="help-tools">
    <h3 id="help-tools" class="text-title font-semibold text-text">MCP tools</h3>
    <ul class="divide-y divide-border overflow-hidden rounded-card border border-border bg-surface">
      <li v-for="tool in tools" :key="tool.name" class="flex items-start gap-3 px-4 py-2.5" data-testid="help-tool">
        <div class="min-w-0 flex-1">
          <code class="block break-words font-mono text-small text-text">{{ tool.name }}</code>
          <p class="mt-0.5 text-small text-text-muted">{{ tool.effect }}</p>
        </div>
        <span
          class="mt-0.5 shrink-0 rounded-key border border-border-strong px-1.5 font-mono text-micro"
          :class="tool.access === 'write' ? 'text-text' : 'text-text-subtle'"
          :data-testid="`help-tool-access-${tool.access}`"
        >
          {{ tool.access }}
        </span>
      </li>
    </ul>
  </section>
</template>
