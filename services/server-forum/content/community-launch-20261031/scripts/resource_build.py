"""Author 35 manual templates and 15 visual packs. See README-resources.md."""
from pack_build_common import *
# All templates below are manual documents: no fabricated importer or executor.
SPECS={
'routine':[
('quiet-start','安静开工七分钟','从收件箱和待办中选出今天的一件关键事，含可填写晨间流程。',['先写下当前精力与最晚结束时间，不评判状态','收集脑中事项到暂存区，先不执行','按今天必须、可以推迟、可以取消分成三栏','从必须栏选一件可在一个工作段推进的事','写出两分钟内能开始的动作，清空桌面只留相关材料'],{'开始时间':'09:10','精力':'一般','最晚结束':'18:00','关键事':'写完导入功能验收清单','两分钟动作':'打开需求文档并列出三条失败路径','暂存事项':['回复邀请邮件','整理截图']},'每天开始工作时','7分钟'),
('clear-shutdown','下班收尾十分钟','记录停在哪、下一步和待回应事项，减少第二天重新找上下文。',['列出今天实际完成的交付物，给每项附本地文件名或任务编号','记录未完成工作的当前状态及阻塞原因','为明天第一件事写可直接执行的一步','把待他人回应的事项独立列出，并写下次查看时间','关闭不再需要的页面，确认文档已保存后结束'],{'今日交付':['接口字段表 v2.md'],'停在':'异常分支尚未补完','明日第一步':'补写空数组返回示例','待回应':[{'事项':'设计确认空态文案','下次查看':'明天上午'}]},'每天结束工作前','10分钟'),
('weekly-reset','周五桌面归零','把散落文件、下载和待办收拢成一个可检索的周末收尾记录。',['记录本周三个成果与各自位置','仅处理桌面和下载目录中新文件，先分类不批量删除','把参考资料移到对应项目并写一句用途','逐项决定未完成待办继续、延期或取消','写下周一启动清单，确认重要文件已有自己的备份'],{'本周成果':['完成作品包首版'],'待分类':['notes-final.md','cover-v3.png'],'保留位置':'项目/发布素材','延期事项':'旧截图整理','周一启动':['核对清单','请同伴试用']},'每周最后一个工作日','20分钟'),
('context-bridge','任务切换接力条','切换项目前留一张接力条，把当前位置和恢复动作写给未来的自己。',['写出当前任务的一个具体目标','记录最近一次已确认有效的结果','记录尚未验证的猜测，明确标为猜测','列出恢复任务所需的三个材料位置','写出下一次打开后的一步操作，再切换任务'],{'目标':'找出导出图片偏色原因','已确认':'源PNG颜色正常','猜测':'浏览器合成层可能改变透明背景','材料':['issue-28','test.png','颜色对照表.md'],'恢复动作':'用不透明背景复测同一图片'},'需要切换项目之前','3分钟'),
('study-loop','学习闭环四步卡','先预测、再学习、再回忆、最后用一个小例子检验理解。',['写出本次想回答的一个问题和自己的预测','阅读或练习一个小段内容，并注明材料位置','关掉材料，用自己的话复述三个要点','做一个最小练习或反例，不会的地方标注出来','把未解决的问题移入下次学习入口'],{'问题':'为什么排序必须稳定','预测':'相同分数的记录需要保留先后次序','材料':'本地排序笔记第2节','回忆要点':['比较条件','相等分支','稳定性'],'练习':'给同分记录添加原始序号再排序','未解决':'合并排序的空间代价'},'每次短时学习','25分钟')],
'reminder':[
('screen-pause','屏幕间歇提醒文案包','可手动设置的短暂停顿提示，含不打断会议的轻柔版本。',['先选择工作日内允许提醒的时段','按自己的习惯决定间隔，不把示例当作健康处方','在系统提醒工具中手工录入一条文案','设置会议、演示和专注时段静音','试用一天后调整频率，删除让自己分心的提醒'],{'提醒标题':'换个姿势，看看远处','温和文案':'这一段可以先告一段落，愿意的话起身活动一下。','建议触发':'由你手动决定间隔','静音场景':['会议','演示'],'试用记录':'下午提醒过密，减少一次'},'自行安排的工作间隙','不含自动定时'),
('plant-round','绿植巡看清单','用观察而非固定浇水天数管理五个常见检查点，附盆栽记录表。',['给每盆植物写名称与摆放位置','查看土表、叶片和盆底，记录实际观察','先确认该植物自己的照料要求，再决定是否处理','处理后写下做了什么，不用统一浇水量','在自己的日历中安排下次查看，季节变化时重新评估'],{'植物':'书桌绿植A','位置':'东侧窗边','土表观察':'仍略湿','叶片观察':'无明显变化','本次动作':'暂不浇水','下次查看':'周六上午'},'每次巡看绿植时','观察记录模板'),
('bill-check','订阅到期自查表','集中查看订阅名称、下次扣费和取消入口，不保存卡号或密码。',['仅记录服务名称，不抄录账号密码和支付凭据','从服务自己的账单页面确认下次日期与金额','区分必需、偶尔使用、准备取消三类','想取消时前往该服务官方设置自行确认','保存取消确认信息，并安排下次手动核查'],{'服务名称':'示例设计工具','下次日期':'请填写账单显示日期','金额':'请填写并注明币种','使用判断':'最近一个月未用','动作':'到服务设置核实是否保留','确认材料':'本地取消记录文件名'},'自己选定的每月核查日','不代为取消或付款'),
('backup-check','个人备份巡检提醒','用抽样打开文件验证备份，而不只看同步完成提示。',['选择一个需要保护的文件夹，记录本机路径','确认备份目标和最近完成时间','抽取一个小文件从备份位置打开','核对文件内容或你已有的校验值，记下结果','出现差异时停止覆盖，先保留两份并查原因'],{'来源':'文档/作品集','备份位置':'本人外置盘/备份','最近完成':'填写时间','抽样文件':'readme.md','打开结果':'可读取，段落一致','异常处理':'保留来源和备份，暂不覆盖'},'手动完成备份之后','含抽样恢复步骤'),
('leave-desk','离桌前五件事','适用于出门或长时间离开工位的自选核对卡，可删去不适用项。',['保存当前文件并留下恢复任务的一句话','检查需要随身携带的物品是否齐全','确认不需要继续运行的设备状态','按自己的使用环境确认门窗与桌面物品','锁定电脑，并在有需要时告知同伴离开时间'],{'恢复提示':'从需求表第三项继续','随身物品':['钥匙','手机','证件'],'设备确认':'不需要的台灯已关闭','环境确认':'按当前场所要求检查','预计返回':'自行填写'},'每次较长时间离桌前','可删减清单')],
'task':[
('release-readiness','小版本发布核对单','把变更、回退、说明和验证证据整理成可执行的发布前任务表。',['写出本次变更范围与明确不包含的内容','列出一条正常路径和两条失败路径并记录结果','确认版本号、产物名和校验值来自同一次构建','写明回退入口、责任人和需要保留的数据','由实际负责人决定是否发布并留下完成记录'],{'版本':'0.2.1','范围':'修复空列表显示','正常路径':'有内容时可正常展示','失败路径':['无网络','空列表'],'回退':'恢复上一版本静态资源','发布决定':'待负责人确认'},'准备小版本交付','证据优先'),
('bug-evidence','缺陷复现证据袋','用环境、步骤、期望和实际结果写一份能让他人复现的问题报告。',['记录产品版本、系统和可公开的运行环境','用最少步骤从一个确定的初始状态开始复现','分别写期望结果与实际结果，不混入修复猜测','附经过脱敏的截图或日志文件名','在干净状态重复一次，标记发生频率及临时绕法'],{'环境':'macOS / 产品版本待填','初始状态':'新建空项目','复现步骤':['打开列表','清空筛选','刷新'],'期望':'展示空态提示','实际':'内容区域空白','频率':'3次中3次','附件':['empty-state.png']},'提交缺陷之前','不含真实用户数据'),
('meeting-followups','会议行动项落地表','把讨论结论拆成负责人、完成条件和检查时间，避免只有会议摘要。',['先分开记录已决定、未决定和仅供参考的事项','每个行动项只保留一个主负责人','把完成条件写成可观察的交付物','让相关人确认时间而不是替别人承诺','到检查日记录完成、阻塞或重新约定'],{'决定':'先交付只读预览','未决定':'自动发布时机','行动项':[{'事项':'完成预览页','负责人':'待确认','交付物':'可访问的本地预览及字段说明','检查时间':'双方确认后填写'}]},'会议结束后','不自动派发任务'),
('home-inventory','搬家小物盘点单','按房间和箱号记录物品、位置与易碎标记，便于打包和找回。',['先按房间建立清单，避免一次覆盖全部住处','为每个箱子分配简短唯一编号','逐件写名称、数量和放入的箱号','将易碎、急用或暂不打包的物品单独标记','到新地点按箱号核对，不在公开记录中写精确住址'],{'房间':'书房','箱号':'S-03','物品':[{'名称':'笔记本','数量':4,'标记':'普通'},{'名称':'台灯','数量':1,'标记':'易碎'}],'急用包':['充电线','钥匙'],'核对':'待到达后确认'},'整理或搬家前','可打印表格'),
('reading-extract','一篇文章三张卡','把阅读结果拆成观点卡、证据卡和应用卡，留下可再次使用的笔记。',['先写文章标题、作者和你实际读到的来源位置','用一句话复述作者的核心观点','摘录必要的短证据并区分转述与原话','写一个与你当前工作相关的应用场景','记录不认同或尚不能验证的部分'],{'来源':'本地文章.pdf 第3页','观点卡':'更短的反馈周期可帮助更早发现假设错误','证据卡':'作者给出的案例，尚需结合自己的场景核验','应用卡':'将周末统一检查改为每天结束前看一次结果','未验证':'是否适用于跨团队依赖'},'读完一篇材料之后','附明确来源栏')],
'goal':[
('reading-four-weeks','四周主题阅读路线','从一个问题出发，每周留下摘要与反例，目标是理解而非堆阅读量。',['选一个想回答的问题并写出起点认知','第一周读一份入门材料并制作术语表','第二周对照两种解释，记录分歧','第三周用一个小案例检验理解','第四周写一页总结，回看起点并决定后续'],{'目标问题':'如何设计清晰的错误提示','周期':'4周','最小投入':'每周2次，每次由自己决定时长','里程碑':['术语表','对照笔记','案例测试','一页总结'],'成功标准':'能解释一个真实页面的问题与改法'},'4周','以产出而非时长验收'),
('portfolio-sprint','四周作品集小冲刺','把一个已有项目整理成可展示案例，包含范围、过程、结果和限制。',['明确目标读者与本次只展示的一个项目','收集能公开的过程材料并删去私人信息','搭出背景、决定、实现、结果四段结构','邀请一人试读并记录其理解偏差','修改一次后发布到自己选择的位置，记录待改进点'],{'读者':'希望了解我实现过程的同伴','项目':'离线清单组件','里程碑':['选题与授权确认','四段初稿','试读记录','定稿'],'成功标准':'试读者能说出目标、取舍和实际结果','边界':'不使用未经允许的客户素材'},'4周','不承诺求职结果'),
('declutter-two-weeks','两周数字整理计划','每天只整理一个小范围，保留可追溯的移动记录与误删缓冲。',['第一天记录需要整理的三个目录及保留原则','每次只选一个小批次分类，先移动不永久删除','为保留文件建立清楚名称，不统一覆盖旧版本','每隔几天验证常用链接和项目仍能打开','两周后回顾整理规则，按自己的备份政策处理暂存区'],{'范围':['桌面','下载','项目截图'],'保留原则':'近期项目和唯一原件先保留','批次上限':'每次不超过一个小目录','里程碑':['目录地图','分类试行','链接检查','规则定稿'],'成功标准':'常用资料能在约定位置找到'},'14天','先暂存后决定'),
('typing-practice','二十一天输入练习册','围绕准确性、节奏和自选文本的小步练习，不将速度作为唯一目标。',['选一段不含私人信息的练习文字并记录起点','前一周先追求少改错，记录常错组合','第二周把常错组合做成短练习','第三周用相似难度的新段落对比','每天记录实际练习与感受，漏一天无需补双倍'],{'材料':'自己写的200字短文','起点':'待本人记录','常错组合':['待填'],'里程碑':['起点记录','常错表','短练习','同难度复测'],'成功标准':'在相近材料上更少出错且能保持舒适节奏'},'21天','纸面或表格手动记录'),
('sideproject-map','六周小项目航线','把一个周末想法压缩成可验证的小成果，预留收尾而非无限加功能。',['第一周描述一个目标用户和具体使用场景','第二周做最小原型，列出主动不做的功能','第三周完成一条端到端路径','第四周请少量同伴试用，区分缺陷和新需求','第五周修关键问题，第六周补说明、归档与回顾'],{'场景':'个人快速记录一段学习结论','最小成果':'能新建、查看和导出纯文本','不做':['账号','社交','云同步'],'里程碑':['场景说明','原型','闭环','试用','修复','交接'],'成功标准':'陌生试用者按说明完成一次真实任务'},'6周','包含收尾与范围控制')],
'skill':[
('evidence-audit','证据与断言核对技能','针对用户提供的材料逐项标记支持、矛盾与未知，不编造来源。',['把待审文本拆成可验证的单独断言','只从用户提供且实际可读的材料中收集对应证据','记录材料位置、日期与证据是否直接支持','将断言分类为支持、部分支持、矛盾或无法判断','给出最小修改建议，并保留未解决问题'],{'输入':'待审段落和本地来源摘录','断言':'项目已支持自动导入全部类型','证据':'格式说明只列出五类可导入资源','判断':'部分支持','改写':'项目支持说明中列出的资源导入类型','未知':'其他类型的执行状态'},'给出待核实文本与可读来源','不得伪造引文或假装浏览'),
('issue-triage','问题分诊技能','把混合反馈拆成缺陷、需求、疑问与环境问题，保留原始证据。',['逐条保留反馈原文和可公开的上下文','按实际行为与预期差异判断是否为缺陷','区分用户目标与其提出的实现方案','用影响范围和阻塞程度说明优先级依据','为每项输出下一步收集证据或负责人待确认'],{'原始反馈':'导出按钮没反应，希望自动同步','分类':['待复现缺陷','新增需求'],'缺失信息':['版本','点击后的提示','控制台截图'],'优先级依据':'当前阻塞用户完成导出','下一步':'先复现导出问题再独立评估同步'},'提供一组用户反馈','不替团队承诺排期'),
('release-notes','面向用户的更新说明技能','将提交摘要改写成用户能理解的行为变化，同时保留兼容限制。',['从输入中找出用户可观察的变化','将变更按新增、修复和使用注意整理','每条说明写清触发场景与现在的结果','删去没有证据支持的性能或稳定性夸张描述','列出实际验证范围和尚未确认的限制'],{'技术输入':'空数组时返回空态，修复JSON解析错误','用户说明':'没有收藏内容时会显示引导提示，不再出现空白区域。','验证范围':'桌面浏览器本地验证','尚未确认':'旧手机系统上的显示情况'},'提供真实变更与验证结果','不虚构发布版本或测试通过'),
('csv-quality','CSV质量检查说明技能','按空值、重复、类型和编码检查用户提供的表格片段。',['确认分隔符、编码、列名与每列含义','检查列数一致性和明显的解析问题','按用户定义区分允许空值与缺失数据','检查主键候选重复、日期和数字格式混杂','输出问题行号、影响和建议，不静默修改原始数据'],{'列定义':{'id':'唯一字符串','amount':'十进制数','day':'YYYY-MM-DD'},'样例行':['A01,12.5,2026-10-01','A01,,2026/10/02'],'发现':['id重复','amount缺失','日期格式混杂'],'处理建议':'先确认重复是否为重复订单，不直接删行'},'粘贴脱敏CSV和列定义','只审可见行，不声称已扫全文件'),
('doc-distill','长文结构提炼技能','把用户提供的长文整理成目的、结论、依据与待办，保留不确定性。',['确定目标读者和需要回答的问题','标出材料中明确结论与仅为假设的内容','按主题归并重复信息，不改变原始立场','给结论关联章节或摘录位置','输出简明摘要、行动项和未解决问题'],{'材料':'三段设计讨论摘录','结论':'先交付本地预览','依据位置':'讨论第2段','行动项':'补充空态与错误态说明','未决':'是否需要批量导入','摘要口径':'只覆盖所提供材料'},'提供完整可见的目标文本','遗漏材料必须明确说明')],
'expert':[
('technical-editor','技术文档编辑提示词','聚焦可执行步骤、术语一致性和验证边界的人工对话角色。',['先确认读者、目标任务与实际运行环境','检查术语、前置条件和步骤顺序','对每个操作补齐可观察的成功条件','把未知、计划中与已经实现分开表达','给出最小改稿与需要作者确认的问题'],{'读者':'初次下载资源包的用户','输入问题':'文档说一键应用但只是下载','编辑建议':'改为下载后在对应页面导入','需确认':'该类型当前是否支持导入','输出':'修改稿与变更理由'},'技术说明与操作指南','不假装实际运行用户环境'),
('research-librarian','资料整理员提示词','帮助整理用户已经收集的材料，建立证据目录与检索线索。',['确认本次问题和纳入材料的范围','按来源、日期和主题建立目录','区分一手材料、转述和个人笔记','为每份材料写用途、限制和可检索关键词','输出阅读顺序与仍需补充的证据'],{'研究问题':'比较两个离线方案','资料目录':[{'名称':'方案A说明','类型':'一手说明','日期':'用户提供','用途':'确认输入输出'}],'缺口':'缺少同环境测试结果','建议阅读顺序':['格式约束','实测记录','讨论摘录']},'用户提供的文件或摘录','不声称拥有未提供的全文'),
('meeting-facilitator','小组讨论引导员提示词','为小组讨论准备议程、问题与行动项草稿，由人确认决定。',['确认讨论目标、参与者角色和时间预算','将议题拆成事实、方案与决定三部分','为分歧准备可被证据回答的问题','在结束前整理已决定与未决定事项','输出需参与者确认的行动项，不替他们承诺'],{'会议目标':'决定首版范围','议程':['约束5分钟','方案10分钟','决定5分钟'],'分歧问题':'哪项能力最影响一次完整使用','行动项草稿':'负责人和时间均待本人确认','未决':'次要主题是否延期'},'会前准备与会后整理','不代替参会者投票或决策'),
('localization-reader','中文界面校对员提示词','从上下文、按钮动作和错误提示检查中文界面文案。',['确认界面位置、对象、操作结果和字数约束','检查同一对象在不同页面的叫法','让按钮以明确动作表达，避免模糊确认词','检查错误提示是否包含可行的恢复动作','输出原文、建议、理由及依赖上下文'],{'位置':'下载失败提示','原文':'操作失败','建议':'下载未完成，请检查网络后重试','理由':'指出失败对象并给恢复动作','需确认':'是否支持断点续传后再写入文案'},'中文产品文案校对','不编造产品能力或平台支持'),
('product-questioner','产品方案追问者提示词','通过目标、代价和验收条件审视一页方案，不强行增加功能。',['先复述用户问题和方案想改善的行为','核对已有约束和明确不做的事情','找出最关键且尚未验证的假设','提出一个成本更小的验证办法','输出支持、担忧、验收问题与下一步实验'],{'方案':'给每个用户做个性化首页','关键假设':'用户更需要自动排序而非自己筛选','较小实验':'先让五位试用者用固定筛选完成任务','验收问题':'是否更快找到需要的作品','保持范围':'本轮不做推荐模型'},'一页产品方案或需求摘录','不把主观偏好写成用户事实')],
'workflow':[
('bug-to-repro','反馈到复现手动工作流','从一条模糊反馈逐步生成可复现报告和交接清单。',['收集原始反馈，去掉不应共享的身份信息','确认版本、起始状态与最短步骤','在自己的授权环境中复现并保存证据','缩小触发条件，分开已知与猜测','写交接报告，列出待验证项与临时绕法'],{'输入':'点击收藏后页面不更新','阶段产物':['脱敏反馈','环境表','最小步骤','证据文件名','交接报告'],'验收':'另一位同伴能照步骤观察到同一结果','停止条件':'缺少授权或无法获得必要环境时先标记未复现'},'反馈 → 环境 → 复现 → 缩小 → 交接','不自动访问用户设备'),
('idea-to-test','想法到小实验手动工作流','用一个可观察的问题检验想法，记录结果而非强行证明正确。',['把想法改写成一个可反驳的假设','定义目标参与者、任务和可观察结果','选择一个不需要完整开发的最小样本','在得到参与者同意后执行并原样记录','根据结果决定继续、修改或停止'],{'想法':'简化收藏入口能减少寻找时间','假设':'首次试用者能在30秒内找到收藏','最小样本':'纸面页面与3位自愿试用者','记录':['用时','误点','口述困惑'],'决定':'若多数找不到入口，先修改信息位置'},'假设 → 样本 → 观察 → 记录 → 决定','小样本不代表总体结论'),
('markdown-to-release','Markdown文章交付工作流','从草稿到可审阅的发布材料，包含图片权利、链接与替代文本检查。',['确认目标读者、文章目的和事实来源','审阅结构、断言与短引用的来源位置','核对图片来源与使用许可，补充替代文本','在本地预览标题层级、代码块和窄屏布局','生成交付清单，由本人在目标平台人工发布'],{'输入':'guide.md 与配图目录','检查项':['标题层级','代码语言','图片许可','替代文本','相对路径'],'产物':['文章定稿','配图清单','预览问题表'],'发布状态':'等待人工发布'},'草稿 → 核实 → 配图 → 预览 → 交付','不自动发布或替你授权素材'),
('photo-archive','照片归档手动工作流','先备份再分类，保留原图与整理记录，避免破坏原始文件。',['先确认照片由自己拥有或获准整理','保留一份未改动原图并验证能读取','按拍摄时间或事件建立目录，不依赖人脸识别','重复候选先暂存并人工对照，不直接删除','抽样核对数量和文件可读性，记录整理规则'],{'输入':'本人相机导出目录','目录规则':'年份/月份-事件','保护措施':'原图目录只读保存','重复处理':'候选放入待核查目录','验收':'抽样图片可打开且原图数量未减少'},'授权 → 备份 → 分类 → 对照 → 验收','不上传照片、不推断人物身份'),
('feature-handover','功能交接手动工作流','把范围、接口、运行方式、证据与已知限制整理成接手包。',['明确交付功能及明确不包含的范围','记录入口、输入输出和必要配置名称，不记录密钥值','列出验证命令与实际执行结果','说明已知限制、失败恢复和回退方式','让接手者按文档完成一次操作并记录缺漏'],{'功能':'离线作品卡预览','输入输出':'JSON引用 → 安全展示卡片','配置名称':['PUBLIC_BASE_URL'],'验证结果':'本地三种空态已检查','限制':'未在旧系统测试','接手验收':'另一人按文档打开预览'},'范围 → 契约 → 验证 → 限制 → 接手','不把未执行测试写成已通过')]
}
for kind,rows in SPECS.items():
 for slug,title,summary,steps,example,trigger,boundary in rows:
  p=make(kind,slug,title,summary,['原创','手动模板',LABEL[kind]],{'manualOnly':True,'execution':'manual-template','automated':False})
  labels=list(example.keys())
  data={'templateVersion':1,'title':title,'usage':'manual','automatic_execution':False,'trigger':trigger,'boundary':boundary,'steps':[{'id':f's{i+1}','action':x,'done':False} for i,x in enumerate(steps)],'record':{k:None for k in labels}}
  if kind=='reminder':data['schedule']={'enabled':False,'timezone':'local','configure_manually':True,'time':None}
  if kind=='goal':data['review']={'current_stage':1,'completed':False,'evidence':[],'next_review':None}
  if kind=='workflow':data['nodes']=[{'id':f'n{i+1}','label':x,'depends_on':[] if i==0 else [f'n{i}'],'status':'not_started','evidence':None} for i,x in enumerate(steps)]
  put(p,kind+'.json',data);put(p,'example.json',example)
  checklist='\n'.join(f'{i+1}. {s}' for i,s in enumerate(steps))
  fields='\n'.join(f'## {k}\n\n（请按自己的实际情况填写）\n' for k in labels)
  put(p,'template.md',f'# {title} · 使用记录\n\n日期：____　使用人：____\n\n'+ '\n'.join('- [ ] '+s for s in steps)+'\n\n'+fields+'\n## 本次回顾\n\n已完成：____\n\n遇到的问题：____\n\n下一步与检查时间：____\n')
  prompt=f'''# {title} · 可复制提示词

你是帮助用户完成“{title}”的协作者。先读取用户实际提供的内容；信息不足时最多询问三个会影响结果的问题。只依据可见材料，不编造来源、执行结果或他人的决定。

目标：{summary}

工作步骤：
{checklist}

输出字段：{'、'.join(labels)}，最后追加“待确认事项”和“下一步”。区分事实、用户偏好与建议。给出可供用户修改的草稿，由用户确认后自行操作。

使用边界：{boundary}。此提示词本身不调用系统、不设置通知、不执行定时任务、不安装任何工具。

## 我的输入

（在这里粘贴自己的脱敏材料、目标和约束）
'''
  put(p,'prompt.md',prompt)
  if kind=='skill':put(p,'SKILL.md',f'---\nname: {slug}\ndescription: {summary}\n---\n\n'+prompt)
  if kind=='expert':put(p,'system-prompt.md',prompt.replace('你是帮助用户完成', '你的角色是专注于此任务的人工对话助手，帮助用户完成'))
  put(p,'README.md',f'''# {title}

{summary}

这是**可下载、手动使用的内容模板**，不是可执行插件。下载不会自动创建任务、提醒、目标或工作流，也不会修改小伴设置。当前无需安装依赖或联网即可阅读和填写；使用 AI 提示词时，由你选择并打开自己的对话工具。

## 怎样使用

1. 解压本 ZIP，先打开 `template.md`，复制一份作为自己的记录。
2. 触发场景：{trigger}。
3. 依照下面的步骤填写；`example.json` 是明确标记的虚构填写示例，不是你的数据。
4. `{kind}.json` 提供同样流程的结构化空白模板，可在文本编辑器中手动填写，不承诺被任何应用自动导入。
5. 需要 AI 帮助整理时，打开 `prompt.md`，复制提示词并补充自己的脱敏输入。{' `SKILL.md` 是可供兼容工具人工登记的技能说明；本包不运行安装器。' if kind=='skill' else ''}

## 具体步骤

{checklist}

## 文件说明

- `template.md`：可勾选、可填写的实际工作页。
- `{kind}.json`：步骤、状态和记录字段的手动数据模板。
- `example.json`：完成一次流程的具体示例。
- `prompt.md`：围绕本任务编写的可复制提示词。
{'- `SKILL.md`：含 name/description 的技能说明。' if kind=='skill' else '- `system-prompt.md`：可人工粘贴的角色说明。' if kind=='expert' else ''}
- `manifest.json`：下载包的类型和版本声明，`manualOnly:true`。

## 完成条件

能按步骤留下自己的记录，区分已完成、未知与下一步。{boundary}。完成之后由你选择保存位置；本包不上传材料、不创建后台任务、不持久保存输入。

## 创作与许可

本包是小伴社区原创示例，文档及代码采用 MIT 许可。示例均为虚构，不包含真实账号、密钥或个人记录。
''')
# Effects: five deliberately different shapes/motions.
effects=[
('amber-paper','琥珀纸屑','暖金纸片缓慢旋落，适合低密度节日桌面。',{'sky':['#122638','#314254'],'layers':[{'count':72,'shape':{'kind':'path','d':'M-0.7,-1 L0.7,-1 L0.7,1 L-0.7,1 Z','style':'fill'},'size':[6,17],'speed':[12,34],'direction':96,'wind':12,'spin':[-1.2,1.2],'flip':{'rate':[0.2,0.7],'min':0.16},'alpha':[0.35,0.8],'colors':['#f4c56a','#d99a55','#ffe2a5']},{'count':22,'shape':{'kind':'glow','core':0.12},'size':[6,12],'speed':[7,16],'direction':270,'alpha':[0.16,0.4],'colors':[['#ffe6ae','#d69a45']]}]}),
('pond-rings','夜池回环','蓝青色水面圆环错落扩散，使用分组涟漪纯数据。',{'sky':['#071d2b','#113a43'],'layers':[{'count':36,'shape':{'kind':'ring','width':[0.7,1.8],'aspect':0.38},'size':[65,155],'speed':0,'alpha':[0.25,0.7],'colors':[['#5ed0cb','#b5efe4'],['#448cba','#91d4ed']],'burst':{'duration':[2.8,4.4],'rest':[2,6],'x':[0.08,0.92],'y':[0.15,0.9],'travel':0,'group':3,'stagger':0.45,'grow':[0.06,1],'fade':'out'}}]}),
('drift-motes','浮光微尘','少量青绿光点在桌面缓缓游荡，保留大面积留白。',{'sky':['#0d2426','#1d3f42'],'layers':[{'count':65,'shape':{'kind':'glow','core':0.17},'size':[5,17],'speed':[1,5],'direction':260,'wander':{'amplitude':[7,18],'frequency':[0.06,0.18]},'twinkle':{'rate':[0.15,0.5],'depth':0.75},'alpha':[0.2,0.75],'blend':'add','colors':[['#d7f5bf','#6ab995'],['#a4ebe7','#458e9c']]}]}),
('sail-clouds','纸舟轻航','小纸舟轮廓沿斜向缓行，带轻微摇摆的几何特效。',{'sky':['#162747','#4f647c'],'layers':[{'count':24,'shape':{'kind':'path','d':'M-1,0 L1,0 L0.55,0.6 L-0.55,0.6 Z M0,-1 L0.8,-0.1 L0,-0.1 Z','style':'both','lineWidth':0.035},'size':[15,32],'speed':[9,22],'direction':348,'sway':{'amplitude':[3,8],'frequency':[0.1,0.2]},'pendulum':0.18,'scatter':0,'alpha':[0.3,0.7],'colors':[['#f5e6bd','#d1af76'],['#dcecf0','#8aa7bc']]}]}),
('blue-hour-trails','蓝调细雨','稀疏银蓝短线斜落，避免高密度遮挡桌面文字。',{'sky':['#11192e','#293c5b'],'layers':[{'count':140,'shape':{'kind':'streak','length':[10,27],'width':[0.45,1.05],'head':0},'size':[1,2],'speed':[75,165],'direction':101,'wind':6,'alpha':[0.12,0.38],'colors':['#abc9ed','#d2e5f1']},{'count':28,'shape':{'kind':'glow','core':0},'size':[3,8],'speed':[9,17],'direction':92,'alpha':[0.08,0.22],'colors':['#8fb1db']}]}),
]
for slug,title,summary,data in effects:
 p=make('effect',slug,title,summary,['原创','低密度','纯数据']);data={'schemaVersion':1,'id':slug,'name':title,'description':summary,**data};put(p,'effect.json',data)
