<script setup lang="ts">
import { useId } from 'vue'
import type { HelpExample } from '@/features/settings/model/help-context'
import { helpText, type HelpValues } from '@/features/settings/lib/help-template'
import HelpCalls from '@/features/settings/components/HelpCalls.vue'
import HelpCopyButton from '@/features/settings/components/HelpCopyButton.vue'

defineProps<{ examples: readonly HelpExample[]; values: HelpValues }>()

const headingId = useId()
</script>

<template>
  <section class="flex flex-col gap-1" :aria-labelledby="headingId" data-testid="help-examples">
    <h4 :id="headingId" class="text-small font-semibold text-text-muted">Ask in plain words</h4>
    <ul class="divide-y divide-border border-y border-border">
      <li v-for="example in examples" :key="example.title" class="flex items-start gap-3 py-3" data-testid="help-example">
        <div class="flex min-w-0 flex-1 flex-col gap-1">
          <h5 class="text-body font-medium text-text">{{ example.title }}</h5>
          <p class="text-body text-text-muted" data-testid="help-example-prompt">{{ helpText(example.prompt, values) }}</p>
          <HelpCalls :names="example.tools" />
        </div>
        <HelpCopyButton :text="helpText(example.prompt, values)" :label="`Copy the request: ${example.title}`" />
      </li>
    </ul>
  </section>
</template>
