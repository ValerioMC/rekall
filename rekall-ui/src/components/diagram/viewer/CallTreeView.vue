<script setup lang="ts">
import { messageKey } from '@/common/diagram/scene/sequence-scene'
import type { CallNode } from '@/common/diagram/call-tree'
import type { CodeLinks } from '@/model/diagram'

/**
 * Calls as a tree: each row is a call, its receiver and what came back, nested under the call it
 * was made from. A row selects its message on the canvas.
 */
defineOptions({ name: 'CallTreeView' })

defineProps<{
  nodes: readonly CallNode[]
  labels: ReadonlyMap<string, string>
  links: CodeLinks
  selected: string | null
  depth?: number
}>()

const emit = defineEmits<{ select: [key: string] }>()
</script>

<template>
  <ul class="flex flex-col" :class="depth ? 'call-children ml-[11px] border-l border-border pl-2' : ''" role="group">
    <li v-for="node in nodes" :key="node.call.number" role="treeitem" :aria-expanded="node.children.length > 0 ? true : undefined">
      <button
        class="focus-ring group flex w-full min-w-0 items-start gap-2 rounded-control px-1.5 py-1 text-left transition-colors hover:bg-surface-raised"
        :class="selected === messageKey(node.call.number) ? 'bg-accent-soft' : ''"
        :aria-current="selected === messageKey(node.call.number) ? 'true' : undefined"
        data-testid="call-row"
        @click="emit('select', messageKey(node.call.number))"
      >
        <span class="mt-px shrink-0 rounded-key bg-canvas px-1 font-mono text-micro tabular-nums text-text-subtle">{{ node.call.number }}</span>
        <span class="min-w-0 flex-1">
          <span class="flex min-w-0 items-baseline gap-1.5">
            <span class="truncate font-mono text-caption text-text">{{ node.call.text || '(no text)' }}</span>
            <span v-if="(links[messageKey(node.call.number)]?.length ?? 0) > 0" class="shrink-0 font-mono text-micro text-kind-action" title="Opens its code">&lt;/&gt;</span>
          </span>
          <span class="block truncate text-micro text-text-subtle">
            → {{ labels.get(node.call.to) ?? node.call.to }}<template v-if="node.reply"> · ↩ {{ node.reply.text || 'returns' }}</template>
          </span>
        </span>
      </button>
      <CallTreeView
        v-if="node.children.length > 0"
        :nodes="node.children"
        :labels="labels"
        :links="links"
        :selected="selected"
        :depth="(depth ?? 0) + 1"
        @select="emit('select', $event)"
      />
    </li>
  </ul>
</template>
