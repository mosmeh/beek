module.exports = {
    env: {
        browser: true,
        es2020: true,
        jquery: true,
    },
    extends: 'google',
    parserOptions: {
        ecmaVersion: 11,
        sourceType: 'module',
    },
    rules: {
        'indent': ['error', 4],
        'require-jsdoc': 'off',
    },
};
