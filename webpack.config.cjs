const path = require("path");
const autoprefixer = require("autoprefixer");
const CopyPlugin = require("copy-webpack-plugin");
const WasmPackPlugin = require("@wasm-tool/wasm-pack-plugin");
const HtmlWebpackPlugin = require("html-webpack-plugin");
const MiniCssExtractPlugin = require("mini-css-extract-plugin");

const dist = path.resolve(__dirname, "dist");

module.exports = {
  mode: "development",
  experiments: {
    asyncWebAssembly: true,
  },
  entry: {
    index: "./src/js/index.js",
  },
  output: {
    path: dist,
    filename: "[name].js",
  },
  devServer: {
    static: dist,
    port: 8080,
    hot: true,
  },
  plugins: [
    new CopyPlugin({
      patterns: [path.resolve(__dirname, "static")],
    }),
    new WasmPackPlugin({
      crateDirectory: __dirname,
    }),
    new HtmlWebpackPlugin({
      template: "./src/index.html",
    }),
    new MiniCssExtractPlugin(),
  ],
  module: {
    rules: [
      {
        mimetype: "image/svg+xml",
        scheme: "data",
        type: "asset/resource",
        generator: {
          filename: "icons/[hash].svg",
        },
      },
      {
        test: /\.(scss)$/,
        use: [
          { loader: MiniCssExtractPlugin.loader },
          { loader: "css-loader" },
          {
            loader: "postcss-loader",
            options: {
              postcssOptions: {
                plugins: [autoprefixer],
              },
            },
          },
          { loader: "sass-loader" },
        ],
      },
    ],
  },
};
