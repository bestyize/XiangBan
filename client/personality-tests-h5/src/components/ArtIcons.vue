<script setup lang="ts">
import type { TestId } from '../engine/types';

defineProps<{
  kind: TestId | 'animal-hero';
  animalKey?: string;
}>();
</script>

<template>
  <!-- 封面小波普插画 -->
  <svg v-if="kind === 'sbti'" viewBox="0 0 96 76" width="78" height="62" aria-hidden="true">
    <rect x="16" y="10" width="64" height="54" rx="18" fill="#FFFDF8" stroke="#18181B" stroke-width="3.2" />
    <circle cx="37" cy="34" r="4.2" fill="#18181B" />
    <circle cx="59" cy="34" r="4.2" fill="#18181B" />
    <path d="M34 48 Q48 56 62 48" stroke="#18181B" stroke-width="3.2" fill="none" stroke-linecap="round" />
    <circle cx="76" cy="18" r="10" fill="#FF4D5E" stroke="#18181B" stroke-width="2.6" />
    <text x="76" y="22" text-anchor="middle" font-size="11" font-weight="900" fill="#FFFDF8">?</text>
  </svg>

  <svg v-else-if="kind === 'pantone'" viewBox="0 0 96 76" width="78" height="62" aria-hidden="true">
    <circle cx="38" cy="38" r="22" fill="#FFD89B" stroke="#18181B" stroke-width="3.2" />
    <circle cx="58" cy="38" r="22" fill="#5468A8" fill-opacity="0.88" stroke="#18181B" stroke-width="3.2" />
    <circle cx="48" cy="38" r="5" fill="#FFFDF8" />
  </svg>

  <svg v-else-if="kind === 'animal'" viewBox="0 0 98 76" width="80" height="62" aria-hidden="true">
    <path d="M18 56 C18 34, 30 24, 49 24 C68 24, 80 34, 80 56 C80 66, 68 70, 49 70 C30 70, 18 66, 18 56 Z" fill="#C48F5F" stroke="#18181B" stroke-width="3.2" />
    <circle cx="49" cy="16" r="8.5" fill="#FF8C2A" stroke="#18181B" stroke-width="2.6" />
    <path d="M36 44 Q40 41 44 44 M54 44 Q58 41 62 44" stroke="#18181B" stroke-width="2.8" fill="none" stroke-linecap="round" />
    <ellipse cx="49" cy="51" rx="5.2" ry="3.2" fill="#18181B" />
  </svg>

  <svg v-else-if="kind === 'manual'" viewBox="0 0 96 76" width="78" height="62" aria-hidden="true">
    <rect x="20" y="8" width="54" height="58" rx="10" fill="#FFFDF8" stroke="#18181B" stroke-width="3.2" />
    <path d="M32 24 H62 M32 36 H58 M32 48 H48" stroke="#18181B" stroke-width="3.2" stroke-linecap="round" />
    <circle cx="69" cy="54" r="11" fill="#FF4D5E" stroke="#18181B" stroke-width="2.6" />
    <text x="69" y="58" text-anchor="middle" font-size="12" font-weight="900" fill="#FFFDF8">!</text>
  </svg>

  <svg v-else-if="kind === 'checkup'" viewBox="0 0 98 76" width="80" height="62" aria-hidden="true">
    <rect x="12" y="18" width="66" height="38" rx="10" fill="#FFFDF8" stroke="#18181B" stroke-width="3.2" />
    <rect x="78" y="30" width="8" height="14" rx="3" fill="#18181B" />
    <rect x="19" y="25" width="16" height="24" rx="4" fill="#FF4D5E" stroke="#18181B" stroke-width="2.2" />
    <path d="M53 24 L46 38 H54 L48 50" stroke="#18181B" stroke-width="3" fill="none" stroke-linecap="round" stroke-linejoin="round" />
  </svg>

  <svg v-else-if="kind === 'classic16'" viewBox="0 0 96 76" width="78" height="62" aria-hidden="true">
    <polygon points="48,7 82,38 48,69 14,38" fill="#FFFDF8" stroke="#18181B" stroke-width="3.2" stroke-linejoin="round" />
    <circle cx="48" cy="38" r="12" fill="#E4F857" stroke="#18181B" stroke-width="3" />
    <circle cx="48" cy="38" r="4.5" fill="#18181B" />
  </svg>

  <!-- 二期 6 款全新波普封面插画 -->
  <svg v-else-if="kind === 'ingredients'" viewBox="0 0 96 76" width="78" height="62" aria-hidden="true">
    <rect x="20" y="8" width="56" height="60" rx="9" fill="#FFFDF8" stroke="#18181B" stroke-width="3.2" />
    <rect x="27" y="16" width="42" height="10" rx="2" fill="#18181B" />
    <path d="M27 35 H69 M27 45 H60 M27 55 H50" stroke="#18181B" stroke-width="3.2" stroke-linecap="round" />
    <circle cx="67" cy="54" r="8" fill="#FF4D5E" stroke="#18181B" stroke-width="2.4" />
  </svg>

  <svg v-else-if="kind === 'xianxia'" viewBox="0 0 96 76" width="78" height="62" aria-hidden="true">
    <rect x="29" y="7" width="38" height="62" rx="6" fill="#FFF8E7" stroke="#8E231C" stroke-width="3.2" />
    <circle cx="48" cy="28" r="9.5" fill="#E3203C" stroke="#2A1D12" stroke-width="2.2" />
    <path d="M37 46 H59 M40 55 H56" stroke="#8E231C" stroke-width="3.2" stroke-linecap="round" />
  </svg>

  <svg v-else-if="kind === 'rpg'" viewBox="0 0 96 76" width="78" height="62" aria-hidden="true">
    <rect x="13" y="15" width="70" height="46" rx="14" fill="#19231E" stroke="#18181B" stroke-width="3.2" />
    <path d="M30 31 V45 M23 38 H37" stroke="#E4F857" stroke-width="3.6" stroke-linecap="round" />
    <circle cx="61" cy="34" r="4.5" fill="#FF4D5E" />
    <circle cx="69" cy="42" r="4.5" fill="#6BE5A8" />
  </svg>

  <svg v-else-if="kind === 'bigfive'" viewBox="0 0 96 76" width="78" height="62" aria-hidden="true">
    <polygon points="48,9 78,31 67,65 29,65 18,31" fill="#FFFDF8" stroke="#18181B" stroke-width="3" stroke-linejoin="round" />
    <polygon points="48,19 69,34 60,55 36,53 29,34" fill="#6BE5A8" stroke="#18181B" stroke-width="2.4" stroke-linejoin="round" />
    <circle cx="48" cy="39" r="3.5" fill="#18181B" />
  </svg>

  <svg v-else-if="kind === 'attachment'" viewBox="0 0 96 76" width="78" height="62" aria-hidden="true">
    <rect x="17" y="10" width="62" height="56" rx="11" fill="#FFFDF8" stroke="#18181B" stroke-width="3.2" />
    <path d="M48 10 V66 M17 38 H79" stroke="#18181B" stroke-width="2.2" stroke-dasharray="4 3" />
    <circle cx="62" cy="25" r="7" fill="#FF4D5E" stroke="#18181B" stroke-width="2.4" />
    <circle cx="34" cy="51" r="5" fill="#8DD3FF" stroke="#18181B" stroke-width="2.2" />
  </svg>

  <svg v-else-if="kind === 'darktriad'" viewBox="0 0 96 76" width="78" height="62" aria-hidden="true">
    <polygon points="48,10 80,62 16,62" fill="#18181B" stroke="#FFFDF8" stroke-width="3" stroke-linejoin="round" />
    <circle cx="48" cy="43" r="9.5" fill="#FF4D5E" stroke="#FFFDF8" stroke-width="2.2" />
  </svg>

  <!-- 动物塑结果页大尺寸萌系插画（按动物原型自动换色与细节） -->
  <svg v-else viewBox="0 0 150 118" width="144" height="114" aria-label="动物塑插画">
    <ellipse cx="75" cy="105" rx="46" ry="7" fill="#EFE0CC" />
    <!-- 狐狸 / 猫 / 雪豹尖耳，其余圆耳 -->
    <template v-if="animalKey === 'fox' || animalKey === 'blackcat' || animalKey === 'collie'">
      <polygon points="40,46 34,18 58,34" :fill="animalKey === 'blackcat' ? '#26252B' : animalKey === 'fox' ? '#D97B43' : '#3C3A42'" stroke="#18181B" stroke-width="3" stroke-linejoin="round" />
      <polygon points="110,46 116,18 92,34" :fill="animalKey === 'blackcat' ? '#26252B' : animalKey === 'fox' ? '#D97B43' : '#3C3A42'" stroke="#18181B" stroke-width="3" stroke-linejoin="round" />
    </template>
    <template v-else>
      <ellipse cx="47" cy="40" rx="8.5" ry="10" transform="rotate(-24 47 40)" :fill="animalKey === 'samoyed' ? '#FFFDF8' : animalKey === 'koala' ? '#8E96A0' : animalKey === 'leopard' ? '#D9D3C7' : '#A87448'" stroke="#18181B" stroke-width="3" />
      <ellipse cx="103" cy="40" rx="8.5" ry="10" transform="rotate(24 103 40)" :fill="animalKey === 'samoyed' ? '#FFFDF8' : animalKey === 'koala' ? '#8E96A0' : animalKey === 'leopard' ? '#D9D3C7' : '#A87448'" stroke="#18181B" stroke-width="3" />
    </template>

    <!-- 主头身 -->
    <path
      d="M30 82 C28 50, 48 34, 75 34 C102 34, 122 50, 120 82 C118 98, 102 104, 75 104 C48 104, 32 98, 30 82 Z"
      :fill="
        animalKey === 'blackcat' ? '#2C2B33' :
        animalKey === 'samoyed' ? '#FFFDF8' :
        animalKey === 'fox' ? '#E4894D' :
        animalKey === 'koala' ? '#A4ACB6' :
        animalKey === 'leopard' ? '#E8E3D9' :
        animalKey === 'redpanda' ? '#D66A38' :
        animalKey === 'collie' ? '#3B3942' :
        '#C48F5F'
      "
      stroke="#18181B"
      stroke-width="3.4"
    />

    <!-- 脸颊内衬 -->
    <ellipse
      cx="75"
      cy="74"
      rx="24"
      ry="15"
      :fill="animalKey === 'blackcat' ? '#3C3B44' : animalKey === 'samoyed' ? '#FFF7EC' : '#F4DFC8'"
      opacity="0.88"
    />

    <!-- 眼睛（藏狐半眯、水豚微笑闭眼、其余大圆眼） -->
    <template v-if="animalKey === 'fox'">
      <line x1="51" y1="63" x2="64" y2="63" stroke="#18181B" stroke-width="3.2" stroke-linecap="round" />
      <circle cx="58" cy="66" r="2.6" fill="#18181B" />
      <line x1="86" y1="63" x2="99" y2="63" stroke="#18181B" stroke-width="3.2" stroke-linecap="round" />
      <circle cx="92" cy="66" r="2.6" fill="#18181B" />
    </template>
    <template v-else-if="animalKey === 'capybara' || animalKey === 'koala'">
      <path d="M53 64 Q58 59 64 64 M86 64 Q92 59 97 64" stroke="#18181B" stroke-width="3" fill="none" stroke-linecap="round" />
    </template>
    <template v-else>
      <circle cx="58" cy="64" r="4.5" :fill="animalKey === 'blackcat' ? '#E4F857' : '#18181B'" />
      <circle cx="92" cy="64" r="4.5" :fill="animalKey === 'blackcat' ? '#E4F857' : '#18181B'" />
    </template>

    <!-- 鼻子与微笑 -->
    <ellipse cx="75" cy="71" rx="6" ry="3.8" fill="#18181B" />
    <path d="M69 78 Q75 83 81 78" stroke="#18181B" stroke-width="2.5" fill="none" stroke-linecap="round" />

    <!-- 腮红 -->
    <circle cx="47" cy="73" r="5.5" fill="#FF8C8C" opacity="0.58" />
    <circle cx="103" cy="73" r="5.5" fill="#FF8C8C" opacity="0.58" />

    <!-- 水豚头顶小橘子 -->
    <template v-if="!animalKey || animalKey === 'capybara'">
      <circle cx="75" cy="24" r="11.5" fill="#FF8C2A" stroke="#18181B" stroke-width="2.8" />
      <ellipse cx="80" cy="13" rx="6" ry="3" transform="rotate(-20 80 13)" fill="#4ECB71" stroke="#18181B" stroke-width="2" />
    </template>
  </svg>
</template>
