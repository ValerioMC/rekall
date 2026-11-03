<script setup lang="ts">
import StepRowControls from '@/features/steps/components/StepRowControls.vue'
import type { DoneCheck } from '@/features/check/model/check'

/** One check as it will run: its name, command, folder and timeout, with the controls to change it. */
defineProps<{ check: DoneCheck; canMoveUp: boolean; canMoveDown: boolean }>()

defineEmits<{ edit: []; up: []; down: []; remove: [] }>()
</script>

<template>
  <li class="group/check flex items-start gap-2 rounded-control px-2 py-1.5 hover:bg-surface-hover" data-testid="check-row">
    <div class="min-w-0 flex-1">
      <span class="block text-body text-text" data-testid="check-row-label">{{ check.label }}</span>
      <code class="block truncate font-mono text-caption text-text-muted" :title="check.command" data-testid="check-row-command">{{ check.command }}</code>
      <span class="text-micro text-text-subtle" data-testid="check-row-meta">
        {{ check.folder ? `in ${check.folder}` : 'in the project folder' }}, stops after {{ check.timeoutSeconds }} s
      </span>
    </div>
    <div class="flex shrink-0 items-center gap-0.5 opacity-60 focus-within:opacity-100 group-hover/check:opacity-100">
      <button
        class="focus-ring mr-0.5 grid h-6 place-items-center rounded-key px-1.5 text-micro font-medium text-text-subtle transition-colors hover:bg-surface-hover hover:text-text"
        :aria-label="`Edit ${check.label}`"
        data-testid="check-edit"
        @click="$emit('edit')"
      >
        Edit
      </button>
      <StepRowControls
        :title="check.label"
        :can-move-up="canMoveUp"
        :can-move-down="canMoveDown"
        testid-prefix="check"
        @up="$emit('up')"
        @down="$emit('down')"
        @remove="$emit('remove')"
      />
    </div>
  </li>
</template>
