const { app } = require('electron')
const path = require('path')
const log4js = require('log4js')

const LOG_DIR = path.join(app.getPath('userData'), 'logs')

log4js.configure({
  appenders: {
    console: { type: 'console' },
    file: {
      type: 'file',
      filename: path.join(LOG_DIR, 'betterwx.log'),
      maxLogSize: 10485760,
      backups: 5,
      compress: true
    },
    dbus: {
      type: 'logLevelFilter',
      appender: {
        type: 'src/utils/dbus-appender' // 自定义DBus输出
      },
      level: 'ERROR'
    }
  },
  categories: {
    default: { appenders: ['console', 'file', 'dbus'], level: 'debug' }
  }
})

module.exports = {
  main: log4js.getLogger('main'),
  renderer: log4js.getLogger('renderer'),
  dbus: log4js.getLogger('dbus')
}