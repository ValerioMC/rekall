import pluginVue from 'eslint-plugin-vue'
import vueTsConfigs from '@vue/eslint-config-typescript'
import skipFormatting from '@vue/eslint-config-prettier/skip-formatting'

export default [
  { ignores: ['dist/**', 'node_modules/**', 'e2e/**'] },
  ...pluginVue.configs['flat/recommended'],
  ...vueTsConfigs(),
  skipFormatting,
  {
    rules: {
      // Component file names are already unique inside their folder, and prefixing every one
      // of them with "App" to satisfy the multi-word rule adds nothing.
      'vue/multi-word-component-names': 'off',
      '@typescript-eslint/no-explicit-any': 'error',
      '@typescript-eslint/consistent-type-imports': 'error'
    }
  }
]
