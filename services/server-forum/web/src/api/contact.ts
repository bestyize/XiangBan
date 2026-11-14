import { request } from './forum'

export interface ContactSettings {
  wechatOfficialName: string
  wechatOfficialQr: boolean
  qqGroupName: string
  qqGroupNumber: string
  qqGroupInviteUrl: string
  qqGroupQr: boolean
  wechatGroupName: string
  wechatGroupQr: boolean
  wechatGroupExpiresOn: string | null
}

export const contactChannels = [
  { key: 'wechat-official', title: '微信公众号', name: 'wechatOfficialName', qr: 'wechatOfficialQr', hint: '用微信扫一扫关注' },
  { key: 'qq-group', title: 'QQ 群', name: 'qqGroupName', qr: 'qqGroupQr', hint: '用 QQ 扫一扫加群' },
  { key: 'wechat-group', title: '微信群', name: 'wechatGroupName', qr: 'wechatGroupQr', hint: '用微信扫一扫入群' },
] as const

export type ContactChannel = (typeof contactChannels)[number]['key']

export const contactQrUrl = (channel: ContactChannel) => `/api/contact/qr/${channel}`
export const getContact = () => request<ContactSettings>('GET', '/contact', undefined, false)
