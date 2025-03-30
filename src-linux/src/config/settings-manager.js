const Store = require('electron-store')
const { safeStorage } = require('electron')
const crypto = require('crypto')

const schema = {
  accounts: {
    type: 'array',
    items: {
      type: 'object',
      properties: {
        id: { type: 'string' },
        encryptedData: { type: 'string' },
        lastLogin: { type: 'number' }
      }
    }
  },
  blockRecall: { type: 'boolean', default: true },
  multiInstance: { type: 'boolean', default: false }
}

class SettingsManager {
  constructor() {
    this.store = new Store({ schema, encryptionKey: this.getEncryptionKey() })
  }

  getEncryptionKey() {
    let key = safeStorage.getEncryptionKey()
    if (!key) {
      key = crypto.randomBytes(32).toString('hex')
      safeStorage.setEncryptionKey(key)
    }
    return key
  }

  saveAccount(account) {
    const encrypted = safeStorage.encryptString(JSON.stringify(account))
    this.store.set('accounts', [
      ...this.store.get('accounts', []),
      {
        id: account.id,
        encryptedData: encrypted.toString('hex'),
        lastLogin: Date.now()
      }
    ])
  }

  getDecryptedAccounts() {
    return this.store.get('accounts', []).map(acc => {
      try {
        const buf = Buffer.from(acc.encryptedData, 'hex')
        return JSON.parse(safeStorage.decryptString(buf))
      } catch (e) {
        return null
      }
    }).filter(Boolean)
  }
}

module.exports = new SettingsManager()