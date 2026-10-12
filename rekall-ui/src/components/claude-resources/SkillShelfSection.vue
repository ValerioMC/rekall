<script setup lang="ts">
import { computed } from 'vue'
import ResourceRow from './ResourceRow.vue'
import SkillBucketHeader from './SkillBucketHeader.vue'
import SkillGroupMenu from './SkillGroupMenu.vue'
import type { SkillBucket, SkillShelf } from '@/model/skill-shelves'
import type { ResourceSummary } from '@/model/claude-resources'

/** Skills from one provider. A locked shelf is read only and says who provides it. */
const props = defineProps<{
  shelf: SkillShelf
  expanded: boolean
  isBucketOpen: (bucketId: string) => boolean
  groups: readonly string[]
  selectedPath: string | null
}>()

const emit = defineEmits<{
  toggle: []
  'toggle-bucket': [bucketId: string]
  select: [path: string]
  move: [path: string, group: string | null]
  rename: [from: string, to: string]
}>()

const bodyId = computed(() => `skill-shelf-${props.shelf.id.replace(/[^a-z0-9]+/gi, '-')}`)

function currentGroup(bucket: SkillBucket): string | null {
  return bucket.label
}

function select(skill: ResourceSummary): void {
  emit('select', skill.path)
}
</script>

<template>
  <section class="mb-2" :aria-label="shelf.title" :data-testid="`skill-shelf-${shelf.locked ? 'locked' : 'own'}`">
    <button
      type="button"
      class="focus-ring flex w-full cursor-pointer items-center gap-2 rounded-control px-2 py-1.5 text-left transition-colors hover:bg-surface-raised"
      :aria-expanded="expanded"
      :aria-controls="bodyId"
      @click="emit('toggle')"
    >
      <svg class="size-3 shrink-0 text-text-subtle transition-transform motion-reduce:transition-none" :class="expanded ? 'rotate-90' : ''" viewBox="0 0 12 12" fill="none" aria-hidden="true">
        <path d="m4.5 2.5 3.5 3.5-3.5 3.5" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round" />
      </svg>
      <svg v-if="shelf.locked" class="size-3.5 shrink-0 text-filed" viewBox="0 0 16 16" fill="none" role="img" aria-label="Read only">
        <title>Read only</title>
        <rect x="3.5" y="7" width="9" height="6.5" rx="1.5" stroke="currentColor" stroke-width="1.3" />
        <path d="M5.5 7V5a2.5 2.5 0 0 1 5 0v2" stroke="currentColor" stroke-width="1.3" />
      </svg>
      <svg v-else class="size-3.5 shrink-0 text-accent" viewBox="0 0 16 16" fill="none" aria-hidden="true">
        <path d="M3 13l1-3.5L11 2.5l2.5 2.5L6.5 12z" stroke="currentColor" stroke-width="1.3" stroke-linejoin="round" />
      </svg>
      <span class="min-w-0 flex-1">
        <span class="block truncate text-body font-semibold leading-[1.3] text-text">{{ shelf.title }}</span>
        <span v-if="shelf.caption" class="block truncate text-micro leading-[1.4] text-text-subtle">{{ shelf.caption }}</span>
      </span>
      <span class="shrink-0 font-mono text-micro tabular-nums text-text-subtle">{{ shelf.count }}</span>
    </button>

    <div v-show="expanded" :id="bodyId" class="ml-3 border-l border-border pl-1.5">
      <template v-for="bucket in shelf.buckets" :key="bucket.id">
        <template v-if="bucket.label !== null">
          <SkillBucketHeader
            :label="bucket.label"
            :count="bucket.skills.length"
            :expanded="isBucketOpen(bucket.id)"
            :body-id="`${bodyId}-${bucket.id.replace(/[^a-z0-9]+/gi, '-')}`"
            @toggle="emit('toggle-bucket', bucket.id)"
            @rename="emit('rename', bucket.label!, $event)"
          />
          <div v-show="isBucketOpen(bucket.id)" :id="`${bodyId}-${bucket.id.replace(/[^a-z0-9]+/gi, '-')}`" class="ml-3 border-l border-border pl-1.5">
            <ResourceRow v-for="skill in bucket.skills" :key="skill.path" :summary="skill" :selected="selectedPath === skill.path" @select="select(skill)">
              <template #actions>
                <SkillGroupMenu :skill-name="skill.name" :current="currentGroup(bucket)" :groups="groups" @move="emit('move', skill.path, $event)" />
              </template>
            </ResourceRow>
          </div>
        </template>
        <template v-else>
          <p v-if="shelf.buckets.length > 1" class="px-2 pb-0.5 pt-1.5 text-micro font-medium uppercase tracking-wider text-text-subtle">Not in a group</p>
          <ResourceRow v-for="skill in bucket.skills" :key="skill.path" :summary="skill" :selected="selectedPath === skill.path" @select="select(skill)">
            <template v-if="!shelf.locked" #actions>
              <SkillGroupMenu :skill-name="skill.name" :current="null" :groups="groups" @move="emit('move', skill.path, $event)" />
            </template>
          </ResourceRow>
        </template>
      </template>
    </div>
  </section>
</template>
