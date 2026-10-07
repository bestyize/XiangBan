export interface ScenarioInfo {
  periodName: string
  greeting: string
  chips: string[]
}

/**
 * Returns time-of-day greeting and prompt chips based on local hour (0-23) and day of week.
 */
export function getScenario(hour: number = new Date().getHours(), isWeekend: boolean = [0, 6].includes(new Date().getDay())): ScenarioInfo {
  if (isWeekend && hour >= 8 && hour < 20) {
    return {
      periodName: '周末时光',
      greeting: '今天放假，好好把时间留给自己。想聊聊天，随时敲我。',
      chips: ['今天有什么推荐玩', '无聊发呆中', '求夸夸', '周末想去散散心'],
    }
  }

  if (hour >= 17 && hour < 20) {
    return {
      periodName: '下班通勤',
      greeting: '下班啦，路上注意安全。今晚打算吃点好吃的犒劳自己吗？',
      chips: ['累瘫了求安慰', '今晚吃点啥', '地铁挤死人了', '终于溜出工位'],
    }
  }

  if (hour >= 20 && hour < 23) {
    return {
      periodName: '居家回血',
      greeting: '到家了吧？把烦人的工作抛到脑后，今晚是属于你自己的时间。',
      chips: ['今天遇到件离谱事', '随便聊聊天', '有什么解压推荐', '下班第二人格启动'],
    }
  }

  if (hour >= 23 || hour < 5) {
    return {
      periodName: '深夜树洞',
      greeting: '夜深了，房间安静下来了。要是脑子还在转，跟我说说话吧。',
      chips: ['有点睡不着', '今天心情有点灰', '陪我待会儿', '不想面对明天'],
    }
  }

  return {
    periodName: '白天时光',
    greeting: '在忙里偷闲吗？给生活留一点缝隙，别把自己逼太紧。',
    chips: ['摸鱼休息一下', '今天有点累', '找小伴说句话', '打起精神来'],
  }
}
