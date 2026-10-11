import { watchEffect } from 'vue'

const SITE = '超级小伴'

/** 设置页面标题；空标题用站点首页标题。 */
export function useHead(get: () => { title: string }) {
  watchEffect(() => {
    const { title } = get()
    document.title = title ? `${title} - ${SITE}` : `${SITE} - 开发者和 AI 用户的社区`
  })
}
