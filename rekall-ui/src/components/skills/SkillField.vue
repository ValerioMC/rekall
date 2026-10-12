<script setup lang="ts">
import { computed, ref } from 'vue'
import SkillChip from '@/components/skills/SkillChip.vue'
import SkillGlyph from '@/components/skills/SkillGlyph.vue'
import SkillPicker from '@/components/skills/SkillPicker.vue'
import { useSkillCatalog } from '@/composables/useSkillCatalog'
import type { EffectiveSkill } from '@/model/skill-assignment'

/**
 * The skills of one owner as chips, with the button that opens the catalogue to change them.
 * `inherited` are the skills a wider scope already brings: drawn first, dashed, not removable.
 * The owner's own list is the model, and every change is sent up as the whole list.
 */
const props = withDefaults(
  defineProps<{
    inherited?: readonly EffectiveSkill[]
    /** What the picker is called, and what the add button names: "task", "step" or "tag". */
    owner: string
    ownOrigin?: 'task' | 'step'
  }>(),
  { inherited: () => [], ownOrigin: 'task' }
)

const skills = defineModel<readonly string[]>({ required: true })

const { names: known, loaded } = useSkillCatalog()
const open = ref(false)
const addButton = ref<HTMLElement | null>(null)

const inheritedNames = computed(() => props.inherited.map((skill) => skill.name))
const isMissing = (name: string): boolean => loaded.value && !known.value.has(name)

function remove(name: string): void {
  skills.value = skills.value.filter((candidate) => candidate !== name)
}
</script>

<template>
  <div class="flex min-w-0 flex-wrap items-center gap-1.5" data-testid="skill-field">
    <SkillChip
      v-for="skill in inherited"
      :key="`inherited:${skill.name}`"
      :name="skill.name"
      :origin="skill.origin"
      :missing="isMissing(skill.name)"
      :removable="false"
    />
    <SkillChip
      v-for="name in skills"
      :key="name"
      :name="name"
      :origin="ownOrigin"
      :missing="isMissing(name)"
      @remove="remove(name)"
    />
    <button
      ref="addButton"
      type="button"
      class="focus-ring inline-flex h-6 items-center gap-1 rounded-key border border-dashed border-border-strong px-2 text-caption text-text-muted transition-colors hover:border-anchor hover:text-anchor"
      :aria-expanded="open"
      aria-haspopup="dialog"
      :aria-label="`Add a skill to this ${owner}`"
      data-testid="skill-add"
      @click="open = !open"
    >
      <SkillGlyph :size="11" />
      {{ skills.length + inherited.length ? 'Add' : 'Add a skill' }}
    </button>
    <SkillPicker
      v-if="open && addButton"
      :model-value="skills"
      :anchor="addButton"
      :inherited="inheritedNames"
      :title="`Skills for this ${owner}`"
      @update:model-value="skills = $event"
      @close="open = false"
    />
  </div>
</template>
