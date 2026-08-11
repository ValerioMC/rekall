<script setup lang="ts">
import { computed } from 'vue'
import AppBadge from '@/components/ui/AppBadge.vue'
import AppCard from '@/components/ui/AppCard.vue'
import type { Entity, EntityStatus, Relation } from '@/model/schema'

const props = defineProps<{ entity: Entity; relations: readonly Relation[] }>()

const TONES = { APPLIED: 'safe', MODIFIED: 'warn', DRAFT: 'accent' } as const

const tone = computed(() => TONES[props.entity.status satisfies EntityStatus])

const visibleFields = computed(() => props.entity.fields.slice(0, 6))
const hiddenFieldCount = computed(() => Math.max(0, props.entity.fields.length - 6))
</script>

<template>
  <RouterLink
    :to="`/schema/${entity.id}`"
    data-testid="entity-card"
    class="focus-ring block rounded-[var(--radius-card)]"
  >
    <AppCard interactive class="h-full">
      <div class="flex items-start gap-2">
        <h2 class="text-[15px] font-semibold text-text">{{ entity.label }}</h2>
        <AppBadge :tone="tone" dot>{{ entity.status.toLowerCase() }}</AppBadge>
        <div class="flex-1" />
        <code class="text-[12px] text-text-subtle">{{ entity.physicalName }}</code>
      </div>

      <p class="mt-2 line-clamp-2 text-[13px] text-text-muted">{{ entity.description }}</p>

      <div v-if="entity.aliases.length" class="mt-2 text-[11.5px] text-text-subtle">
        also known as {{ entity.aliases.join(', ') }}
      </div>

      <div class="mt-3.5 flex flex-wrap gap-1.5">
        <AppBadge v-for="field in visibleFields" :key="field.id" mono>{{ field.columnName }}</AppBadge>
        <AppBadge v-if="hiddenFieldCount" mono>+{{ hiddenFieldCount }}</AppBadge>
        <AppBadge v-if="!entity.fields.length" tone="warn">no fields yet</AppBadge>
      </div>

      <div v-if="relations.length" class="mt-3 space-y-1 border-t border-border pt-3">
        <p v-for="relation in relations" :key="relation.id" class="text-[12px] text-text-subtle">
          <span class="text-accent">&rarr;</span>
          <code class="ml-1 text-text-muted">{{ relation.targetTableName }}</code>
          <span class="ml-1">
            {{ relation.kind === 'MANY_TO_ONE' ? 'belongs to one' : 'many-to-many' }}
          </span>
        </p>
      </div>
    </AppCard>
  </RouterLink>
</template>
