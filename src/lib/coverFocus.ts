/** Image point, in percent, that should sit in the middle of a cover frame. */
export function clampFocus(value: number): number {
  if (!Number.isFinite(value)) return 50;
  return Math.min(100, Math.max(0, value));
}

export function roundFocus(value: number): number {
  return Math.round(clampFocus(value) * 10) / 10;
}

export function renderedCoverSize(
  imageWidth: number,
  imageHeight: number,
  frameWidth: number,
  frameHeight: number,
): { width: number; height: number } {
  if (imageWidth < 1 || imageHeight < 1 || frameWidth < 1 || frameHeight < 1) {
    return { width: frameWidth, height: frameHeight };
  }
  const scale = Math.max(frameWidth / imageWidth, frameHeight / imageHeight);
  return { width: imageWidth * scale, height: imageHeight * scale };
}

/** object-position that pins the stored image point to the center of this frame. */
export function objectPosition(
  imageWidth: number,
  imageHeight: number,
  frameWidth: number,
  frameHeight: number,
  focusX: number,
  focusY: number,
): string {
  const rendered = renderedCoverSize(imageWidth, imageHeight, frameWidth, frameHeight);
  const x = axisPosition(frameWidth, rendered.width, focusX);
  const y = axisPosition(frameHeight, rendered.height, focusY);
  return `${roundPercent(x)}% ${roundPercent(y)}%`;
}

function roundPercent(value: number): number {
  return Math.round(value * 10) / 10;
}

function axisPosition(frame: number, rendered: number, focus: number): number {
  const overflow = rendered - frame;
  if (overflow < 1) return 50;
  const point = (clampFocus(focus) / 100) * rendered;
  const position = ((frame / 2 - point) / (frame - rendered)) * 100;
  return clampFocus(position);
}

/**
 * Drag moves the photo. The image point under the middle of the frame is what we store.
 * An axis the frame does not crop stays put, so a wide banner does not invent a sideways shift.
 */
export function focusAfterDrag(
  startX: number,
  startY: number,
  dx: number,
  dy: number,
  renderedWidth: number,
  renderedHeight: number,
  frameWidth: number,
  frameHeight: number,
): { x: number; y: number } {
  const x =
    renderedWidth - frameWidth < 1
      ? startX
      : startX - (dx / renderedWidth) * 100;
  const y =
    renderedHeight - frameHeight < 1
      ? startY
      : startY - (dy / renderedHeight) * 100;
  return { x: clampFocus(x), y: clampFocus(y) };
}
