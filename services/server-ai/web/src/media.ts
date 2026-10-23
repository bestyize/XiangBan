import type { IconName } from './components/Icon.vue';
import type { Capability } from './types';

export interface CapabilityInfo {
  id: Capability;
  label: string;
  icon: IconName;
  /** 这项能力给客户端里的什么功能用。 */
  usedBy: string;
  /** 各家在这项能力上的差异，提醒换模型时要连带改什么。 */
  caution: string;
}

export const CAPABILITIES: CapabilityInfo[] = [
  {
    id: 'speech',
    label: '语音',
    icon: 'audio-lines',
    usedBy: '朗读回复（/v1/audio/speech）',
    caution: '音色名每家完全不同（OpenAI 是 alloy、nova，豆包、阿里、MiniMax 用各自的音色 ID），换模型时要连音色一起换。',
  },
  {
    id: 'image',
    label: '图片生成',
    icon: 'image',
    usedBy: '生成图片（/v1/images/generations）',
    caution: '各家支持的尺寸不同，只填上游文档里列出的尺寸。',
  },
  {
    id: 'transcribe',
    label: '语音识别',
    icon: 'mic',
    usedBy: '把录音转成文字（/v1/audio/transcriptions）',
    caution: '大多兼容 OpenAI 的接口，换模型一般只改名字。',
  },
  {
    id: 'realtime',
    label: '实时语音',
    icon: 'waveform',
    usedBy: '实时对话的云端引擎（/v1/realtime）',
    caution: '只能选走 OpenAI Realtime 协议的模型，Gemini Live、豆包实时语音等协议不同，经 New API 用不了。',
  },
];
