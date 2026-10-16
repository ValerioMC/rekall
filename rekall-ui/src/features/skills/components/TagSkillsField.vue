<script setup lang="ts">
import SkillField from '@/features/skills/components/SkillField.vue'

/** The skills every task under a tag runs with. Changing them changes every one of those tasks. */
const skills = defineModel<readonly string[]>({ required: true })
</script>

<template>
  <p class="mb-1.5 mt-4 eyebrow text-caption">Skills</p>
  <div data-testid="tag-skills">
    <SkillField v-model="skills" owner="tag" />
  </div>
  <p class="mt-1.5 text-caption text-text-subtle">
    Claude invokes these on every task with this tag, steps included.
  </p>
</template>
