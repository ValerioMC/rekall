<script setup lang="ts">
import { ref, watch } from 'vue'
import AppBadge from '@/components/ui/AppBadge.vue'
import ConnectionKindIcon from '@/features/connection/components/ConnectionKindIcon.vue'
import type { ProjectConnection } from '@/features/connection/model/connection'

/** One connection the project uses: its target edited in place, and a way to stop using it. */
const props = defineProps<{ binding: ProjectConnection }>()
const emit = defineEmits<{ target: [target: string | null]; unbind: [] }>()

const draft = ref(props.binding.target ?? '')

watch(
  () => props.binding.target,
  (target) => {
    draft.value = target ?? ''
  }
)

function commit(): void {
  const target = draft.value.trim() || null
  if (target !== props.binding.target) emit('target', target)
}
</script>

<template>
  <li class="flex flex-col gap-1.5 py-2" data-testid="project-connection">
    <div class="flex items-center gap-2">
      <ConnectionKindIcon :kind="binding.kind" :size="14" />
      <span class="min-w-0 flex-1 truncate text-small text-text">{{ binding.title }}</span>
      <AppBadge v-if="!binding.credentialSetAt" tone="warn" dot data-testid="project-connection-missing"
        >No credential</AppBadge
      >
      <button
        class="focus-ring grid size-6 place-items-center rounded-key text-text-subtle hover:bg-surface-hover hover:text-danger"
        :aria-label="`Stop using ${binding.title}`"
        data-testid="unbind-connection"
        @click="emit('unbind')"
      >
        <svg class="size-3" viewBox="0 0 14 14" fill="none" aria-hidden="true">
          <path d="m3.5 3.5 7 7m0-7-7 7" stroke="currentColor" stroke-width="1.6" stroke-linecap="round" />
        </svg>
      </button>
    </div>
    <div class="flex items-center gap-2">
      <span class="shrink-0 font-mono text-caption text-anchor/80">{{ binding.label }}</span>
      <input
        v-model="draft"
        spellcheck="false"
        autocomplete="off"
        :aria-label="
          binding.kind === 'GITLAB'
            ? `GitLab project path for ${binding.title}`
            : `Namespace for ${binding.title}`
        "
        data-testid="connection-target"
        class="field text-text h-7 min-w-0 flex-1 rounded-control px-2 font-mono text-caption"
        :placeholder="binding.kind === 'GITLAB' ? 'group/project' : 'namespace'"
        @blur="commit"
        @keydown.enter.prevent="commit"
      />
    </div>
  </li>
</template>
