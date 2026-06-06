import js from '@eslint/js';
import jsxA11y from 'eslint-plugin-jsx-a11y';
import reactHooks from 'eslint-plugin-react-hooks';
import { globalIgnores } from 'eslint/config';
import globals from 'globals';
import tseslint from 'typescript-eslint';

/**
 * ESLint flat config (ESLint 9) for the Luminos control panel.
 *
 * Layers: JS recommended → typescript-eslint recommended → React Hooks
 * recommended → jsx-a11y recommended. The jsx-a11y layer is a hard gate
 * (story 006 FR-9 / AC-3.1): accessibility lint must stay clean.
 */
export default tseslint.config(
  globalIgnores(['dist', 'coverage', 'src/ipc/bindings.ts']),
  {
    files: ['**/*.{ts,tsx}'],
    extends: [
      js.configs.recommended,
      tseslint.configs.recommended,
      jsxA11y.flatConfigs.recommended,
    ],
    plugins: {
      'react-hooks': reactHooks,
    },
    languageOptions: {
      ecmaVersion: 2022,
      globals: { ...globals.browser },
      parserOptions: {
        ecmaFeatures: { jsx: true },
      },
    },
    rules: {
      ...reactHooks.configs['recommended-latest'].rules,
      '@typescript-eslint/no-explicit-any': 'error',
      '@typescript-eslint/consistent-type-imports': [
        'error',
        { prefer: 'type-imports', fixStyle: 'inline-type-imports' },
      ],
    },
  },
  {
    files: ['**/*.{test,spec}.{ts,tsx}', 'src/test/**/*.{ts,tsx}'],
    languageOptions: {
      globals: { ...globals.node },
    },
  },
  {
    files: ['*.config.{ts,js}'],
    languageOptions: {
      globals: { ...globals.node },
    },
  }
);
