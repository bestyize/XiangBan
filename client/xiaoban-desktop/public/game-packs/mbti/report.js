/* Deterministic, offline report assembly. No network, prompts or generated scores. */
(() => {
  'use strict'
  const rules = globalThis.XiaobanMbtiRules
  const profiles = globalThis.XiaobanMbtiProfiles
  const family = { NT: { name: '分析家', color: '#8050aa' }, NF: { name: '外交家', color: '#168368' }, SJ: { name: '守护者', color: '#2685ad' }, SP: { name: '探险家', color: '#ac7800' } }
  const poles = {
    E: ['交流让想法活起来', '你更常在互动中获得线索和动力。可以用短交流启动任务，再留一小段时间收拢所得。'],
    I: ['独处帮助你整理自己', '你更常通过安静的时间恢复精力、梳理想法。把集中交流与独立投入分开安排，会更容易保持节奏。'],
    S: ['从具体经验进入问题', '你更常从真实样例、细节和实际步骤建立理解。先试一个可观察的例子，再总结可复用的经验。'],
    N: ['从联系里发现可能', '你更常寻找信息之间的联系和整体方向。为一个设想配上一项事实核对，能帮助想法落地。'],
    T: ['用清楚依据比较选择', '你更常先核对标准、成本与理由。把判断依据说清楚，也邀请相关的人补充实际影响。'],
    F: ['让决定照顾到人的需要', '你更常将感受和价值纳入决定。把在意的影响说具体，会更容易形成可以共同执行的安排。'],
    J: ['有安排，更容易安心投入', '你更常在明确节点和收尾安排中获得节奏。列出少量关键步骤，并为变化留一格余量。'],
    P: ['留有余地，更容易展开探索', '你更常随着新发现调整路径。保留探索空间，同时约定一个检查进展的时间。']
  }
  const balanced = [
    '你在交流与独处之间保留了相近的空间。可以把见面讨论与安静恢复交替安排，观察哪种组合更适合当天。',
    '你同时使用具体经验和整体联想。遇到新任务，可以把实际样例与方向草图放在一起比较。',
    '你的回答同时关注判断依据与人的感受。做决定时，把事实标准与相关人的需要分别写出来，再一起衡量。',
    '你的节奏有弹性：你愿意做计划，也给临时兴起的小事留位置。先抓住重要节点，细节可以逐步调整。'
  ]
  // Each scenario group cites three existing question IDs. These are answer summaries,
  // not additional validated scales or measurements of ability.
  const scenarios = [
    { id:'recharge', section:'life', title:'恢复精力', questions:['ei-1','ei-5','ei-10'], first:['与熟悉的人一起放松','你在恢复精力的情境中更常选择互动。','安排一次轻松的见面，并留十分钟整理自己的感受。'], second:['留一段自己的时间','你在恢复精力的情境中更常选择独处。','在忙碌的一天后，留出一段不被打扰的空档。'] },
    { id:'collaboration', section:'work', title:'梳理工作想法', questions:['ei-3','ei-8','ei-11'], first:['在讨论中逐渐理清','你在梳理工作想法时更常选择先交流。','用十分钟短讨论开始任务，再写下一个明确的下一步。'], second:['先独立整理，再交流','你在梳理工作想法时更常选择先独立思考。','讨论前先写下三个要点，再把问题带进交流。'] },
    { id:'information', section:'work', title:'接收新信息', questions:['sn-1','sn-3','sn-10'], first:['先看例子与实际做法','你在理解新任务和工具时更常选择具体材料。','开始新任务时，请对方给一个实际做成的样例。'], second:['先看原理与整体方向','你在理解新任务和工具时更常选择原理和方向。','先画一张整体草图，再用一个实际例子核对。'] },
    { id:'learning', section:'life', title:'学习与复盘', questions:['sn-4','sn-7','sn-12'], first:['从操作经验里积累','你在学习与复盘时更常选择具体步骤。','完成一次尝试后，记录三个下次可以复用的步骤。'], second:['寻找能够迁移的规律','你在学习与复盘时更常选择联系和新用法。','写下一条新发现的规律，并找一个不同场景试用。'] },
    { id:'conflict', section:'relationships', title:'面对不同意见', questions:['tf-3','tf-7','tf-10'], first:['先把理由与要求讲清','你在分歧情境中更常先比较理由和要求。','讨论分歧时，先列共同目标，再逐项核对双方依据。'], second:['先理解彼此的在意','你在分歧情境中更常先了解顾虑和价值。','讨论分歧时，先互相说清最在意的事，再一起定下一步。'] },
    { id:'support', section:'relationships', title:'回应反馈与烦恼', questions:['tf-2','tf-5','tf-11'], first:['先找到可以改的地方','你在反馈与倾诉情境中更常先关注问题和办法。','给出建议前，先问对方想一起想办法，还是先说一说。'], second:['先接住表达与感受','你在反馈与倾诉情境中更常先关注表达和感受。','听完对方后，用一句话确认理解，再问是否需要一起想办法。'] },
    { id:'planning', section:'life', title:'安排闲暇', questions:['jp-1','jp-3','jp-9'], first:['提前有个大致安排','你在闲暇安排中更常选择先确定计划。','周末只确定一两个关键安排，其余时间留白。'], second:['随心情与现场选择','你在闲暇安排中更常选择保留弹性。','给一次即兴出游定好必要的时间和花费边界。'] },
    { id:'delivery', section:'work', title:'推进与完成', questions:['jp-2','jp-7','jp-10'], first:['沿着节点逐步完成','你在项目与作品情境中更常选择明确节点。','把一项工作分成三个可见的小成果，先完成第一个。'], second:['边探索，边调整路径','你在项目与作品情境中更常选择开放探索。','给正在探索的工作约一个回看日期，届时交出当前版本。'] }
  ]
  function dimensionReport(result, dimension, index) {
    const percent=result.percentages[index], total=result.totals[index], strength=Math.max(percent,100-percent)
    const pole=percent>=50?dimension.first:dimension.second
    const band=total<6?'developing':percent===50?'balanced':strength<=58?'near':strength<75?'moderate':'strong'
    const label={developing:'继续探索',balanced:'两侧相近',near:'偏好灵活',moderate:'倾向较清楚',strong:'倾向更鲜明'}[band]
    let description = band==='developing' ? `这个维度已答 ${total} / 12 题。继续完成这些情境，可以补全你在不同场合的选择。` :
      band==='balanced' || band==='near' ? balanced[index] :
      (band==='strong'?'这一侧在你的回答中出现得更集中。':'你在多数情境中选择了这一侧，也保留着另一种做法。')+poles[pole][1]
    return { ...dimension, percent, total, pole, strength, band, label, title:band==='balanced'?dimension.context:poles[pole][0], description }
  }
  function contextReports(answers,type) {
    if (!answers) return []
    const byId = new Map(rules.questions.map((q,i)=>[q.id,{q,choice:answers[i]}]))
    return scenarios.flatMap(group=>{
      const evidence=group.questions.map(id=>byId.get(id)).filter(x=>x && (x.choice===0||x.choice===1))
      if(evidence.length<2)return []
      const dim=rules.dimensions[evidence[0].q.dimension]
      const first=evidence.filter(({q,choice})=>q.options[choice].pole===dim.first).length
      const majority=Math.max(first,evidence.length-first)
      if(first*2===evidence.length || majority/evidence.length<2/3)return []
      const pole=first*2>evidence.length?dim.first:dim.second
      const [headline,observation,action]=pole===dim.first?group.first:group.second
      return [{id:group.id,section:group.section,title:group.title,headline,observation,action,pole,
        differentFromType:pole!==type[evidence[0].q.dimension],
        evidence:evidence.map(({q,choice})=>({id:q.id,prompt:q.prompt,answer:q.options[choice].text,pole:q.options[choice].pole})),majority}]
    })
  }
  function build(raw) {
    const result=rules.normalizeSave({version:2,lastResult:raw}).lastResult
    if(!result)return null
    const profile=profiles[result.type], dimensions=rules.dimensions.map((d,i)=>dimensionReport(result,d,i))
    const contexts=contextReports(result.answers,result.type)
    const priorities=[...contexts].sort((a,b)=>Number(b.differentFromType)-Number(a.differentFromType) || b.majority-a.majority)
    const actions=[...new Set([...priorities.map(c=>c.action),...profile.growth])].slice(0,3)
    return {version:1,type:result.type,profile,family:family[profile.family],completedAt:result.completedAt,
      answered:result.totals.reduce((a,b)=>a+b,0),dimensions,contexts,actions,
      keywords:dimensions.filter(d=>!['developing','balanced','near'].includes(d.band)).sort((a,b)=>b.strength-a.strength).map(d=>d.title).slice(0,3),
      hasAnswerSnapshot:!!result.answers}
  }
  globalThis.XiaobanMbtiReport={build,scenarios}
})()
