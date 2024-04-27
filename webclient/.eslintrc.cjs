/* eslint-env node */
module.exports = {
  root: true,
  extends: [
    "plugin:vue/vue3-essential",
    "plugin:vue/vue3-strongly-recommended",
    "plugin:vue/vue3-recommended",
    "eslint:recommended",
    "@vue/eslint-config-typescript/recommended",
    "@vue/eslint-config-prettier",
    "@vue/eslint-config-prettier/skip-formatting",
  ],
  parserOptions: {
    ecmaVersion: "latest",
  },
  rules: {
    "vue/no-v-html": "off",
    "vue/block-lang": [
      "error",
      {
        script: {
          lang: "ts",
        },
      },
    ],
    "vue/block-order": [
      "error",
      {
        order: ["script", "template", "style", "i18n"],
      },
    ],
    "vue/block-tag-newline": "error",
    "vue/component-api-style": [
      "error",
      ["script-setup", "composition"], // "script-setup", "composition", "composition-vue2", or "options"
    ],
    "vue/multi-word-component-names": "off",
    "vue/component-name-in-template-casing": ["error", "PascalCase", { registeredComponentsOnly: false }],
    "vue/custom-event-name-casing": ["error", "camelCase"],
    "vue/define-macros-order": "error",
    "vue/define-props-declaration": ["error", "type-based"],
    "vue/html-button-has-type": [
      "error",
      {
        button: true,
        submit: true,
        reset: true,
      },
    ],
    "vue/no-boolean-default": ["error", "default-false"],
    "vue/no-empty-component-block": "error",
    "vue/html-comment-content-spacing": ["error", "always"],
    "vue/no-ref-object-reactivity-loss": "error",
    "vue/no-required-prop-with-default": "error",
    "vue/no-restricted-call-after-await": "error",
    //"vue/no-root-v-if": "error", todo: enable when there is a loading animation
    "vue/no-setup-props-reactivity-loss": "error",
    //"vue/no-static-inline-styles": "error", todo: enable after migration to tailwind
    "vue/no-useless-mustaches": "error",
    "vue/no-useless-v-bind": "error",
    "vue/no-v-text": "error",
    "vue/padding-line-between-blocks": "error",
    "vue/prefer-prop-type-boolean-first": "error",
    "vue/prefer-separate-static-class": "error",
    "vue/require-macro-variable-name": "error",
    "vue/require-typed-ref": "error",
    "vue/static-class-names-order": "off",
    "vue/v-for-delimiter-style": "error",
    "vue/no-constant-condition": "error",
  },
};