# Wallpapers: CSS-only motion, fully self-contained, with reduced-motion support.
basecss='''*{box-sizing:border-box}html,body{margin:0;width:100%;height:100%;overflow:hidden}body{background:var(--bg);color:var(--ink);font-family:system-ui,sans-serif}main{position:absolute;inset:0;overflow:hidden;isolation:isolate}.caption{position:absolute;left:6vw;bottom:7vh;z-index:10;font-size:clamp(11px,1.3vw,18px);letter-spacing:.35em;opacity:.56}.caption small{display:block;margin-top:10px;font-size:.65em;letter-spacing:.18em}@media(prefers-reduced-motion:reduce){*{animation:none!important}}'''
wallpapers=[
('tidal-silk','潮汐绸带','柔和的蓝青绸带持续呼吸起伏，纯CSS离线动态壁纸。','--bg:#0b2030;--ink:#d2ebed;', '''.ribbon{position:absolute;width:140vw;height:38vh;left:-20vw;top:30vh;border-radius:50%;filter:blur(28px);background:linear-gradient(90deg,transparent,#1f6872,#4e9da0,#315c8b,transparent);opacity:.55;animation:tide 18s ease-in-out infinite alternate}.ribbon:nth-child(2){top:48vh;opacity:.25;animation-delay:-7s;transform:rotate(-14deg)}.ribbon:nth-child(3){top:12vh;opacity:.18;animation-delay:-12s;background:linear-gradient(90deg,transparent,#8e7ab0,#547789,transparent)}@keyframes tide{from{transform:translateY(-7vh) rotate(-13deg) scaleY(.6)}to{transform:translateY(9vh) rotate(9deg) scaleY(1.3)}}''','<div class="ribbon"></div><div class="ribbon"></div><div class="ribbon"></div>','TIDAL SILK','潮来潮往，保持自己的节奏'),
('contour-valley','等高线山谷','细线山谷缓慢漂移，留白充足的地形几何动态背景。','--bg:#18252b;--ink:#e5dfc5;', ''.join(f'.line:nth-child({i+1}){{top:{12+i*2.5}vh;left:{-30+i*.3}vw;animation-delay:{-i*.6}s;opacity:{.12+(i%5)*.055}}}' for i in range(24))+'''.line{position:absolute;width:150vw;height:95vh;border:1px solid #b7c6b2;border-radius:48% 52% 60% 40%;transform:rotate(-18deg);animation:terrain 24s ease-in-out infinite alternate}@keyframes terrain{to{transform:translate(5vw,-3vh) rotate(-12deg)}}''',''.join('<i class="line"></i>' for _ in range(24)),'CONTOUR VALLEY','把目光留给远处的山谷'),
('orbital-studio','轨道工作室','三个不同周期的几何轨道围绕微光缓转，无时间或联网依赖。','--bg:#121627;--ink:#e6d4ab;', '''.sun{position:absolute;left:50%;top:50%;width:13vmin;height:13vmin;margin:-6.5vmin;background:#e6c77b;border-radius:50%;box-shadow:0 0 10vmin #e6c77b33}.orbit{position:absolute;left:50%;top:50%;width:var(--s);height:var(--s);margin:calc(var(--s)/-2);border:1px solid #adbbce33;border-radius:50%;animation:orbit var(--t) linear infinite}.orbit:after{content:"";position:absolute;left:50%;top:-.6vmin;width:1.2vmin;height:1.2vmin;border-radius:50%;background:var(--c);box-shadow:0 0 2vmin var(--c)}@keyframes orbit{to{transform:rotate(360deg)}}''','<div class="sun"></div><div class="orbit" style="--s:34vmin;--t:32s;--c:#8dd9d2"></div><div class="orbit" style="--s:57vmin;--t:55s;--c:#b5a6db"></div><div class="orbit" style="--s:80vmin;--t:87s;--c:#e6bf81"></div>','ORBITAL STUDIO','每条轨道，都有自己的周期'),
('morning-glass','晨光玻璃','杏色晨光穿过半透明几何窗片，适合浅色桌面的慢动画。','--bg:#eee4d3;--ink:#685d50;', '''.light{position:absolute;inset:-30%;background:conic-gradient(from 40deg at 50% 30%,#f6d9a7,transparent 30%,#d8e2d2 45%,transparent 60%,#f6d9a7);filter:blur(24px);animation:morning 28s ease-in-out infinite alternate}.pane{position:absolute;top:15vh;width:23vw;height:65vh;border:1px solid #fff9;background:linear-gradient(120deg,#fff4,transparent);border-radius:50vw 50vw 4px 4px;box-shadow:2vw 2vw 8vw #887d6510}.pane:nth-of-type(2){left:12vw}.pane:nth-of-type(3){left:38.5vw;top:10vh}.pane:nth-of-type(4){left:65vw;top:20vh}@keyframes morning{to{transform:translate(5vw,3vh) rotate(18deg)}}''','<div class="light"></div><div class="pane"></div><div class="pane"></div><div class="pane"></div>','MORNING GLASS','让柔和的光慢慢进来'),
('lantern-harbor','灯影港湾','安静的几何港湾与水面灯影，缓慢起伏的夜色动态壁纸。','--bg:#101f34;--ink:#e8d3a2;', '''.water{position:absolute;top:55%;inset-inline:0;height:45%;background:linear-gradient(#1e3b51,#0d1c30)}.moon{position:absolute;top:17%;right:23%;width:9vmin;height:9vmin;border-radius:50%;background:#e2d2a5;box-shadow:0 0 8vmin #e2d2a522}.house{position:absolute;bottom:45%;left:var(--x);width:7vw;height:var(--h);background:#132a40;border:1px solid #73919e22}.house:after{content:"";position:absolute;left:35%;top:25%;width:25%;height:23%;background:#dab87b;box-shadow:0 1.5vw 0 #dab87b77}.reflection{position:absolute;top:60%;left:var(--x);width:7vw;height:23vh;opacity:.3;background:repeating-linear-gradient(transparent 0 7px,#cfa96b 8px 10px,transparent 11px 17px);clip-path:polygon(35% 0,70% 0,100% 100%,0 100%);animation:water 5s ease-in-out infinite alternate}@keyframes water{to{transform:scaleX(.65) translateY(1vh);opacity:.18}}''','<div class="moon"></div><div class="water"></div>'+''.join(f'<div class="house" style="--x:{x}vw;--h:{h}vh"></div><div class="reflection" style="--x:{x}vw;animation-delay:-{i}s"></div>' for i,(x,h) in enumerate([(20,11),(31,17),(43,12),(54,21),(65,14)])),'LANTERN HARBOR','晚一些，也可以慢一些')]
for slug,title,summary,vars,css,body,en,note in wallpapers:
 p=make('wallpaper',slug,title,summary,['原创','离线','CSS动画'],{'title':title})
 put(p,'index.html',f'<!doctype html><html lang="zh-CN"><meta charset="utf-8"><meta name="viewport" content="width=device-width,initial-scale=1"><title>{title}</title><style>:root{{{vars}}}{basecss}{css}</style><main aria-label="{title}">{body}<div class="caption">{en}<small>{note}</small></div></main></html>')
