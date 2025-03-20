import globals from "globals";
import google from "eslint-config-google";
import prettier from "eslint-config-prettier";

export default [
  {
    ignores: ["repositories/", "target/"],
  },
  prettier,
  {
    ...google,
    languageOptions: {
      globals: {
        ...globals.commonjs,
        ...globals.es2021,
      },

      ecmaVersion: "latest",
      sourceType: "module",
    },

    rules: {
      indent: [
        "error",
        2,
        {
          SwitchCase: 1,
        },
      ],

      "max-len": [
        "error",
        {
          code: 120,
          ignoreComments: true,
          ignoreUrls: true,
          ignoreStrings: true,
        },
      ],
    },
  },
];
