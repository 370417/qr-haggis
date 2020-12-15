module.exports = {
    plugins: [
        require('postcss-custom-properties')({
            preserve: false,
        }),
        require('postcss-calc'),
        require('cssnano')({
            preset: 'advanced',
        }),
    ],
};
