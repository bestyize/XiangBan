import { api, setToken } from '../api/client.ts'
import { getDeviceId, resetDeviceId } from './device.ts'

export async function logout(next = '/login') {
  await api('/api/account/logout', { method: 'POST', body: { deviceId: getDeviceId() } }, true)
  setToken(null)
  resetDeviceId()
  location.replace(next)
}
