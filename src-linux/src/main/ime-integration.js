const { IBus, Engine } = require('node-gtk')

class WeChatIME {
  constructor() {
    this.ibus = new IBus()
    this.engine = null
  }

  init() {
    this.ibus.init()
    this.engine = new Engine({
      name: 'betterwx-ime',
      icon: 'assets/ime-icon.png',
      language: 'zh_CN'
    })

    this.engine.on('process-key-event', (keysym, state) => {
      this.handleKeyEvent(keysym, state)
    })

    this.ibus.registerEngine(this.engine)
  }

  handleKeyEvent(keysym, state) {
    // 与Electron输入事件桥接
    require('electron').ipcMain.emit('ime-input-event', null, {
      keysym,
      state
    })
  }
}

module.exports = new WeChatIME()