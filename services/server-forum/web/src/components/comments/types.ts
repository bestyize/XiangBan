import type { NewsComment } from '@/api/discover'
import type { Page } from '@/api/types'

/** 评论区的数据来源：日报、前沿、开源项目各自对接口，界面和交互共用一套。 */
export interface CommentAdapter {
  list: (page: number) => Promise<Page<NewsComment>>
  post: (body: string, replyTo?: number) => Promise<unknown>
  like: (id: number, on: boolean) => Promise<unknown>
  remove: (id: number) => Promise<unknown>
}
