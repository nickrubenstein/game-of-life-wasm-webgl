const CopyWebpackPlugin = require("copy-webpack-plugin");
const path = require('path');

module.exports = env => {
    return {
        entry: "./bootstrap.js",
        output: {
            path: path.resolve(__dirname, "dist"),
            filename: "bootstrap.js",
        },
        experiments: {
            asyncWebAssembly: true
        },
        mode: env.mode || "development",
        plugins: [
            new CopyWebpackPlugin({
                patterns: [
                    'index.html',
                    'favicon.ico'
                ]
            })
        ],
    }
};