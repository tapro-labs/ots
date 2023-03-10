module.exports = {
  extends:   [
    'stylelint-config-standard-scss',
    'stylelint-config-standard-vue/scss'
  ],
  rules:     {
    'string-quotes': 'single',
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
