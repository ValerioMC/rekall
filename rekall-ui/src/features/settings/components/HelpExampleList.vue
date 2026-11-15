<script setup lang="ts">
import type { HelpExample } from '@/features/settings/model/help-context'
import HelpCopyButton from '@/features/settings/components/HelpCopyButton.vue'

defineProps<{ examples: readonly HelpExample[] }>()
</script>

<template>
  <section class="flex flex-col gap-2" aria-labelledby="help-examples" data-testid="help-examples">
    <h3 id="help-examples" class="text-title font-semibold text-text">Ask it like this</h3>
    <ul class="flex flex-col gap-3">
      <li v-for="example in examples" :key="example.title" class="rounded-card border border-border bg-surface" data-testid="help-example">
        <div class="flex items-center justify-between gap-3 border-b border-border px-4 py-2">
          <h4 class="text-body font-medium text-text">{{ example.title }}</h4>
          <span class="flex shrink-0 flex-wrap justify-end gap-x-3 font-mono text-micro text-text-subtle">
            <span v-for="tool in example.tools" :key="tool">{{ tool }}</span>
          </span>
        </div>
        <div class="flex items-start gap-3 px-4 py-3">
          <p class="min-w-0 flex-1 text-body text-text" data-testid="help-example-prompt">{{ example.prompt }}</p>
          <HelpCopyButton :text="example.prompt" :label="`Copy the request: ${example.title}`" />
        </div>
      </li>
    </ul>
  </section>
</template>
