<script setup lang="ts">
import { computed } from 'vue'
import ElementCard from '@/features/diagram/components/canvas/ElementCard.vue'
import CanvasNote from '@/features/diagram/components/canvas/CanvasNote.vue'
import SequenceFragment from './SequenceFragment.vue'
import SequenceMessage from './SequenceMessage.vue'
import { roleStyle } from '@/features/diagram/lib/element-style'
import { edgeEmphasis, nodeEmphasis, type Emphasis, type Spotlight } from '@/features/diagram/lib/emphasis'
import { wrapText } from '@/features/diagram/lib/text-wrap'
import { PARTICIPANT_HEIGHT, type SequenceLayout } from '@/features/diagram/lib/layout/sequence-layout'
import { messageKey, messagesOf, type SequenceNote, type SequenceScene } from '@/features/diagram/lib/scene/sequence-scene'
import type { CodeLinks } from '@/features/diagram/model/diagram'

/**
 * A sequence sheet as Mermaid lays one out: participants across the top and again at the foot,
 * a lifeline down from each, the messages in order, with activations, blocks and notes. The
 * participants are the same cards every other sheet draws its elements with.
 */
const props = defineProps<{ scene: SequenceScene; layout: SequenceLayout; links: CodeLinks; spotlight: Spotlight | null }>()

const emit = defineEmits<{ select: [key: string] }>()

const participants = computed(() =>
  props.scene.participants.flatMap((participant) => {
    const placed = props.layout.participants.get(participant.id)
    if (!placed) return []
    const look = roleStyle(participant.isActor ? 'actor' : 'participant')
    return [{ participant, placed, look, lines: wrapText(participant.label, placed.width - 76, 12.5, 1) }]
  })
)

const messages = computed(() =>
  messagesOf(props.scene).flatMap((message) => {
    const placed = props.layout.messages.get(message.number)
    return placed ? [{ message, placed }] : []
  })
)

const notes = computed(() => {
  const byIndex = new Map(props.scene.steps.map((step, index) => [index, step]))
  return props.layout.notes.map((placed) => ({ placed, note: byIndex.get(placed.index) as SequenceNote }))
})

function linked(key: string): boolean {
  return (props.links[key]?.length ?? 0) > 0
}

function messageEmphasis(number: number): Emphasis {
  const key = messageKey(number)
  return props.spotlight?.selected === key ? 'selected' : edgeEmphasis(props.spotlight, key)
}

/** A note dims with the participants it is about. */
function noteEmphasis(note: SequenceNote): Emphasis {
  if (!props.spotlight) return 'rest'
  return note.participantIds.some((id) => props.spotlight?.nodes.has(id)) ? 'rest' : 'dimmed'
}
</script>

<template>
  <g>
    <g v-for="(box, index) in layout.boxes" :key="`box-${index}`" :transform="`translate(${box.x} ${box.y})`" class="sequence-box" aria-hidden="true">
      <rect :width="box.width" :height="box.height" rx="14" />
      <text v-if="box.label" x="14" y="17">{{ box.label }}</text>
    </g>

    <SequenceFragment v-for="(fragment, index) in layout.fragments" :key="`fragment-${index}`" :placed="fragment" layer="frame" />

    <g v-for="{ participant, placed } in participants" :key="`life-${participant.id}`" :class="`is-${nodeEmphasis(spotlight, participant.id)}`" class="sequence-lifeline" aria-hidden="true">
      <path :d="`M${placed.centre},${layout.lifelineTop}V${layout.lifelineBottom}`" />
    </g>

    <rect
      v-for="(bar, index) in layout.activations"
      :key="`bar-${index}`"
      :x="bar.x"
      :y="bar.y"
      :width="bar.width"
      :height="bar.height"
      rx="2"
      class="sequence-activation"
      :class="`is-${nodeEmphasis(spotlight, bar.participantId)}`"
      aria-hidden="true"
    />

    <SequenceFragment v-for="(fragment, index) in layout.fragments" :key="`words-${index}`" :placed="fragment" layer="words" />

    <CanvasNote v-for="{ placed, note } in notes" :key="`note-${placed.index}`" :box="placed" :lines="placed.lines" :emphasis="noteEmphasis(note)" />

    <SequenceMessage
      v-for="{ message, placed } in messages"
      :key="`message-${message.number}`"
      :message="message"
      :placed="placed"
      :numbered="scene.autonumber"
      :linked="linked(messageKey(message.number))"
      :emphasis="messageEmphasis(message.number)"
      @select="emit('select', $event)"
    />

    <template v-for="{ participant, placed, look, lines } in participants" :key="participant.id">
      <ElementCard
        :element-key="participant.id"
        :x="placed.x"
        :y="placed.y"
        :width="placed.width"
        :height="PARTICIPANT_HEIGHT"
        outline="rounded"
        :look="look"
        :lines="lines"
        :linked="linked(participant.id)"
        :emphasis="nodeEmphasis(spotlight, participant.id)"
        @select="emit('select', $event)"
      />
      <g :transform="`translate(${placed.x} ${placed.footY})`" class="sequence-foot" :class="`is-${nodeEmphasis(spotlight, participant.id)}`" :style="{ '--tint': look.tint }" aria-hidden="true">
        <rect :width="placed.width" height="30" rx="8" />
        <text :x="placed.width / 2" y="19.5" text-anchor="middle">{{ lines[0] }}</text>
      </g>
    </template>
  </g>
</template>

<style scoped>
.sequence-box rect {
  fill: color-mix(in srgb, var(--color-kind-external) 4%, transparent);
  stroke: color-mix(in srgb, var(--color-kind-external) 22%, transparent);
}

.sequence-box text {
  font-family: var(--font-sans);
  font-size: var(--text-caption);
  font-weight: 600;
  fill: var(--color-text-muted);
}

.sequence-lifeline path {
  stroke: var(--color-border-strong);
  stroke-width: 1;
  stroke-dasharray: 3 4;
}

.sequence-lifeline.is-selected path,
.sequence-lifeline.is-lit path {
  stroke: color-mix(in srgb, var(--color-accent) 55%, transparent);
}

.sequence-activation {
  fill: color-mix(in srgb, var(--color-kind-action) 18%, var(--color-surface-raised));
  stroke: color-mix(in srgb, var(--color-kind-action) 45%, transparent);
  stroke-width: 1;
}

.sequence-foot rect {
  fill: color-mix(in srgb, var(--tint) 5%, var(--color-surface));
  stroke: color-mix(in srgb, var(--tint) 28%, transparent);
}

.sequence-foot text {
  font-family: var(--font-sans);
  font-size: var(--text-caption);
  fill: var(--color-text-muted);
}

.is-dimmed {
  opacity: 0.22;
}
</style>
