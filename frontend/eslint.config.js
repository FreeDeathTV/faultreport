import js from "@eslint/js";
import globals from "globals";
import tseslint from "typescript-eslint";
import reactPlugin from "eslint-plugin-react";
import reactHooksPlugin from "eslint-plugin-react-hooks";

/** @type {import('eslint').Linter.Config[]} */
export default tseslint.config(
  { ignores: ["dist", "node_modules", "build", "coverage"] },
  {
    extends: [
      js.configs.recommended,
      ...tseslint.configs.recommended,
    ],
    files: ["**/*.{ts,tsx}"],
    languageOptions: {
      ecmaVersion: "latest",
      globals: {
        ...globals.browser,
        ...globals.es2021,
      },
    },
    plugins: {
      "react": reactPlugin,
      // Note: react-hooks plugin usually requires the 'default' property 
      // depending on the module resolution setup
      "react-hooks": reactHooksPlugin.configs ? reactHooksPlugin : reactHooksPlugin.default,
    },
    rules: {
      ...(reactHooksPlugin.configs?.recommended.rules || reactHooksPlugin.default?.configs.recommended.rules),
      
      // Rule 9.4: No Production Console Logging (SECURITY)
      // Strictly enforced to prevent accidental exposure of PII or credentials in the browser console.
      "no-console": ["error", { allow: ["debug"] }],
      "no-debugger": "error",

      // General Security & Logic Best Practices
      "no-eval": "error",
      "no-implied-eval": "error",
      "no-new-func": "error",
      "no-script-url": "error",
      "no-caller": "error",
      "no-extend-native": "error",
      
      // React Security (XSS Prevention)
      "react/no-danger": "error",
      "react/jsx-no-target-blank": "error",

      // Type Safety (Rule 9.2)
      "@typescript-eslint/no-explicit-any": "error",
    },
    settings: {
      react: {
        version: "detect",
      },
    },
  }
);