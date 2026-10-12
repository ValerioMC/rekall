<script setup lang="ts">
import MemoryGroupSection from './MemoryGroupSection.vue'
import type { MemoryNode } from '@/model/memory-tree'

/** Projects, each with the folders inside it nested below. */
defineProps<{
  nodes: readonly MemoryNode[]
  expanded: (id: string) => boolean
  selectedPath: string | null
}>()

defineEmits<{ toggle: [id: string]; select: [path: string] }>()
</script>

<template>
  <MemoryGroupSection
    v-for="node in nodes"
    :key="node.group.id"
    :node="node"
    :expanded="expanded(node.group.id)"
    :selected-path="selectedPath"
    @toggle="$emit('toggle', node.group.id)"
    @select="$emit('select', $event)"
  >
    <MemoryTree
      v-if="node.children.length > 0"
      :nodes="node.children"
      :expanded="expanded"
      :selected-path="selectedPath"
      @toggle="$emit('toggle', $event)"
      @select="$emit('select', $event)"
    />
  </MemoryGroupSection>
</template>
