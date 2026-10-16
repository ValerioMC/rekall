<script setup lang="ts">
import { computed } from 'vue'
import ElementCard from '@/features/diagram/components/canvas/ElementCard.vue'
import CanvasNote from '@/features/diagram/components/canvas/CanvasNote.vue'
import BoxRows from './BoxRows.vue'
import GraphEdge from './GraphEdge.vue'
import GraphGroup from './GraphGroup.vue'
import GraphMark from './GraphMark.vue'
import { roleStyle, type ElementStyle } from '@/features/diagram/lib/element-style'
import { edgeEmphasis, nodeEmphasis, type Spotlight } from '@/features/diagram/lib/emphasis'
import { BOX_HEADER } from '@/features/diagram/lib/layout/node-size'
import type { GraphLayout, PlacedNode } from '@/features/diagram/lib/layout/graph-layout'
import type { GraphNode, GraphScene } from '@/features/diagram/lib/scene/graph-scene'
import type { CodeLinks } from '@/features/diagram/model/diagram'

/**
 * A flowchart, state, class or ER sheet: groups first, then the lines, then the elements on top.
 * It draws what it is given and decides nothing about the sheet.
 */
const props = defineProps<{ scene: GraphScene; layout: GraphLayout; links: CodeLinks; spotlight: Spotlight | null }>()

const emit = defineEmits<{ select: [key: string] }>()

type DrawnAs = 'card' | 'box' | 'mark' | 'note'

interface DrawnNode {
  readonly node: GraphNode
  readonly placed: PlacedNode
  readonly as: DrawnAs
  readonly look: ElementStyle
}

const nodes = computed<DrawnNode[]>(() =>
  props.scene.nodes.flatMap((node) => {
    const placed = props.layout.nodes.get(node.id)
    return placed ? [{ node, placed, as: drawnAs(node), look: lookOf(node) }] : []
  })
)

const groups = computed(() =>
  props.scene.groups
    .flatMap((group) => {
      const placed = props.layout.groups.get(group.id)
      return placed ? [{ group, placed }] : []
    })
    .sort((a, b) => a.placed.depth - b.placed.depth)
)

const edges = computed(() => {
  const byKey = new Map(props.scene.edges.map((edge) => [edge.key, edge]))
  return props.layout.edges.flatMap((placed) => {
    const edge = byKey.get(placed.key)
    return edge ? [{ edge, placed }] : []
  })
})

const groupTint = computed(() => (props.scene.kind === 'state' ? roleStyle('state').tint : 'var(--color-kind-external)'))

/** Elements arrive in reading order: a stagger along the flow, capped so a big sheet is not slow. */
const arriveSpan = computed(() => {
  const xs = nodes.value.map((drawn) => drawn.placed.x + drawn.placed.y)
  return { min: Math.min(...xs, 0), width: Math.max(1, Math.max(...xs, 0) - Math.min(...xs, 0)) }
})

function arriveDelay(placed: PlacedNode): number {
  return Math.round(((placed.x + placed.y - arriveSpan.value.min) / arriveSpan.value.width) * 320)
}

function drawnAs(node: GraphNode): DrawnAs {
  if (node.outline === 'note') return 'note'
  if (node.role === 'pseudo' || (node.role === 'decision' && node.label.length === 0)) return 'mark'
  return node.role === 'class' || node.role === 'entity' ? 'box' : 'card'
}

/** A class's stereotype replaces its kind label: «interface» says more than "class". */
function lookOf(node: GraphNode): ElementStyle {
  const look = roleStyle(node.role)
  const stereotype = node.annotations[0]
  return stereotype ? { ...look, label: `«${stereotype}»` } : look
}

function linked(key: string): boolean {
  return (props.links[key]?.length ?? 0) > 0
}
</script>

<template>
  <g>
    <GraphGroup
      v-for="{ group, placed } in groups"
      :key="group.id"
      :placed="placed"
      :label="group.label"
      :tint="groupTint"
      :linked="linked(group.id)"
      :emphasis="nodeEmphasis(spotlight, group.id)"
      @select="emit('select', $event)"
    />
    <GraphEdge v-for="{ edge, placed } in edges" :key="edge.key" :placed="placed" :edge="edge" :emphasis="edgeEmphasis(spotlight, edge.key)" />
    <template v-for="drawn in nodes" :key="drawn.node.id">
      <GraphMark v-if="drawn.as === 'mark'" :placed="drawn.placed" :outline="drawn.node.outline" :tint="drawn.look.tint" :emphasis="nodeEmphasis(spotlight, drawn.node.id)" />
      <CanvasNote v-else-if="drawn.as === 'note'" :box="drawn.placed" :lines="drawn.placed.lines" :emphasis="nodeEmphasis(spotlight, drawn.node.id)" />
      <ElementCard
        v-else
        :element-key="drawn.node.id"
        :x="drawn.placed.x"
        :y="drawn.placed.y"
        :width="drawn.placed.width"
        :height="drawn.placed.height"
        :outline="drawn.node.outline"
        :look="drawn.look"
        :lines="drawn.placed.lines"
        :linked="linked(drawn.node.linkKey)"
        :emphasis="nodeEmphasis(spotlight, drawn.node.id)"
        :arrive-delay="arriveDelay(drawn.placed)"
        :header-height="drawn.as === 'box' ? BOX_HEADER : undefined"
        @select="emit('select', $event)"
      >
        <BoxRows v-if="drawn.as === 'box'" :width="drawn.placed.width" :compartments="drawn.node.compartments" />
      </ElementCard>
    </template>
  </g>
</template>
