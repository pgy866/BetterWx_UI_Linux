const dbus = require('dbus-next')
const { LinuxTray } = require('../main/natives')

class WeChatDaemon {
  constructor() {
    this.bus = dbus.sessionBus()
    this.service = null
    this.initDBusService()
  }

  async initDBusService() {
    const iface = {
      name: 'com.betterwx.Linux',
      methods: {
        CreateInstance: ['s', '', '', (profile) => this.createInstance(profile)]
      },
      signals: {
        InstanceCreated: ['s']
      }
    }

    this.service = await this.bus.exportService(
      '/com/betterwx/Linux',
      iface
    )
  }

  createInstance(profile) {
    const instance = new WeChatManager().createInstance(profile)
    this.service.emit('InstanceCreated', profile)
  }
}

module.exports = new WeChatDaemon()