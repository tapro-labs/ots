module.exports = {
  extends:   [
    'stylelint-config-standard-scss',
    'stylelint-config-standard-vue/scss'
  ],
  rules:     {
    "font-family-name-quotes": "always-where-recommended",
    "function-url-quotes": "always",
    "selector-attribute-quotes": "always",
    'scss/at-rule-no-unknown': [
      true,
      {
        ignoreAtRules: [
          'tailwind',
          'apply',
          'variants',
          'responsive',
          'screen',
        ],
      },
    ],
    'declaration-block-trailing-semicolon': null,
    'no-descending-specificity': null,
    'no-invalid-position-at-import-rule': null,
  },
  overrides: [
    {
      files: [
        './src/*.vue',
        './src/**/*.vue'
      ],
      rules: {
        'unit-allowed-list': [
          'px',
          'rem',
        ]
      }
    }
  ]
};
