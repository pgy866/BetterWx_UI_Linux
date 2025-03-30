const { assert } = require('chai')
const { WeChatManager } = require('../src/main/main')
const { mockIpcMain } = require('electron-ipc-mock')

describe('Main Process', () => {
  let manager

  before(() => {
    manager = new WeChatManager()
    mockIpcMain()
  })

  it('should create isolated instances', async () => {
    await manager.createInstance('test1')
    const instance = manager.instances.get('test1')
    assert.exists(instance.webContents.session)
    assert.notEqual(
      instance.webContents.session,
      manager.instances.get('default').webContents.session
    )
  })

  it('should block recall requests', (done) => {
    const session = manager.instances.get('default').webContents.session
    session.webRequest.onBeforeRequest((details, callback) => {
      if (details.url.includes('revokemsg')) {
        callback({ cancel: true })
        done()
      }
    })
  })
})