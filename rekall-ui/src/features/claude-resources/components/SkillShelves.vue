<script setup lang="ts">
import { computed, ref } from 'vue'
import SkillShelfSection from './SkillShelfSection.vue'
import { groupLabels, type SkillShelf } from '@/features/skills/model/skill-shelves'
import type { SkillPlacement } from '@/features/claude-resources/model/claude-resources'

/** The Skills tab: the person's own skills in groups they made, then read-only skills by source. */
const props = defineProps<{
  shelves: readonly SkillShelf[]
  placements: readonly SkillPlacement[]
  selectedPath: string | null
  searching: boolean
}>()

const emit = defineEmits<{
  select: [path: string]
  move: [path: string, group: string | null]
  rename: [from: string, to: string]
}>()

/** Shelves of read-only skills start folded, the person's own start open; a toggle flips either. */
const flippedShelves = ref<ReadonlySet<string>>(new Set())
const foldedBuckets = ref<ReadonlySet<string>>(new Set())

const groups = computed(() => groupLabels(props.placements))

function holdsSelection(shelf: SkillShelf): boolean {
  return shelf.buckets.some((bucket) => bucket.skills.some((skill) => skill.path === props.selectedPath))
}

function isShelfOpen(shelf: SkillShelf): boolean {
  if (props.searching || holdsSelection(shelf)) return true
  return flippedShelves.value.has(shelf.id) === shelf.locked
}

function isBucketOpen(bucketId: string): boolean {
  if (props.searching) return true
  const holdsOpenFile = props.shelves.some((shelf) => shelf.buckets.some((bucket) => bucket.id === bucketId && bucket.skills.some((skill) => skill.path === props.selectedPath)))
  return holdsOpenFile || !foldedBuckets.value.has(bucketId)
}

function flip(set: ReadonlySet<string>, id: string): ReadonlySet<string> {
  const next = new Set(set)
  if (!next.delete(id)) next.add(id)
  return next
}
</script>

<template>
  <SkillShelfSection
    v-for="shelf in shelves"
    :key="shelf.id"
    :shelf="shelf"
    :expanded="isShelfOpen(shelf)"
    :is-bucket-open="isBucketOpen"
    :groups="groups"
    :selected-path="selectedPath"
    @toggle="flippedShelves = flip(flippedShelves, shelf.id)"
    @toggle-bucket="foldedBuckets = flip(foldedBuckets, $event)"
    @select="emit('select', $event)"
    @move="(path, group) => emit('move', path, group)"
    @rename="(from, to) => emit('rename', from, to)"
  />
</template>
