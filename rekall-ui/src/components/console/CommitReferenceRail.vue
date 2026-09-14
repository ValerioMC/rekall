<script setup lang="ts">
import { computed, ref } from 'vue'
import CommitReferenceList from '@/components/console/CommitReferenceList.vue'
import type { CommitReference } from '@/model/commitReference'

const props = defineProps<{
  references: readonly CommitReference[]
}>()

const expanded = ref(false)

const latest = computed(() => props.references[0] ?? null)
const railNodes = computed(() => props.references.slice(0, 6))
const overflowCount = computed(() => props.references.length - railNodes.value.length)
</script>

<template>
  <div class="min-w-0" data-testid="commit-rail">
    <button
      type="button"
      class="focus-ring group/rail flex w-full min-w-0 items-center gap-2.5 rounded-[var(--radius-control)] py-1 text-left"
      :aria-expanded="expanded"
      data-testid="commit-rail-toggle"
      @click="expanded = !expanded"
    >
      <span class="flex shrink-0 items-center" aria-hidden="true">
        <span
          v-for="(reference, index) in railNodes"
          :key="reference.id"
          class="size-2.5 shrink-0 rounded-full border border-border-strong bg-canvas transition-colors group-hover/rail:border-accent"
          :style="index > 0 ? { marginLeft: '-5px' } : undefined"
        />
        <span
          v-if="overflowCount > 0"
          class="ml-1.5 shrink-0 rounded-full border border-border-strong bg-surface px-1.5 py-px font-mono text-[10px] text-text-subtle"
        >
          +{{ overflowCount }}
        </span>
      </span>

      <span class="min-w-0 flex-1 truncate text-[11.5px] text-text-muted" data-testid="commit-rail-latest">
        <span class="font-mono text-accent">{{ latest?.commitHash.slice(0, 7) }}</span>
        <span class="mx-1 text-text-subtle">&#8226;</span>{{ latest?.comment }}
      </span>

      <span class="shrink-0 text-[10.5px] text-text-subtle">
        {{ references.length }} commit{{ references.length === 1 ? '' : 's' }}
      </span>

      <svg
        class="size-3 shrink-0 text-text-subtle transition-transform duration-150"
        :class="expanded && 'rotate-90'"
        viewBox="0 0 12 12"
        fill="none"
        aria-hidden="true"
      >
        <path
          d="m4.2 2.4 3.6 3.6-3.6 3.6"
          stroke="currentColor"
          stroke-width="1.3"
          stroke-linecap="round"
          stroke-linejoin="round"
        />
      </svg>
    </button>

    <div
      class="grid transition-[grid-template-rows] duration-200 ease-out"
      :class="expanded ? 'grid-rows-[1fr]' : 'grid-rows-[0fr]'"
    >
      <div class="overflow-hidden" data-testid="commit-rail-panel">
        <div class="max-h-[260px] overflow-y-auto pb-1 pt-0.5">
          <CommitReferenceList :references="references" show-step-tag />
        </div>
      </div>
    </div>
  </div>
</template>
