const path = require('path');
const CopyPlugin = require("copy-webpack-plugin");
const WasmPackPlugin = require("@wasm-tool/wasm-pack-plugin");


module.exports = {
    entry: './src/index.tsx',
    module: {
        rules: [
            {
                test: /\.tsx?$/,
                use: 'ts-loader',
                exclude: /node_modules/,
            },
        ],
    },
    plugins: [
        new CopyPlugin({
            patterns: [
                { from: 'static', to: path.join(__dirname, 'dist') },
                // { from: 'wasm', to: path.join(__dirname, 'dist') },
            ],
        }),
        new WasmPackPlugin({
            crateDirectory: path.resolve(__dirname, "../backend"),
            outDir: "../frontend/dist",
            outName: "qr_haggis",
            withTypescript: true,
        }),
    ],
    devtool: "source-map",
    devServer: {
        contentBase: path.join(__dirname, 'dist'),
        compress: true,
        port: 9000,
        // writeToDisk: true,
    },
    resolve: {
        extensions: ['.tsx', '.ts', '.js'],
    },
    output: {
        filename: 'bundle.js',
        path: path.resolve(__dirname, 'dist'),
    },
};
