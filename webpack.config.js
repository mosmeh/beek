const webpack = require('webpack');
const path = require('path');
const HtmlWebpackPlugin = require('html-webpack-plugin');
const WasmPackPlugin = require("@wasm-tool/wasm-pack-plugin");
const { CleanWebpackPlugin } = require('clean-webpack-plugin');

module.exports = {
    entry: path.resolve(__dirname, 'web', 'index.js'),
    output: {
        path: path.resolve(__dirname, 'public'),
        filename: 'index.js',
    },
    devServer: {
        contentBase: path.resolve(__dirname, 'web'),
        open: true,
    },
    performance: {
        hints: false
    },
    module: {
        rules: [{
            test: /\.js$/,
            exclude: /node_modules/,
            use: {
                loader: 'babel-loader',
                options: {
                    presets: ['@babel/preset-env']
                }
            }
        },
        {
            test: /\.css/,
            use: [
                "style-loader",
                {
                    loader: "css-loader",
                    options: { url: false }
                }
            ]
        }
        ]
    },
    plugins: [
        new HtmlWebpackPlugin({
            template: path.resolve(__dirname, 'web', 'index.html')
        }),
        new WasmPackPlugin({
            crateDirectory: path.resolve(__dirname, '.'),
            outDir: path.resolve(__dirname, 'web', 'pkg')
        }),
        new webpack.ProvidePlugin({
            TextDecoder: ['text-encoding', 'TextDecoder'],
            TextEncoder: ['text-encoding', 'TextEncoder']
        }),
        new CleanWebpackPlugin(),
    ],
};
