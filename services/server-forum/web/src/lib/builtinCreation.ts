/** Same order and categories as the mobile GameLibrary; assets are copied at dev/build time. */
export const BUILTIN_GAMES = [
  { id:'dusk-glider', name:'暮色滑翔', genre:'冒险', description:'横屏 3D 滑翔，在日落之前飞过一座座岛。', orientation:'landscape' },
  { id:'ink-blade', name:'水墨剑客', genre:'冒险', description:'横屏水墨动作，出剑弹反，闯过竹林和雪江。', orientation:'landscape' },
  { id:'jade-gomoku', name:'雅韵五子棋', genre:'益智', description:'水墨玉石质感五子棋，支持人机对弈与双人对局。', orientation:'landscape' },
  { id:'pro-cube', name:'3D全阶魔方', genre:'益智', description:'3 至 9 阶魔方，支持手势操作与智能还原。', orientation:'landscape' },
  { id:'snake-arena', name:'小伴贪吃蛇', genre:'解压', description:'无尽成长或三分钟冲分，吃糖果长成大蛇。', orientation:'landscape' },
  { id:'block-puzzle', name:'消失高手', genre:'益智', description:'拖动方块填满整行或整列，消除得分。', orientation:'portrait' },
  { id:'merge-2048', name:'2048', genre:'益智', description:'滑动合并相同数字，慢慢合出 2048。', orientation:'portrait' },
  { id:'water-sort', name:'奶茶大师', genre:'解压', description:'把同色饮料倒在一起，慢慢调制。', orientation:'portrait' },
  { id:'falling-blocks', name:'俄罗斯方块', genre:'益智', description:'移动和旋转方块，填满一行就能消除。', orientation:'portrait' },
].map(game => ({ ...game, cover:`/builtin-games/${game.id}/cover.svg` }))
export type BuiltinGame = typeof BUILTIN_GAMES[number]
export function builtinGame(id:string): BuiltinGame | undefined { return BUILTIN_GAMES.find(game => game.id === id) }
/** Only the current opaque-origin game frame may acknowledge the host nonce. */
export function isBuiltinGameMessage(data: unknown, origin: string, source: unknown, frame: unknown, nonce: string): boolean {
  if (!data || typeof data !== 'object' || !frame || source !== frame || origin !== 'null' || !nonce) return false
  const message = data as Record<string, unknown>
  return message.channel === 'xiaban.game-pack' && message.protocol === 1 && message.nonce === nonce
}
