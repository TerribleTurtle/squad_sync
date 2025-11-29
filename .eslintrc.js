module.exports = {
    root: true,
    extends: ["eslint:recommended", "plugin:@typescript-eslint/recommended", "prettier"],
    parser: "@typescript-eslint/parser",
    plugins: ["@typescript-eslint"],
    env: {
        node: true,
        es2022: true,
    },
    rules: {
        "@typescript-eslint/no-unused-vars": ["warn", { argsIgnorePattern: "^_" }],
        "no-console": ["warn", { allow: ["warn", "error", "info"] }],
    },
    ignorePatterns: ["node_modules", "dist", ".turbo", "coverage"],
};
