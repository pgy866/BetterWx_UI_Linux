// 在preload.js中添加
if (process.platform === 'linux') {
  require('node-gtk').registerGtkImModule()
}