<script setup lang="ts">
import AppBadge from '@/components/ui/AppBadge.vue'
import AppButton from '@/components/ui/AppButton.vue'
import AppCard from '@/components/ui/AppCard.vue'
import type { Relation } from '@/model/schema'
import type { RelationId } from '@/model/branded'

defineProps<{ outgoing: readonly Relation[]; incoming: readonly Relation[] }>()

const emit = defineEmits<{ remove: [id: RelationId] }>()
</script>

<template>
  <div class="space-y-2.5">
    <AppCard
      v-for="relation in outgoing"
      :key="relation.id"
      data-testid="relation-outgoing"
      class="flex flex-wrap items-center gap-3 !py-3.5"
    >
      <AppBadge tone="accent">
        {{ relation.kind === 'MANY_TO_ONE' ? 'belongs to one' : 'many-to-many' }}
      </AppBadge>
      <code class="text-text">{{ relation.targetTableName }}</code>
      <AppBadge>on delete {{ relation.onDelete.toLowerCase().replace('_', ' ') }}</AppBadge>
      <span class="text-[12.5px] text-text-subtle">{{ relation.description }}</span>
      <div class="flex-1" />
      <AppButton variant="ghost" size="sm" @click="emit('remove', relation.id)">remove</AppButton>
    </AppCard>

    <!-- The inverse direction is shown but not editable: it is derived, not stored. -->
    <AppCard v-for="relation in incoming" :key="relation.id" class="flex flex-wrap items-center gap-3 !py-3.5">
      <AppBadge>has many</AppBadge>
      <code class="text-text">{{ relation.sourceTableName }}</code>
      <span class="text-[12.5px] text-text-subtle">derived from the other side</span>
    </AppCard>

    <p v-if="!outgoing.length && !incoming.length" class="text-[13px] text-text-subtle">
      This entity stands alone.
    </p>
  </div>
</template>
