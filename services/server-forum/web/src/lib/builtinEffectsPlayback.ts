/** Animation is only useful while the preview can be seen and motion is allowed. */
export function shouldAnimateEffect(playing:boolean, visible:boolean, hidden:boolean, reducedMotion:boolean): boolean {
  return playing && visible && !hidden && !reducedMotion
}
