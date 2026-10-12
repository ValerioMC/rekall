<script setup lang="ts">
import { computed } from 'vue'
import { storeToRefs } from 'pinia'
import SkillField from '@/components/skills/SkillField.vue'
import { useConsoleStore } from '@/stores/console.store'
import { skillsOfTag } from '@/model/skill-assignment'
import type { TagId } from '@/model/branded'

/**
 * The skills a new task starts with. The tag chosen above brings its own, shown dashed so the
 * list reads as what the task will run with; they are the tag's to change, not this form's.
 */
const props = defineProps<{ tagId: TagId | null }>()

const skills = defineModel<readonly string[]>({ required: true })

const { tagSkills } = storeToRefs(useConsoleStore())
const fromTag = computed(() => skillsOfTag(props.tagId, tagSkills.value))
</script>

<template>
  <p class="mb-1.5 mt-5 eyebrow text-caption">Skills</p>
  <div data-testid="record-skills">
    <SkillField v-model="skills" :inherited="fromTag" owner="task" />
  </div>
  <p class="mt-1.5 text-caption text-text-subtle">
    Claude invokes these when you run the task. Every step inherits them.
  </p>
</template>
