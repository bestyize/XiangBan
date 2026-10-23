import corpus from './corpus.json';
export interface CompanionChoice { id: string; label: string }
export interface CompanionReply { id: number; petName: string; text: string; choices: CompanionChoice[]; model: boolean }

// Standalone browser previews use the same authored corpus; desktop state is owned by the host.
let previewCounter = 0;
export function previewReply(scene = 'touch'): CompanionReply {
  const bank = corpus.scenes[scene as keyof typeof corpus.scenes] ?? corpus.scenes.touch;
  const id = ++previewCounter;
  return { id, petName: '小伴', text: bank[(id - 1) % bank.length].replaceAll('{name}', '小伴'), model: false,
    choices: [{ id: 'happy', label: '今天还不错' }, { id: 'tired', label: '有点累了' }] };
}
