const { Notification } = require('electron')
const dbus = require('dbus-next')
const { LinuxTray } = require('../main/natives')

class NotificationCenter {
  constructor() {
    this.bus = dbus.sessionBus()
    this.notificationId = 0
    this.initDBusInterface()
  }

  async initDBusInterface() {
    const iface = await this.bus.getProxyObject(
      'org.freedesktop.Notifications',
      '/org/freedesktop/Notifications'
    )
    
    this.interface = iface.getInterface('org.freedesktop.Notifications')
    this.interface.on('ActionInvoked', this.handleAction.bind(this))
  }

  showNotification(title, body, actions = []) {
    const id = this.notificationId++
    const hints = {
      'urgency': 1,
      'desktop-entry': 'betterwx'
    }

    // 系统级通知
    this.interface.Notify(
      'BetterWx',
      id,
      'betterwx',
      title,
      body,
      actions,
      hints,
      5000
    )

    // Electron通知回退
    new Notification({ title, body }).show()
  }

  handleAction(id, actionKey) {
    LinuxTray.handleNotificationAction(id, actionKey)
  }
}

module.exports = new NotificationCenter()