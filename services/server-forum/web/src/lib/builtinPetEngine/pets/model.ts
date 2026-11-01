export interface Shape {
  type: 'path' | 'ellipse' | 'rect' | 'image';
  src?: string;
  d?: string; x?: number; y?: number; width?: number; height?: number; rx?: number; ry?: number;
  fill?: string; stroke?: string; strokeWidth?: number; opacity?: number;
}
export interface Part { id: string; name: string; pivot: [number, number]; motion: 'none' | 'sway' | 'left' | 'right'; shapes: Shape[] }
export interface PetDefinition {
  id: string; name: string; description: string; kind: string;
  builtinPlant?: 'pot' | 'pot-white-flower';
  parts: Part[];
}
