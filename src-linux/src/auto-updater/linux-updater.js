const { autoUpdater } = require('electron-updater')
const { AppImageUpdater } = require('electron-updater/lib/AppImageUpdater')
const { dialog } = require('electron')

class LinuxUpdater extends AppImageUpdater {
  constructor() {
    super({
      provider: 'github',
      repo: 'betterwx-linux',
      owner: 'your-org'
    })
    
    this.initListeners()
  }

  initListeners() {
    this.on('update-available', (info) => {
      dialog.showMessageBox({
        type: 'info',
        buttons: ['稍后', '立即更新'],
        message: `发现新版本 ${info.version}`,
        detail: '是否现在安装更新？'
      }).then(({ response }) => {
        if (response === 1) this.downloadUpdate()
      })
    })
  }

  checkForUpdates() {
    this.setFeedURL(this.getUpdateURL())
    super.checkForUpdates()
  }

  getUpdateURL() {
    return `https://api.github.com/repos/your-org/betterwx-linux/releases/latest`
  }
}

module.exports = new LinuxUpdater()