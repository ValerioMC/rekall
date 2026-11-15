<script setup lang="ts">
import { computed, useId } from 'vue'
import { HELP_TOOLS, type HelpToolName } from '@/features/settings/model/help-context'

const props = defineProps<{ names: readonly HelpToolName[] }>()

const tools = computed(() => props.names.map((name) => HELP_TOOLS[name]))
const headingId = useId()
</script>

<template>
  <section class="flex flex-col gap-1" :aria-labelledby="headingId" data-testid="help-tools">
    <h4 :id="headingId" class="text-small font-semibold text-text-muted">Also ask directly, with no command</h4>
    <ul class="divide-y divide-border border-y border-border">
      <li v-for="tool in tools" :key="tool.name" class="flex items-start gap-3 py-3" data-testid="help-tool">
        <div class="flex min-w-0 flex-1 flex-col gap-1">
          <code class="block break-words font-mono text-small text-text">{{ tool.name }}</code>
          <p class="text-small text-text-muted">{{ tool.effect }}</p>
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