# No-inline-script widgets; interaction works under script-src self CSP.
widgetcss='''*{box-sizing:border-box}html,body{margin:0;width:100%;height:100%;overflow:hidden;background:transparent;font-family:system-ui,"PingFang SC",sans-serif;color:#263f3c}main{height:100%;padding:20px;border:1px solid #c7d7ce;border-radius:22px;background:#f5f2e8;display:flex;flex-direction:column;gap:10px}h1{font-size:20px;margin:0}p{font-size:13px;line-height:1.6;margin:0;color:#64776e}label,summary{cursor:pointer}button,input{font:inherit}.small{font-size:11px;color:#7e877c}.bottom{margin-top:auto}input[type=checkbox],input[type=radio]{accent-color:#487d69}input:focus-visible,summary:focus-visible{outline:2px solid #528976;outline-offset:3px}'''
widgets=[
('breathing-circle','呼吸留白卡','纯CSS圆环慢慢放大缩小；可勾选暂停，无计时记录或健康承诺。',320,340,'''#pause{position:absolute;left:24px;top:65px}.pause{font-size:12px;padding-left:24px}.circle{margin:auto;width:125px;height:125px;display:grid;place-items:center;border-radius:50%;background:#b7d6c3;box-shadow:0 0 0 13px #dfebdf;animation:breathe 8s ease-in-out infinite}.circle span{font-size:14px;letter-spacing:.2em}@keyframes breathe{50%{transform:scale(1.18);background:#d1dfc3}}#pause:checked~.circle{animation-play-state:paused}''','<h1>留一点空白</h1><input id="pause" type="checkbox"><label class="pause" for="pause">暂停动画</label><div class="circle"><span>慢慢来</span></div><p class="bottom">跟随自己的舒适节奏，不需要追赶圆环。</p>'),
('three-priorities','今日三件事','可勾选、可填写的三项优先事项卡，刷新后恢复空白，不会保存输入。',340,290,'''label.row{display:flex;align-items:center;gap:10px;padding:8px 0;border-bottom:1px solid #d9dfd0}.row input[type=text]{width:100%;min-width:0;border:0;background:transparent;color:#263f3c;font-size:14px;outline-offset:1px}.row input[type=checkbox]:checked+input{text-decoration:line-through;color:#8b988e}''','<h1>今天，只记三件</h1>'+''.join(f'<label class="row"><input type="checkbox" aria-label="完成第{i}项"><input type="text" placeholder="第{i}件要紧事" aria-label="第{i}项内容" maxlength="36"></label>' for i in range(1,4))+'<p class="bottom small">输入只在当前页面；刷新或关闭后不保留。</p>'),
('decision-compass','选择罗盘','三个CSS选项切换追问提示，帮助把模糊选择改成下一步。',340,300,'''input[type=radio]{position:absolute;opacity:0}.tabs{display:flex;gap:7px}.tabs label{padding:7px 10px;border-radius:14px;background:#e3e7d9;font-size:12px}.tip{display:none;padding:14px 0;font-size:18px;line-height:1.7}#a:checked~.tabs label[for=a],#b:checked~.tabs label[for=b],#c:checked~.tabs label[for=c]{background:#487d69;color:#fff}#a:checked~.ta,#b:checked~.tb,#c:checked~.tc{display:block}''','<h1>把选择变小一点</h1><input id="a" type="radio" name="q" checked><input id="b" type="radio" name="q"><input id="c" type="radio" name="q"><div class="tabs"><label for="a">不知选什么</label><label for="b">不敢开始</label><label for="c">需要取舍</label></div><div class="tip ta">哪一个选择，最贴近你眼下真正想解决的问题？</div><div class="tip tb">如果只试两分钟，你能开始哪一步？</div><div class="tip tc">什么可以稍后再做，甚至不做？</div><p class="bottom small">点击标签换一个问题 · 不保存选择</p>'),
('desk-reset-card','工位收尾折页','三个可展开的收尾提示，纯HTML交互，适合作为离桌前检查卡。',340,340,'''details{border-bottom:1px solid #d8dfd0;padding:8px 0}summary{font-size:15px}details p{padding:8px 0 0 15px;font-size:12px}''','<h1>离开前，轻轻收尾</h1><details open><summary>保存进度</summary><p>保存当前文件，写一句“下次从这里继续”。</p></details><details><summary>留下下一步</summary><p>只写一个可开始的动作，别把整份待办搬过来。</p></details><details><summary>收好桌面</summary><p>带走随身物品，按场所需要检查设备并锁屏。</p></details><p class="bottom small">本卡不会检测设备，也不会替你执行操作。</p>'),
('focus-boundary','专注边界门牌','用单选切换当前专注、可打扰或离开状态，仅作本地展示。',340,260,'''input[type=radio]{position:absolute;opacity:0}.tabs{display:flex;gap:6px}.tabs label{padding:6px 9px;font-size:12px;background:#e0e6d9;border-radius:12px}.state{display:none;font-size:32px;letter-spacing:.1em;margin:auto 0}#focus:checked~.sf,#open:checked~.so,#away:checked~.sa{display:block}#focus:checked~.tabs label[for=focus],#open:checked~.tabs label[for=open],#away:checked~.tabs label[for=away]{background:#487d69;color:#fff}''','<h1>我的此刻</h1><input id="focus" type="radio" name="state" checked><input id="open" type="radio" name="state"><input id="away" type="radio" name="state"><div class="tabs"><label for="focus">专注</label><label for="open">可打扰</label><label for="away">暂时离开</label></div><div class="state sf">安静推进中</div><div class="state so">可以聊两句</div><div class="state sa">稍后回来</div><p class="bottom small">仅此卡展示；不会静音系统或同步在线状态。</p>')]
for slug,title,summary,w,h,css,body in widgets:
 p=make('widget',slug,title,summary,['原创','纯CSS','无持久存储'],{'width':w,'height':h,'theme':'transparent'})
 put(p,'index.html',f'<!doctype html><html lang="zh-CN"><meta charset="utf-8"><meta name="viewport" content="width=device-width,initial-scale=1"><title>{title}</title><style>{widgetcss}{css}</style><main>{body}</main></html>')
# Catalog is completed after game creation and rendering.
put(OUT,'draft-catalog.json',CAT)
print('created',len(CAT),'packs')
