import type { OpenSourceProjectSummary } from './openSource'

type CoverEdit = Pick<OpenSourceProjectSummary, 'cover_template' | 'cover_url' | 'cover_variant' | 'cover_accent' | 'cover_lines'>

function screenshot(filename: string, variant: CoverEdit['cover_variant'] = 'wide'): CoverEdit {
  return { cover_url: `/open-source-media/${filename}`, cover_template: 'screenshot', cover_variant: variant }
}

const coverEdits: Partial<Record<string, Partial<CoverEdit>>> = {
  'excalidraw/excalidraw': { cover_url: '/open-source-media/excalidraw-showcase.webp', cover_template: 'screenshot', cover_variant: 'wide' },
  'godotengine/godot': { cover_template: 'screenshot', cover_variant: 'wide' },
  'freecad/freecad': { cover_template: 'screenshot', cover_variant: 'balanced' },
  '4ian/gdevelop': { cover_template: 'screenshot', cover_variant: 'offset' },
  'blender/blender': { cover_template: 'screenshot', cover_variant: 'wide' },
  'vorondesign/voron-2': { cover_template: 'photo' },
  'usememos/memos': { cover_template: 'screenshot', cover_variant: 'wide' },
  'paperless-ngx/paperless-ngx': { cover_template: 'screenshot', cover_variant: 'balanced' },
  'codecrafters-io/build-your-own-x': screenshot('build-your-own-x-github-home.webp', 'balanced'),
  'freecodecamp/freecodecamp': screenshot('freecodecamp-github-home.webp', 'balanced'),
  'ebookfoundation/free-programming-books': screenshot('free-programming-books-github-home.webp', 'balanced'),
  'donnemartin/system-design-primer': screenshot('system-design-primer-github-home.webp', 'balanced'),
  'obra/superpowers': screenshot('superpowers-github-home.webp', 'balanced'),
  'practical-tutorials/project-based-learning': screenshot('project-based-learning-github-home.webp', 'balanced'),
  'thealgorithms/python': screenshot('thealgorithms-python-github-home.webp', 'balanced'),
  'ossu/computer-science': screenshot('ossu-github-home.webp', 'balanced'),
  'yt-dlp/yt-dlp': screenshot('yt-dlp-github-home.webp', 'balanced'),
  'microsoft/markitdown': screenshot('markitdown-github-home.webp', 'balanced'),
  'ollama/ollama': screenshot('ollama-app.webp', 'balanced'),
  'microsoft/vscode': screenshot('vscode-showcase.webp'),
  'penpot/penpot': screenshot('penpot-showcase.webp'),
  'graphiteeditor/graphite': screenshot('graphite-showcase.webp'),
  'obsproject/obs-studio': screenshot('obs-showcase.webp', 'balanced'),
  'langflow-ai/langflow': screenshot('langflow-playground.webp'),
  'rustdesk/rustdesk': screenshot('rustdesk-remote.webp'),
  'stirling-tools/stirling-pdf': screenshot('stirling-dashboard.webp'),
  'toeverything/affine': screenshot('affine-writing.webp', 'balanced'),
  'audacity/audacity': screenshot('audacity-workspace.webp'),
  'jellyfin/jellyfin': screenshot('jellyfin-library.webp'),
  'huggingface/lerobot': { cover_url: '/open-source-media/lerobot-so101.webp', cover_template: 'photo' },
  'wled/wled': { cover_url: '/open-source-media/wled-controller.webp', cover_template: 'photo' },
  'arendst/tasmota': screenshot('tasmota-webui.webp', 'balanced'),
  'werwolv/imhex': screenshot('imhex-editor.webp'),
  'skylot/jadx': screenshot('jadx-gui.webp'),
  'anuken/mindustry': screenshot('mindustry-gameplay.webp'),
  'moonlight-stream/moonlight-qt': screenshot('moonlight-streaming.webp'),
  'anthropics/skills': screenshot('anthropic-skills-github-home.webp', 'balanced'),
  'punkpeye/awesome-mcp-servers': screenshot('awesome-mcp-servers-github-home.webp', 'balanced'),
  'ffmpeg/ffmpeg': screenshot('ffmpeg-github-home.webp', 'balanced'),
  'vercel/next.js': screenshot('nextjs-github-home.webp', 'balanced'),
  'microsoft/web-dev-for-beginners': screenshot('web-dev-for-beginners-github-home.webp', 'balanced'),
}

export function previewOpenSourceCover(project: OpenSourceProjectSummary): OpenSourceProjectSummary {
  const key = project.full_name.toLowerCase()
  return { ...project, ...coverEdits[key] }
}
