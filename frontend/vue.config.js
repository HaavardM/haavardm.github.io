const BootstrapVueLoader = require('bootstrap-vue-loader')
module.exports = {
  publicPath: process.env.NODE_ENV === "development" ? "/" : "/playground/",
  configureWebpack: {
    plugins: [new BootstrapVueLoader()],
    optimization: {
      splitChunks: {
        chunks: "all",
      }
    }
  }
};