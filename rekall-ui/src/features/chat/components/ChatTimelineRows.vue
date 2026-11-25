<script setup lang="ts">
import { computed } from 'vue'
import { countCalls } from '@/features/chat/lib/group-by-parent'
import type { TimelineItem } from '@/features/chat/model/timeline'
import { resolveRow } from './timeline-rows'

/**
 * One `li` per item. An Agent call draws what its subagent did inside its own card, through the
 * same rows.
 */
const props = defineProps<{
  items: readonly TimelineItem[]
  inner: ReadonlyMap<string, readonly TimelineItem[]>
  followingNext: boolean
}>()

function innerOf(item: TimelineItem): readonly TimelineItem[] {
  return item.kind === 'tool' ? (props.inner.get(item.toolUseId) ?? []) : []
}

const rows = computed(() =>
  props.items.map((item) => {
    const inner = innerOf(item)
    const row = resolveRow(item, { followingNext: props.followingNext, innerCalls: countCalls(inner) })
    return { item, inner, row }
  })
)
</script>

<template>
  <li v-for="{ item, inner: innerItems, row } in rows" :key="item.key">
    <component :is="row.component" v-bind="row.props">
      <template v-if="innerItems.length > 0" #inner>
        <ol class="flex flex-col gap-2" data-testid="agent-inner">
          <ChatTimelineRows :items="innerItems" :inner="inner" :following-next="followingNext" />
        </ol>
      </template>
    </component>
  </li>
</template>
