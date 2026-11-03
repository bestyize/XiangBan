import type { OpenSourceProjectSummary } from './openSource'

type CoverEdit = Pick<OpenSourceProjectSummary, 'cover_template' | 'cover_url' | 'cover_variant' | 'cover_accent' | 'cover_lines'>

function screenshot(filename: string, variant: CoverEdit['cover_variant'] = 'wide'): CoverEdit {
  return { cover_url: `/open-source-media/${filename}`, cover_template: 'screenshot', cover_variant: variant }
}

const coverEdits: Partial<Record<string, Partial<CoverEdit>>> = {
  'excalidraw/excalidraw': { cover_url: '/open-source-media/excalidraw-showcase.png', cover_template: 'screenshot', cover_variant: 'wide' },
  'godotengine/godot': { cover_template: 'screenshot', cover_variant: 'wide' },
  'freecad/freecad': { cover_template: 'screenshot', cover_variant: 'balanced' },
  '4ian/gdevelop': { cover_template: 'screenshot', cover_variant: 'offset' },
  'blender/blender': { cover_template: 'screenshot', cover_variant: 'wide' },
  'vorondesign/voron-2': { cover_template: 'photo' },
  'usememos/memos': { cover_template: 'screenshot', cover_variant: 'wide' },
  'paperless-ngx/paperless-ngx': { cover_template: 'screenshot', cover_variant: 'balanced' },
  'codecrafters-io/build-your-own-x': screenshot('build-your-own-x-github-home.jpg', 'balanced'),
  'freecodecamp/freecodecamp': screenshot('freecodecamp-github-home.jpg', 'balanced'),
  'ebookfoundation/free-programming-books': screenshot('free-programming-books-github-home.jpg', 'balanced'),
  'donnemartin/system-design-primer': screenshot('system-design-primer-github-home.jpg', 'balanced'),
  'obra/superpowers': screenshot('superpowers-github-home.jpg', 'balanced'),
  'practical-tutorials/project-based-learning': screenshot('project-based-learning-github-home.jpg', 'balanced'),
  'thealgorithms/python': screenshot('thealgorithms-python-github-home.jpg', 'balanced'),
  'ossu/computer-science': screenshot('ossu-github-home.jpg', 'balanced'),
  'yt-dlp/yt-dlp': screenshot('yt-dlp-github-home.jpg', 'balanced'),
  'microsoft/markitdown': screenshot('markitdown-github-home.jpg', 'balanced'),
  'ollama/ollama': screenshot('ollama-app.png', 'balanced'),
  'microsoft/vscode': screenshot('vscode-showcase.png'),
  'penpot/penpot': screenshot('penpot-showcase.jpg'),
  'graphiteeditor/graphite': screenshot('graphite-showcase.png'),
  'obsproject/obs-studio': screenshot('obs-showcase.png', 'balanced'),
  'langflow-ai/langflow': screenshot('langflow-playground.png'),
  'rustdesk/rustdesk': screenshot('rustdesk-remote.png'),
  'stirling-tools/stirling-pdf': screenshot('stirling-dashboard.png'),
  'toeverything/affine': screenshot('affine-writing.jpg', 'balanced'),
  'audacity/audacity': screenshot('audacity-workspace.webp'),
  'jellyfin/jellyfin': screenshot('jellyfin-library.png'),
  'huggingface/lerobot': { cover_url: '/open-source-media/lerobot-so101.webp', cover_template: 'photo' },
  'wled/wled': { cover_url: '/open-source-media/wled-controller.jpg', cover_template: 'photo' },
  'arendst/tasmota': screenshot('tasmota-webui.png', 'balanced'),
  'werwolv/imhex': screenshot('imhex-editor.png'),
  'skylot/jadx': screenshot('jadx-gui.png'),
  'anuken/mindustry': screenshot('mindustry-gameplay.png'),
  'moonlight-stream/moonlight-qt': screenshot('moonlight-streaming.webp'),
  'anthropics/skills': screenshot('anthropic-skills-github-home.jpg', 'balanced'),
  'punkpeye/awesome-mcp-servers': screenshot('awesome-mcp-servers-github-home.jpg', 'balanced'),
  'ffmpeg/ffmpeg': screenshot('ffmpeg-github-home.jpg', 'balanced'),
  'vercel/next.js': screenshot('nextjs-github-home.jpg', 'balanced'),
  'microsoft/web-dev-for-beginners': screenshot('web-dev-for-beginners-github-home.jpg', 'balanced'),
}

export function previewOpenSourceCover(project: OpenSourceProjectSummary): OpenSourceProjectSummary {
  const key = project.full_name.toLowerCase()
  return { ...project, ...coverEdits[key] }
}
