<script setup lang="ts">
import { ref } from 'vue'
import { useRouter } from 'vue-router'
import AnchorBar from '@/components/console/AnchorBar.vue'
import TagsPanel from '@/features/tags/components/TagsPanel.vue'

/** The header content every working screen shares: the `/rk` search, the status capsule, and the Tags panel its key opens. */
defineProps<{ anchor?: string }>()

const router = useRouter()
const tagsOpen = ref(false)
</script>

<template>
  <AnchorBar :anchor="anchor" @open-settings="router.push('/settings')" @open-tags="tagsOpen = true" />
  <Transition name="dialog">
    <TagsPanel v-if="tagsOpen" @close="tagsOpen = false" />
  </Transition>
</template>
