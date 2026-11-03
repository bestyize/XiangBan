type ProjectIntro = {
  purpose: string
  scenarios: string
  start: string
  requirement: string
  link: { label: string; url: string }
}

export const openSourceProjectIntros: Partial<Record<string, ProjectIntro>> = {
  'excalidraw/excalidraw': {
    purpose: '在浏览器里画手绘风格的流程图、界面草图和白板，可以邀请同事一起编辑，也能导出 PNG 或 SVG。',
    scenarios: '开会时一起梳理流程，给同事讲清系统里几个模块的关系，或在写代码前画一张界面草图。',
    start: '打开在线画板，画两个方框，用箭头连起来。画完可以导出图片，或发协作链接让同事加入。',
    requirement: '单人绘图只需浏览器。画板默认保存在当前浏览器里，需要跨设备继续编辑时，可以导出 .excalidraw 文件。',
    link: { label: '打开在线画板', url: 'https://excalidraw.com/' },
  },
  'ollama/ollama': {
    purpose: '把大语言模型下载到自己的电脑上运行，可以直接对话，也能通过本地 API 接到自己的程序里。',
    scenarios: '想在本机试不同模型，或者给一个正在开发的应用接上本地模型做原型。',
    start: '按官方指南安装 Ollama，从模型库选一个较小的模型。打开终端运行 ollama run 加模型名，首次运行会拉取模型。',
    requirement: '首次下载模型需要联网和磁盘空间。能跑多大的模型取决于机器的内存或显存，先选小模型更容易跑通。',
    link: { label: '查看官方快速入门', url: 'https://docs.ollama.com/quickstart' },
  },
  'freecad/freecad': {
    purpose: '用带尺寸约束的草图做三维零件。改动孔径或板厚后，可以按新尺寸更新模型。',
    scenarios: '给 3D 打印设计一个支架，按实物尺寸重画替换零件，或者给同一个零件做几种尺寸。',
    start: '安装 FreeCAD 后打开零件设计工作台，新建一个草图，画一个矩形、标上长宽，再用凸台把它拉出厚度。',
    requirement: '需要安装桌面应用。第一次建模要熟悉草图约束和工作台切换，可以先从一个简单零件练起。',
    link: { label: '查看官方安装包', url: 'https://www.freecad.org/downloads.php' },
  },
}
